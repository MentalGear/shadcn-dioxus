/**
 * The box a `position: fixed` overlay is actually placed in.
 *
 * Specs that assert "this panel / menu / sheet stays inside the viewport" must compare against THIS,
 * never against `document.documentElement.clientWidth` / `clientHeight`, `window.innerWidth` /
 * `innerHeight` or `page.viewportSize()`. The app sets `html { scrollbar-gutter: stable }`
 * (preview/assets/main.css), so those read the gutter-inclusive size (390 at a 390px phone width)
 * while a fixed overlay's containing block excludes the reserved gutter (375): a panel landing 11-15px
 * past the right edge would still pass. That is how a real overflow in `primitives/src/top_layer.rs` hides
 * (`viewportMetrics()` there uses the same probe).
 *
 * Measured from a temporary `position: fixed; inset: 0` element (hidden, inert, removed again), because
 * that is by definition the rectangle a fixed box resolves `inset` / `%` against. Re-measure after
 * every `setViewportSize()`; nothing is cached.
 *
 * Legitimate uses of the other numbers (page-level horizontal overflow via `scrollWidth - clientWidth`,
 * scroll extents, the gutter itself) stay as they are; `scripts/check-spec-viewport-bound.sh` makes
 * each such use say so.
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
