/**
 * The box a `position: fixed` overlay is actually placed in.
 *
 * Specs that assert "this panel / menu / sheet stays inside the viewport" must compare against THIS,
 * never against `document.documentElement.clientWidth` / `clientHeight`, `window.innerWidth` /
 * `innerHeight` or `page.viewportSize()`. The app sets `html { scrollbar-gutter: stable }`
 * (preview/assets/main.css), so those read the gutter-inclusive size (390 at a 390px phone width)
 * while a fixed overlay's containing block excludes the reserved gutter (375): a panel landing 11-15px
 * past the right edge would still pass. That is how a real overflow in `primitives/src/top_layer/scroll_clamp.rs` hides
 * (`visible()` in `primitives/src/top_layer/scroll_clamp.rs` uses the same probe).
 *
 * Measured from a temporary `position: fixed; inset: 0` element (hidden, inert, removed again), because
 * that is by definition the rectangle a fixed box resolves `inset` / `%` against. Re-measure after
 * every `setViewportSize()`; nothing is cached.
 *
 * The other numbers are not an overlay bound but are still real measurements (page-level horizontal overflow, a
 * scroll extent, the gutter itself, the size a test restores, a coordinate domain for `elementFromPoint`): they are
 * read through `pageMetrics(page)` below, which names each one, so a spec says which it means and the gate
 * (`scripts/check-spec-viewport-bound.sh`) accepts only `fixedViewportBox` and `pageMetrics`.
 */
import type { Page } from "@playwright/test";

export type ViewportBox = { x: number; y: number; width: number; height: number };

export async function fixedViewportBox(page: Page): Promise<ViewportBox> {
  return page.evaluate(() => {
    const probe = document.createElement("div");
    probe.style.cssText = "position:fixed;inset:0;visibility:hidden;pointer-events:none;";
    document.documentElement.appendChild(probe);
    const b = probe.getBoundingClientRect();
    probe.remove();
    return { x: b.x, y: b.y, width: b.width, height: b.height };
  });
}

/**
 * The page's own numbers, each named for what it measures (backlog row 427). None of them bounds a `position: fixed`
 * overlay (that is `fixedViewportBox`): they include or sit beside the `scrollbar-gutter: stable` strip.
 *
 *   `innerWidth`, `innerHeight`   the layout viewport including the scrollbar gutter (what `elementFromPoint` takes)
 *   `clientWidth`, `clientHeight` the root element's own client box (the gutter excluded)
 *   `scrollWidth`, `scrollHeight`, `scrollX`, `scrollY`   the root element's scroll size and position
 *   `scrollExtent`      `scrollHeight - innerHeight`: how far the page can scroll down
 *   `sidewaysOverflow`  `scrollWidth - clientWidth`: how far the page overflows sideways (<= 0: it does not)
 *   `gutter`            `innerWidth - clientWidth`: the scrollbar strip
 *   `viewportSize`      the size Playwright set for the page (`page.viewportSize()`), or null for a null viewport
 *
 * One `evaluate`, nothing cached: read again after a `setViewportSize()`, a scroll or a layout change. Code that runs
 * inside `page.evaluate` takes the numbers it needs as arguments (`page.evaluate((vh) => ..., m.innerHeight)`).
 */
export type PageMetrics = {
  innerWidth: number;
  innerHeight: number;
  clientWidth: number;
  clientHeight: number;
  scrollWidth: number;
  scrollHeight: number;
  scrollX: number;
  scrollY: number;
  scrollExtent: number;
  sidewaysOverflow: number;
  gutter: number;
  viewportSize: { width: number; height: number } | null;
};

export async function pageMetrics(page: Page): Promise<PageMetrics> {
  const m = await page.evaluate(() => {
    const de = document.documentElement;
    return {
      innerWidth: window.innerWidth,
      innerHeight: window.innerHeight,
      clientWidth: de.clientWidth,
      clientHeight: de.clientHeight,
      scrollWidth: de.scrollWidth,
      scrollHeight: (document.scrollingElement ?? de).scrollHeight,
      scrollX: window.scrollX,
      scrollY: window.scrollY,
    };
  });
  return {
    ...m,
    scrollExtent: m.scrollHeight - m.innerHeight,
    sidewaysOverflow: m.scrollWidth - m.clientWidth,
    gutter: m.innerWidth - m.clientWidth,
    viewportSize: page.viewportSize(),
  };
}
