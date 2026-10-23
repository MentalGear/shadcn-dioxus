/**
 * A scroll fade never covers its scrollbar. The `dx-scroll-fade` utility of `assets/dx-effects.css` is a
 * `mask-image` on the scroll container, and a mask covers the whole border box, scrollbars included, so a
 * classic scrollbar used to dissolve with the content at every faded edge (most visible with
 * `ScrollFade::Both`, where all four ends of both scrollbars faded). The mask is now a stack of layers
 * whose content area is the padding box minus the scrollbar gutters, which the scroller's script writes
 * as `--dx-scrollbar-inline-size` / `--dx-scrollbar-block-size`; the gutters stay opaque.
 *
 * What is observed: PIXELS. Headless Chromium hides scrollbars by default (`--hide-scrollbars`), so this
 * file launches its own browser with classic ones, and the site paints scrollbars transparent until
 * hover (`main.css`), so the spec gives the scroller an opaque `scrollbar-color` (a red thumb on a blue
 * track) and screenshots it. At a scroll position where both edges of an axis are faded, the track is
 * sampled in the middle of the gutter right at the faded end and again at the middle of the track; with
 * the mask in the way the first pixel is a blend towards the card behind it, with the fix it is the
 * track's own colour. The same is read for the horizontal scrollbar of a `Both` grid, and in RTL, where
 * the classic scrollbar moves to the left. A control reads the content area at the same edge and proves
 * the fade really is on (the content pixel differs from its interior value), so a pass cannot mean "the
 * fade was off".
 *
 * Overlay scrollbars (macOS, iOS) are 0px wide: the gutters measure 0, the content area is the whole box
 * and the mask is the plain gradient it was, so there is nothing to sample there.
 *
 * Functional spec: no millisecond thresholds. Nothing here bounds an overlay in the viewport.
 */
