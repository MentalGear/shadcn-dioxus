/**
 * Combobox list placement is a GEOMETRY contract, not a visual one (backlog row 210): on iOS an open list could
 * cover its own input.
 *
 * The contract, checked at rest after every step below:
 *   1. the list box never intersects the input box (the anchor is an exclusion zone);
 *   2. the list box stays inside the VISUAL viewport (and inside the fixed containing block, which
 *      `fixedViewportBox` measures: under `html { scrollbar-gutter: stable }` that is NOT
 *      `clientWidth`, see `viewport-box.ts` and `scripts/check-spec-viewport-bound.sh`);
 *   3. the list is not collapsed to nothing while either side of the input has real room.
 *
 * Swept over: viewports 390x844, 390x664, 390x500, 390x400 (the last three are what a phone has
 * left with the keyboard up), 768x1024 and 1024x768, plus 390x320 and 667x375 (iPhone landscape); the demo input near the top / middle / bottom of
 * the viewport (the page is padded so every position is reachable); light and dark; RTL on a subset;
 * and, in each case, after typing that shrinks the list, after clearing it so the list grows back,
 * and after the list's own scroll. A second block simulates the keyboard opening WHILE the list is
 * open (viewport shrinks, page scrolls the input into view as iOS does) and a pinch-zoom (CDP page
 * scale factor: the visual viewport becomes smaller than, and pans inside, the layout viewport).
 *
 * WHAT THIS CANNOT SIMULATE (stated plainly): there is no WebKit in this environment and no real iOS
 * keyboard. A reduced viewport height shrinks the LAYOUT viewport too, which iOS does not; the pinch
 * zoom block is what separates visual from layout viewport, and it is Chromium-only. A green run
 * here is necessary, not sufficient: it needs a manual iPhone pass too.
 */
