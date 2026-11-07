/**
 * `ScrollArea { scroll_type }` on the `scroll_type` demo: `Hidden` draws no native scrollbar while the list still
 * scrolls, `Auto` and `Always` keep theirs. `scrollbar-width` is a CSS property, so the observable is the computed
 * style (`scrollbar-width: none`, and no `::-webkit-scrollbar` box where the engine supports it), never an attribute.
 * The same holds for the JS-off first paint. Functional spec: no millisecond thresholds.
 */
import { test, expect } from "./fixtures";
import type { Locator } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const URL = `${BASE_URL}/component/scroll_area/`;

function scroller(page: import("@playwright/test").Page, label: string): Locator {
  return page.locator(`#component-preview-frame-scroll_type [data-scroll-direction][aria-label="${label} folders"]`);
}

test("the Hidden scroller computes scrollbar-width none, has no scrollbar gutter and still scrolls", async ({ page }) => {
  await gotoHydrated(page, URL, GOTO_OPTS);
  for (const label of ["Auto", "Always", "Hidden"]) await scroller(page, label).scrollIntoViewIfNeeded();

  const hidden = scroller(page, "Hidden");
  expect(await hidden.evaluate((n) => getComputedStyle(n).scrollbarWidth)).toBe("none");
  expect(await hidden.evaluate((n) => n.hasAttribute("scrollbar-width"))).toBe(false);
  expect(await hidden.evaluate((n) => getComputedStyle(n, "::-webkit-scrollbar").display)).toBe("none");
  // No gutter: the border box minus the borders equals the client box.
  expect(await hidden.evaluate((n) => n.offsetWidth - n.clientWidth - 2)).toBe(0);
  expect(await hidden.evaluate((n) => n.scrollHeight > n.clientHeight)).toBe(true);
  await hidden.evaluate((n) => n.scrollTo(0, 40));
  expect(await hidden.evaluate((n) => n.scrollTop)).toBeGreaterThan(0);

  for (const label of ["Auto", "Always"]) {
    const other = scroller(page, label);
    expect(await other.evaluate((n) => getComputedStyle(n).scrollbarWidth)).not.toBe("none");
    await expect(other).not.toHaveAttribute("data-scrollbar-hidden");
  }
});
