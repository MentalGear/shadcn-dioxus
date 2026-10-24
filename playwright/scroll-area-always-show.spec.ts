/**
 * `ScrollArea { always_show_scrollbars: true }`: the scroller carries `dx-scroll-area-always-show`, and main.css
 * paints its thumb without hover, so the horizontal demo reads as scrollable at rest. A scroller without the prop
 * (the `main` demo) keeps the app-wide hover-only thumb.
 *
 * Observed through the computed `scrollbar-color` with the pointer parked off both scrollers: a visible thumb has
 * a painted first colour, a hidden one is fully transparent. Functional spec: no millisecond thresholds.
 */
import { test, expect } from "./fixtures";
import type { Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/scroll_area/`;

/** Whether the thumb colour (the first of the computed `scrollbar-color` pair) is painted: `auto` or any colour that
 *  is not fully transparent. The colour serializes in whatever space the theme token uses (`lab(...)`, `rgb(...)`). */
function thumbPainted(el: Locator): Promise<boolean> {
  return el.evaluate((node) => {
    const value = getComputedStyle(node).scrollbarColor.trim();
    if (value === "auto") return true;
    const thumb = value.match(/^[a-z-]+\([^)]*\)|^\S+/)?.[0] ?? "";
    const probe = document.createElement("canvas").getContext("2d") as CanvasRenderingContext2D;
    probe.fillStyle = thumb;
    probe.fillRect(0, 0, 1, 1);
    return probe.getImageData(0, 0, 1, 1).data[3] > 0;
  });
}

test("the horizontal demo shows its scrollbar at rest; the main demo hides it until hover", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  const horizontal = page.locator("#component-preview-frame-horizontal [data-scroll-direction]").first();
  const main = page.locator("#component-preview-frame [data-scroll-direction]").first();
  await horizontal.scrollIntoViewIfNeeded();
  await page.mouse.move(0, 0);

  await expect(horizontal).toHaveClass(/dx-scroll-area-always-show/);
  expect(await horizontal.evaluate((n) => n.scrollWidth > n.clientWidth)).toBe(true);
  expect(await thumbPainted(horizontal)).toBe(true);

  await expect(main).toHaveClass(/dx-scroll-area-auto-hide/);
  expect(await thumbPainted(main)).toBe(false);
});