import fs from "node:fs";
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Classic scrollbars for this file's browser. `launchOptions` of `test.use` REPLACE the config's (they do not
// merge), so the sandbox's own Chromium path that `ssg.local.config.ts` sets is repeated here, under the same
// exists-check; a CI runner without it uses Playwright's default browser.
const SANDBOX_CHROMIUM = "/opt/pw-browsers/chromium-1194/chrome-linux/chrome";
test.use({
  launchOptions: {
    ...(fs.existsSync(SANDBOX_CHROMIUM) ? { executablePath: SANDBOX_CHROMIUM } : {}),
    ignoreDefaultArgs: ["--hide-scrollbars"],
  },
});

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const url = (name: string) => `${BASE_URL}/component/${name}/`;
const frame = (page: Page) => page.locator("#component-preview-frame-fade").first();

type Rgba = [number, number, number, number];

/** Make the scrollbar opaque and recognisable: the site hides it until hover. */
const paintScrollbar = (el: Locator) =>
  el.evaluate((n) => ((n as HTMLElement).style.scrollbarColor = "rgb(255, 0, 0) rgb(0, 0, 255)"));

/** Scroll to the middle of both ranges (negative inline offset in RTL), so every edge is faded. */
const scrollToMiddle = (el: Locator) =>
  el.evaluate((n) => {
    const sign = getComputedStyle(n).direction === "rtl" ? -1 : 1;
    n.scrollTop = (n.scrollHeight - n.clientHeight) / 2;
    n.scrollLeft = (sign * (n.scrollWidth - n.clientWidth)) / 2;
  });

/** The gutters the browser reserved (px), the script's measurement of them, and where they are on screen. */
const geometry = (el: Locator) =>
  el.evaluate((n) => {
    const cs = getComputedStyle(n);
    const px = (v: string) => parseFloat(v) || 0;
    const r = n.getBoundingClientRect();
    const bl = px(cs.borderLeftWidth);
    const br = px(cs.borderRightWidth);
    const bt = px(cs.borderTopWidth);
    const bb = px(cs.borderBottomWidth);
    const w = n.offsetWidth - n.clientWidth - bl - br;
    const h = n.offsetHeight - n.clientHeight - bt - bb;
    const rtl = cs.direction === "rtl";
    // The edge depths are `<length-percentage>`s that the scroll-driven animation leaves as a `calc()`;
    // resolve one to px with a probe inside a box of the scroller's extent (percentages are against it).
    const toPx = (name: string, extent: number, prop: "width" | "height") => {
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;pointer-events:none;${prop}:${extent}px;`;
      const inner = document.createElement("div");
      inner.style[prop] = cs.getPropertyValue(name).trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect()[prop];
      outer.remove();
      return px;
    };
    return {
      w,
      h,
      rtl,
      measuredW: px(n.style.getPropertyValue("--dx-scrollbar-inline-size")),
      measuredH: px(n.style.getPropertyValue("--dx-scrollbar-block-size")),
      // Centre line of each gutter, in viewport coordinates.
      vx: rtl ? r.left + bl + w / 2 : r.right - br - w / 2,
      hy: r.bottom - bb - h / 2,
      // The content area (padding box less the gutters), for the control reads.
      left: r.left + bl + (rtl ? w : 0),
      right: r.right - br - (rtl ? 0 : w),
      top: r.top + bt,
      bottom: r.bottom - bb - h,
      fadeT: toPx("--dx-scroll-fade-t", r.height, "height"),
      fadeB: toPx("--dx-scroll-fade-b", r.height, "height"),
      fadeS: toPx("--dx-scroll-fade-s", r.width, "width"),
      fadeE: toPx("--dx-scroll-fade-e", r.width, "width"),
    };
  });

/** Screenshot the page and read the pixels at the given viewport points. */
async function pixels(page: Page, points: [number, number][]): Promise<Rgba[]> {
  const png = (await page.screenshot()).toString("base64");
  return page.evaluate(
    async ([png, points]) => {
      const bmp = await createImageBitmap(await (await fetch(`data:image/png;base64,${png}`)).blob());
      const canvas = new OffscreenCanvas(bmp.width, bmp.height);
      const ctx = canvas.getContext("2d")!;
      ctx.drawImage(bmp, 0, 0);
      const scale = window.devicePixelRatio; // the screenshot is in device pixels
      return points.map(([x, y]) => Array.from(ctx.getImageData(Math.round(x * scale), Math.round(y * scale), 1, 1).data) as Rgba);
    },
    [png, points] as const,
  );
}

const TRACK: Rgba = [0, 0, 255, 255];
const TOLERANCE = 24;

/** The largest channel difference between two pixels. */
const diff = (a: Rgba, b: Rgba) => Math.max(...a.map((v, i) => Math.abs(v - b[i])));

/**
 * Bring the scroller on screen, give it an opaque scrollbar, park it mid-range and wait for the
 * scroll-driven fade to settle on both edges of the axes in `axes`.
 */
async function prepare(page: Page, el: Locator, axes: ("block" | "inline")[]) {
  await el.scrollIntoViewIfNeeded();
  await paintScrollbar(el);
  await scrollToMiddle(el);
  await expect
    .poll(
      async () => {
        const g = await geometry(el);
        const block = axes.includes("block") ? g.fadeT > 4 && g.fadeB > 4 : true;
        const inline = axes.includes("inline") ? g.fadeS > 4 && g.fadeE > 4 : true;
        return block && inline && g.w > 0;
      },
      { timeout: 15_000, message: "fade edges settled and a classic scrollbar present" },
    )
    .toBe(true);
}

/** Vertical scrollbar: the track in the middle of the gutter, 3px inside each end. */
async function expectVerticalBarOpaque(page: Page, el: Locator) {
  const g = await geometry(el);
  expect(g.w, "a classic vertical scrollbar takes space").toBeGreaterThan(8);
  const [atTop, atBottom] = await pixels(page, [
    [g.vx, g.top + 3],
    [g.vx, g.bottom - 3],
  ]);
  // The track is the pure blue of `scrollbar-color`; the thumb (red) sits mid-range, away from both ends. A
  // masked pixel is a blend towards whatever is behind the scroller (white or dark), never pure blue.
  expect(diff(atTop, TRACK), `track at the top end (${atTop})`).toBeLessThan(TOLERANCE);
  expect(diff(atBottom, TRACK), `track at the bottom end (${atBottom})`).toBeLessThan(TOLERANCE);
}

/** Horizontal scrollbar: the track in the middle of the gutter, 3px inside each end. */
async function expectHorizontalBarOpaque(page: Page, el: Locator) {
  const g = await geometry(el);
  expect(g.h, "a classic horizontal scrollbar takes space").toBeGreaterThan(8);
  const [atStart, atEnd] = await pixels(page, [
    [g.left + 3, g.hy],
    [g.right - 3, g.hy],
  ]);
  expect(diff(atStart, TRACK), `track at the left end (${atStart})`).toBeLessThan(TOLERANCE);
  expect(diff(atEnd, TRACK), `track at the right end (${atEnd})`).toBeLessThan(TOLERANCE);
}

/** Control: the fade is on (a gradient mask, edges past the 4px mark), so a pass is not "no fade". */
async function expectFadeOn(el: Locator) {
  const g = await geometry(el);
  expect(g.fadeT).toBeGreaterThan(4);
  expect(g.fadeB).toBeGreaterThan(4);
  expect(await el.evaluate((n) => getComputedStyle(n).maskImage)).toContain("linear-gradient");
}

test.describe("scroll fade leaves the scrollbar opaque", () => {
  test("VirtualGrid, ScrollFade::Both: both scrollbars are opaque at all four faded ends", async ({ page }) => {
    await gotoHydrated(page, url("virtual_grid"), GOTO_OPTS);
    const grid = frame(page).locator('[data-demo="fade-grid"] [role="grid"]');
    await expect(grid).toHaveClass(/dx-scroll-fade-xy/);
    await prepare(page, grid, ["block", "inline"]);
    await expectFadeOn(grid);
    await expectVerticalBarOpaque(page, grid);
    await expectHorizontalBarOpaque(page, grid);
  });

  test("VirtualGrid in RTL: the scrollbar moved to the left and is still opaque", async ({ page }) => {
    await gotoHydrated(page, url("virtual_grid"), GOTO_OPTS);
    await page.locator("#dx-direction-toggle").click();
    await expect(frame(page)).toHaveAttribute("dir", "rtl");
    const grid = frame(page).locator('[data-demo="fade-grid"] [role="grid"]');
    await prepare(page, grid, ["block", "inline"]);
    const g = await geometry(grid);
    expect(g.rtl).toBe(true);
    expect(g.vx, "the vertical scrollbar is on the left in RTL").toBeLessThan((g.left + g.right) / 2);
    await expectFadeOn(grid);
    await expectVerticalBarOpaque(page, grid);
    await expectHorizontalBarOpaque(page, grid);
  });

  test("VirtualList, ScrollFade::Y: the vertical scrollbar is opaque at both faded ends", async ({ page }) => {
    await gotoHydrated(page, url("virtual_list"), GOTO_OPTS);
    const list = frame(page).locator('[data-demo="fade-windowed"] [role="list"]');
    await expect(list.locator('[role="listitem"]').first()).toBeVisible({ timeout: 60_000 });
    await prepare(page, list, ["block"]);
    await expectFadeOn(list);
    await expectVerticalBarOpaque(page, list);
  });

  test("ScrollArea, ScrollFade::Y: the vertical scrollbar is opaque at both faded ends", async ({ page }) => {
    await gotoHydrated(page, url("scroll_area"), GOTO_OPTS);
    const area = frame(page).locator('[data-demo="fade-y"] [data-scroll-fade="y"]');
    await prepare(page, area, ["block"]);
    await expectFadeOn(area);
    await expectVerticalBarOpaque(page, area);
  });

  test("the script keeps the gutter sizes current when a scrollbar appears and goes", async ({ page }) => {
    await gotoHydrated(page, url("virtual_list"), GOTO_OPTS);
    const list = frame(page).locator('[data-demo="fade-windowed"] [role="list"]');
    await expect(list.locator('[role="listitem"]').first()).toBeVisible({ timeout: 60_000 });
    await list.scrollIntoViewIfNeeded();
    const size = () => list.evaluate((n) => (n as HTMLElement).style.getPropertyValue("--dx-scrollbar-inline-size"));
    await expect.poll(size).not.toBe("0px");
    await expect.poll(size).not.toBe("");
    const g = await geometry(list);
    expect(g.measuredW, "the script's measurement is the gutter the browser reserved").toBeCloseTo(g.w, 0);
    // `scrollbar-width: none` takes the gutter away; the property follows.
    await list.evaluate((n) => ((n as HTMLElement).style.scrollbarWidth = "none"));
    await expect.poll(size).toBe("0px");
    await list.evaluate((n) => ((n as HTMLElement).style.scrollbarWidth = ""));
    await expect.poll(size).not.toBe("0px");
  });

  test(".dx-scroll-fade-none still turns the whole mask off", async ({ page }) => {
    await gotoHydrated(page, url("virtual_list"), GOTO_OPTS);
    const list = frame(page).locator('[data-demo="fade-windowed"] [role="list"]');
    await expect(list.locator('[role="listitem"]').first()).toBeVisible({ timeout: 60_000 });
    await list.evaluate((n) => n.classList.add("dx-scroll-fade-none"));
    expect(await list.evaluate((n) => getComputedStyle(n).maskImage)).toBe("none");
  });
});
