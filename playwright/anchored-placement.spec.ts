/**
 * Anchored overlays are placed in document coordinates (backlog row 477, rule P): a top-layer panel with
 * `position: absolute`, moved only by `transform`, so the page carries it when it scrolls and nothing tracks it.
 *
 * At a phone viewport, with the keyboard pan emulated by a pinch zoom (`Emulation.setPageScaleFactor`: the visual
 * viewport becomes smaller than the layout viewport and pans inside it, which is the frame mismatch an iOS
 * keyboard causes) and by a short viewport:
 *
 *   - the gap between the panel and its anchor stays within 1 px while the page scrolls, on the engine's own
 *     path and on the script path (the anchor-positioning block removed, the iOS 18 case);
 *   - the script path writes no `top`/`left` other than the constant `0`, no `bottom`/`right`, and never
 *     `position: fixed`;
 *   - a context menu opened at a point stays at that document point while the page scrolls;
 *   - a touch scroll that starts outside an open popover leaves it open, a tap outside closes it.
 *
 * Limits: Chromium only (no WebKit here). The pinch zoom reproduces the two viewports, not iOS's 260 ms
 * `visualViewport` event cadence; a green run is necessary, not sufficient, and the device run is the other half.
 */
import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { readAtRest } from "./animations";
import { stripAnchorSupportsBlock } from "./no-anchor-engine";

const FIXTURE = `${BASE_URL}/component/top_layer/`;
const NAV_TIMEOUT = 20 * 60 * 1000;

test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

type Overlay = { name: string; triggerId: string; contentId: string; open: (page: Page) => Promise<void> };

const OVERLAYS: Overlay[] = [
  {
    name: "Popover",
    triggerId: "clip-popover-trigger",
    contentId: "clip-popover-content",
    open: (page) => page.locator("#clip-popover-trigger").click(),
  },
  {
    name: "DropdownMenu",
    triggerId: "clip-dropdown-menu-trigger",
    contentId: "clip-dropdown-menu-content",
    open: (page) => page.locator("#clip-dropdown-menu-trigger").click(),
  },
  {
    name: "Select",
    triggerId: "clip-select-trigger",
    contentId: "clip-select-content",
    open: (page) => page.locator("#clip-select-trigger").click(),
  },
  {
    name: "Combobox",
    triggerId: "clip-combobox-trigger",
    contentId: "clip-combobox-content",
    open: async (page) => {
      await page.locator("#clip-combobox-trigger").click();
      await page.keyboard.press("ArrowDown");
    },
  },
];

async function prepare(page: Page, overlay: Overlay, scriptPath: boolean) {
  await gotoHydrated(page, FIXTURE, { timeout: NAV_TIMEOUT });
  if (scriptPath) await stripAnchorSupportsBlock(page);
  await page.evaluate(() => {
    document.body.style.paddingTop = "3000px";
    document.body.style.paddingBottom = "3000px";
  });
  await page.locator(`#${overlay.triggerId}`).evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
  await overlay.open(page);
  await expect(page.locator(`#${overlay.contentId}`)).toBeVisible();
}

/** Two animation frames and a beat: the script places on the next frame after a scroll or viewport event. */
const settle = (page: Page) =>
  page.evaluate(() => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 60)))));

/** The distance between the panel's and the anchor's nearest edges, whichever side the panel is on. */
const gap = (page: Page, overlay: Overlay) =>
  readAtRest(
    page.locator(`#${overlay.contentId}`),
    (panel, triggerId) => {
      const a = document.getElementById(triggerId as string)!.getBoundingClientRect();
      const r = panel.getBoundingClientRect();
      return Math.max(r.top - a.bottom, a.top - r.bottom, r.left - a.right, a.left - r.right);
    },
    overlay.triggerId,
  );

const SCROLLS = [37, 113, -80, 200, -250, 61];

for (const scriptPath of [false, true]) {
  for (const overlay of OVERLAYS) {
    const engine = scriptPath ? "script path" : "engine path";

    test(`${overlay.name} (${engine}): the gap to the anchor stays within 1px while the page scrolls under a panned visual viewport`, async ({
      page,
    }) => {
      await prepare(page, overlay, scriptPath);
      const cdp = await page.context().newCDPSession(page);
      await cdp.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1.5 });
      await page.locator(`#${overlay.triggerId}`).evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
      await settle(page);
      expect(await page.evaluate(() => window.visualViewport?.scale ?? 1), "the page scale factor took effect").toBeGreaterThan(1.2);
      const start = await gap(page, overlay);
      const trace: number[] = [];
      for (const dy of SCROLLS) {
        await page.evaluate((d) => window.scrollBy(0, d), dy);
        await settle(page);
        const g = await gap(page, overlay);
        trace.push(g);
        expect(Math.abs(g - start), `gap after scrolling ${dy}: ${JSON.stringify({ start, trace })}`).toBeLessThanOrEqual(1);
      }
      await cdp.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1 });
    });

    test(`${overlay.name} (${engine}): the gap to the anchor stays within 1px while the page scrolls with a short viewport`, async ({
      page,
    }) => {
      await prepare(page, overlay, scriptPath);
      await page.setViewportSize({ width: 390, height: 420 });
      await page.locator(`#${overlay.triggerId}`).evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
      await settle(page);
      const start = await gap(page, overlay);
      for (const dy of SCROLLS) {
        await page.evaluate((d) => window.scrollBy(0, d), dy);
        await settle(page);
        const g = await gap(page, overlay);
        expect(Math.abs(g - start), `gap after scrolling ${dy}: ${g} against ${start}`).toBeLessThanOrEqual(1);
      }
    });
  }
}

