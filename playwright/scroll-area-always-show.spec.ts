/**
 * `ScrollArea { always_show_scrollbars: true }`: the scroller carries `dx-scroll-area-always-show`, and main.css
 * paints its thumb without hover, so the horizontal demo reads as scrollable at rest. A scroller without the prop
 * (the `main` demo) keeps the app-wide hover-only thumb.
 *
 * Observed through the computed `scrollbar-color` with the pointer parked off both scrollers: a visible thumb is
 * an opaque first colour, a hidden one is fully transparent. Functional spec: no millisecond thresholds.
 */
import { test, expect } from "./fixtures";
import type { Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/scroll_area/`;

/** The alpha of the thumb colour in the computed `scrollbar-color` (`auto` counts as visible). */
function thumbAlpha(el: Locator): Promise<number> {
  return el.evaluate((node) => {
    const value = getComputedStyle(node).scrollbarColor;
    if (value === "auto") return 1;
    const thumb = value.match(/rgba?\([^)]*\)/)?.[0] ?? "";
    const parts = thumb.replace(/rgba?\(|\)/g, "").split(/[\s,/]+/).filter(Boolean);
    return parts.length === 4 ? Number(parts[3]) : 1;
  });
}

test("the horizontal demo shows its scrollbar at rest; the main demo hides it until hover", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  const horizontal = page.locator("#component-preview-frame-horizontal [data-scroll-direction]").first();
  const main = page.locator("#component-preview-frame-main [data-scroll-direction]").first();
  await horizontal.scrollIntoViewIfNeeded();
  await page.mouse.move(0, 0);

  await expect(horizontal).toHaveClass(/dx-scroll-area-always-show/);
  expect(await horizontal.evaluate((n) => n.scrollWidth > n.clientWidth)).toBe(true);
  expect(await thumbAlpha(horizontal)).toBeGreaterThan(0);

  await expect(main).toHaveClass(/dx-scroll-area-auto-hide/);
  expect(await thumbAlpha(main)).toBe(0);
});