import { test, expect } from "./fixtures";
import { type Page, type Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { fixedViewportBox } from "./viewport-box";
import { readAtRest, readStatic } from "./animations";

const URL = `${BASE_URL}/component/combobox/`;

type Rect = { top: number; left: number; right: number; bottom: number; width: number; height: number };

const input = (page: Page) => page.getByRole("combobox", { name: "Select framework" });
const list = (page: Page) => page.locator("[role='listbox'][data-state='open']");

const rectOf = (l: Locator): Promise<Rect> =>
    readAtRest(l, (el) => {
        const r = el.getBoundingClientRect();
        return { top: r.top, left: r.left, right: r.right, bottom: r.bottom, width: r.width, height: r.height };
    });

/** Everything wrong with the current placement, as readable strings; empty = fine. */
async function problems(page: Page): Promise<string[]> {
    const fixed = await fixedViewportBox(page);
    const vv = await page.evaluate(() => {
        const v = window.visualViewport;
        return v
            ? { left: v.offsetLeft, top: v.offsetTop, width: v.width, height: v.height, scale: v.scale }
            : null;
    });
    // The bound is the fixed containing block intersected with the visual viewport (in layout
    // coordinates: its origin is the visual viewport's pan offset).
    const bound = {
        top: Math.max(fixed.y, vv ? vv.top : fixed.y),
        left: Math.max(fixed.x, vv ? vv.left : fixed.x),
        bottom: Math.min(fixed.y + fixed.height, vv ? vv.top + vv.height : Infinity),
        right: Math.min(fixed.x + fixed.width, vv ? vv.left + vv.width : Infinity),
    };
    const out: string[] = [];
    if ((await list(page).count()) === 0) return ["list is not open"];
    const i = await rectOf(input(page));
    const l = await rectOf(list(page));
    const fmt = (r: { top: number; left: number; right: number; bottom: number }) =>
        `[${r.left.toFixed(1)},${r.top.toFixed(1)} - ${r.right.toFixed(1)},${r.bottom.toFixed(1)}]`;
    const ctx = `list ${fmt(l)} input ${fmt(i)} bound ${fmt(bound)}`;
    const tol = 0.5;
    if (l.left < i.right - tol && l.right > i.left + tol && l.top < i.bottom - tol && l.bottom > i.top + tol) {
        out.push(`list covers the input (${ctx})`);
    }
    const slack = 1;
    if (l.top < bound.top - slack) out.push(`list leaves the viewport at the top (${ctx})`);
    if (l.bottom > bound.bottom + slack) out.push(`list leaves the viewport at the bottom (${ctx})`);
    if (l.left < bound.left - slack) out.push(`list leaves the viewport at the left (${ctx})`);
    if (l.right > bound.right + slack) out.push(`list leaves the viewport at the right (${ctx})`);
    const roomAbove = i.top - bound.top;
    const roomBelow = bound.bottom - i.bottom;
    if (Math.max(roomAbove, roomBelow) >= 60 && l.height < 24) {
        out.push(`list is collapsed (${l.height.toFixed(1)}px) although there is ${Math.max(roomAbove, roomBelow).toFixed(0)}px of room (${ctx})`);
    }
    return out;
}

async function expectPlacementOk(page: Page, step: string) {
    await expect
        .poll(async () => (await problems(page)).join(" | "), { message: step, timeout: 5000 })
        .toBe("");
}

/**
 * Pads the page so the demo input can be scrolled to any height, then scrolls it to `where` in the
 * (current) viewport. "top" is just below the sticky navbar.
 */
async function placeInput(page: Page, where: "top" | "middle" | "bottom") {
    await page.evaluate(() => {
        document.body.style.paddingTop = "1600px";
        document.body.style.paddingBottom = "1600px";
    });
    const box = await fixedViewportBox(page);
    const nav = page.locator(".dx-preview-navbar, nav[aria-label='Primary']").first();
    const navBottom = (await nav.count()) > 0 ? await readStatic(nav, (el) => el.getBoundingClientRect().bottom) : 0;
    const r = await rectOf(input(page));
    const wanted =
        where === "top" ? navBottom + 8 : where === "middle" ? box.height / 2 - r.height / 2 : box.height - r.height - 8;
    await page.evaluate((dy) => window.scrollBy(0, dy), r.top - wanted);
    // the sticky navbar and `scroll-anchoring` may nudge by a pixel; settle
    await page.waitForTimeout(100);
}

async function open(page: Page) {
    await input(page).click();
    await expect(list(page)).toBeVisible();
}

/** The steps every sweep case runs: open, shrink by typing, grow back, scroll the list itself. */
async function exercise(page: Page) {
    await open(page);
    await expectPlacementOk(page, "after opening");

    await page.keyboard.type("sv"); // one option left: the list shrinks
    await expect(list(page).getByRole("option")).toHaveCount(1);
    await expectPlacementOk(page, "after typing shrinks the list");

    await page.keyboard.press("Backspace");
    await page.keyboard.press("Backspace"); // all options again: the list grows back
    await expect(list(page).getByRole("option")).toHaveCount(7);
    await expectPlacementOk(page, "after clearing the query grows the list");

    await list(page).evaluate((el) => {
        el.scrollTop = el.scrollHeight;
    });
    await page.waitForTimeout(150);
    await expectPlacementOk(page, "after the list's own scroll");
}

const VIEWPORTS = [
    { width: 390, height: 844 },
    { width: 390, height: 664 },
    { width: 390, height: 500 },
    { width: 390, height: 400 },
    { width: 768, height: 1024 },
    { width: 1024, height: 768 },
    // beyond the required sweep: a very short phone viewport and iPhone landscape, where neither
    // side of the input has room for the whole list at most positions
    { width: 390, height: 320 },
    { width: 667, height: 375 },
];
const POSITIONS = ["top", "middle", "bottom"] as const;

for (const viewport of VIEWPORTS) {
    for (const where of POSITIONS) {
        for (const scheme of ["light", "dark"] as const) {
            test(`list never covers its input, stays in the viewport: ${viewport.width}x${viewport.height}, input ${where}, ${scheme}`, async ({
                page,
            }) => {
                await page.setViewportSize(viewport);
                await page.emulateMedia({ colorScheme: scheme });
                await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
                await expect(input(page)).toBeVisible();
                await placeInput(page, where);
                await exercise(page);
            });
        }
    }
}

// RTL: the placement maths is physical (left/right), the page mirrors; the invariants must hold anyway.
for (const viewport of [VIEWPORTS[1], VIEWPORTS[3], VIEWPORTS[5]]) {
    for (const where of POSITIONS) {
        test(`RTL: list never covers its input, stays in the viewport: ${viewport.width}x${viewport.height}, input ${where}`, async ({
            page,
        }) => {
            await page.setViewportSize(viewport);
            await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
            await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
            await expect(input(page)).toBeVisible();
            await placeInput(page, where);
            await exercise(page);
        });
    }
}

test("keyboard opens while the list is open: the list moves to the room that is left and never covers the input", async ({
    page,
}) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await placeInput(page, "middle");
    await open(page);
    await expectPlacementOk(page, "open on the full-height viewport");

    // The keyboard takes ~half of an iPhone screen. The page then scrolls the focused input into the
    // part that is still visible, which is what iOS does.
    await page.setViewportSize({ width: 390, height: 400 });
    await input(page).evaluate((el) => el.scrollIntoView({ block: "center" }));
    await expectPlacementOk(page, "after the viewport shrank (keyboard up)");

    await page.keyboard.type("n");
    await expectPlacementOk(page, "typing with the keyboard up");

    await page.setViewportSize({ width: 390, height: 844 }); // keyboard dismissed
    await expectPlacementOk(page, "after the viewport grew back (keyboard down)");
});

test("pinch-zoom: the visual viewport is smaller than and pans inside the layout viewport", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 664 });
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await placeInput(page, "middle");
    const cdp = await page.context().newCDPSession(page);
    try {
        await cdp.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1.5 });
    } catch (e) {
        test.skip(true, `CDP Emulation.setPageScaleFactor unavailable: ${e}`);
    }
    // Zoomed in, only part of the layout viewport is visible; bring the input into that part.
    await input(page).evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
    await page.waitForTimeout(200);
    const scale = await page.evaluate(() => window.visualViewport?.scale ?? 1);
    test.skip(scale < 1.2, "the page scale factor did not take effect in this browser");

    await open(page);
    await expectPlacementOk(page, "open while zoomed");
    await page.keyboard.type("sv");
    await expectPlacementOk(page, "typing while zoomed");
    await cdp.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1 });
});