for (const overlay of OVERLAYS) {
  test(`${overlay.name}: the script writes transform only, never top, left, bottom, right or position: fixed`, async ({ page }) => {
    await prepare(page, overlay, true);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Emulation.setPageScaleFactor", { pageScaleFactor: 1.5 });
    await page.setViewportSize({ width: 390, height: 520 });
    for (const dy of [0, 90, -140, 300]) {
      await page.evaluate((d) => window.scrollBy(0, d), dy);
      await settle(page);
      const written = await page.locator(`#${overlay.contentId}`).evaluate((el) => {
        const s = (el as HTMLElement).style;
        return {
          position: s.position,
          top: s.top,
          left: s.left,
          bottom: s.bottom,
          right: s.right,
          transform: s.transform,
          computed: getComputedStyle(el).position,
        };
      });
      const note = JSON.stringify({ dy, written });
      expect(written.position, note).toBe("absolute");
      expect(written.computed, note).toBe("absolute");
      expect(["", "0px"], note).toContain(written.top);
      expect(["", "0px"], note).toContain(written.left);
      expect(["", "auto"], note).toContain(written.bottom);
      expect(["", "auto"], note).toContain(written.right);
      expect(written.transform, note).toMatch(/^translate\(/);
    }
  });
}

test("ContextMenu: the menu stays at the document point it opened at while the page scrolls", async ({ page }) => {
  await gotoHydrated(page, FIXTURE, { timeout: NAV_TIMEOUT });
  await page.evaluate(() => {
    document.body.style.paddingBottom = "3000px";
  });
  const trigger = page.locator("#clip-context-menu-trigger");
  await trigger.evaluate((el) => el.scrollIntoView({ block: "center", inline: "center" }));
  const box = await trigger.evaluate((el) => {
    const r = el.getBoundingClientRect();
    return { x: r.left + 12, y: r.top + 10, scrollY: window.scrollY };
  });
  await page.mouse.click(box.x, box.y, { button: "right" });
  const menu = page.locator("#clip-context-menu-content");
  await expect(menu).toBeVisible();
  const docTop = () => readAtRest(menu, (el) => el.getBoundingClientRect().top + window.scrollY);
  const docLeft = () => readAtRest(menu, (el) => el.getBoundingClientRect().left + window.scrollX);
  const top0 = await docTop();
  const left0 = await docLeft();
  // the menu's corner sits at the press point when there is room
  expect(Math.abs(left0 - box.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(top0 - (box.y + box.scrollY))).toBeLessThanOrEqual(1);
  for (const dy of [60, 140, -90]) {
    await page.evaluate((d) => window.scrollBy(0, d), dy);
    await settle(page);
    expect(Math.abs((await docTop()) - top0), `after scrolling ${dy}`).toBeLessThanOrEqual(1);
    expect(Math.abs((await docLeft()) - left0), `after scrolling ${dy}`).toBeLessThanOrEqual(1);
  }
});

test.describe("outside dismiss on touch is a tap", () => {
  for (const overlay of OVERLAYS.slice(0, 2)) {
    test(`${overlay.name}: a touch scroll that starts outside leaves it open, a tap outside closes it`, async ({ page }) => {
      await prepare(page, overlay, false);
      const content = page.locator(`#${overlay.contentId}`);
      const outside = await page.evaluate(() => ({ x: 12, y: window.innerHeight - 24 }));
      const cdp = await page.context().newCDPSession(page);
      await cdp.send("Input.synthesizeScrollGesture", {
        x: outside.x,
        y: outside.y,
        yDistance: -160,
        speed: 600,
        gestureSourceType: "touch",
      });
      await settle(page);
      await expect(content, "a touch scroll that began outside must not close it").toBeVisible();
      // a press that lifts in place is a tap
      await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: outside.x, y: outside.y }] });
      await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      await expect(content, "a tap outside closes it").toBeHidden();
    });
  }
});
