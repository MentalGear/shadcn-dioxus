/**
 * The grids' two items of backlog row 309, on the shared engine:
 *
 *  - Find-in-page placeholders: a windowed VirtualGrid / VirtualWrapGrid with `text_value` keeps
 *    every cell's text in the DOM as a `hidden="until-found"` placeholder (the layer VirtualList
 *    uses, `primitives/src/virtual/find.rs`). The browser can find an unmounted cell; a reveal fires
 *    `beforematch` on the placeholder, which names the cell, and the grid brings the real cell into
 *    view and marks it `data-find-match`. Playwright cannot open the find bar: a dispatched
 *    `beforematch` stands in for the reveal (as in virtual-list-find.spec.ts), and `window.find`
 *    for the search.
 *  - Content-visibility mode: under `VirtualizeMode::Auto` a grid of up to 1,000 cells mounts every
 *    cell (`data-virtualize="content-visibility"`) and keeps the grid pattern's keyboard model.
 *
 * Functional spec: no millisecond thresholds (a debug build is fine).
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { gotoPrerendered } from "./page-kinds";

const FIND_URL = `${BASE_URL}/component/block/?name=virtual_grid&variant=find_in_page`;
const CV_URL = `${BASE_URL}/component/block/?name=virtual_grid&variant=content_visibility`;
const MAIN_URL = `${BASE_URL}/component/block/?name=virtual_grid&variant=main`;

const grid = (page: Page, demo: string) => page.locator(`[data-vg-demo="${demo}"] [data-virtual-grid="grid"]`);
const wrap = (page: Page, demo: string) => page.locator(`[data-vg-demo="${demo}"] [data-virtual-grid="wrap"]`);
const placeholder = (scope: Locator, index: string) => scope.locator(`[data-find-index="${index}"]`);

/** Whether the element `selector` inside the scroller is mounted and fully inside its scrollport. */
function inView(scroller: Locator, selector: string) {
  return scroller.evaluate((el, sel) => {
    const node = el.querySelector(sel);
    if (!node) return false;
    const a = node.getBoundingClientRect();
    const b = el.getBoundingClientRect();
    return a.top >= b.top - 1 && a.bottom <= b.bottom + 1 && a.left >= b.left - 1 && a.right <= b.right + 1;
  }, selector);
}

/** What the browser does on a find-in-page reveal, minus the find bar Playwright cannot open. */
function reveal(scope: Locator, index: string) {
  return placeholder(scope, index).evaluate((el) => {
    el.removeAttribute("hidden");
    el.dispatchEvent(new Event("beforematch", { bubbles: true }));
  });
}

test.describe("virtual grids: find-in-page placeholders (row 309)", () => {
  test("every cell of the windowed data grid is a findable placeholder, outside the accessibility tree", async ({ page }) => {
    await gotoHydrated(page, FIND_URL);
    const g = grid(page, "find-in-page");
    await expect(g).toHaveAttribute("data-virtualize", "windowed");
    await expect(g.locator('[data-grid-cell="0:0"]')).toBeVisible();
    expect(await g.locator('[role="gridcell"], [role="rowheader"]').count()).toBeLessThan(400);
    // One placeholder per cell (2,000 x 8), mounted or not: the layer never changes while the grid scrolls.
    await expect(placeholder(g, "1776:0")).toHaveAttribute("hidden", "until-found");
    await expect(placeholder(g, "1776:0")).toHaveText("Order 01777");
    expect(await g.locator("[data-find-index]").count()).toBe(16_000);
    await expect(g.locator('[data-grid-cell="1776:0"]')).toHaveCount(0);
    expect(await g.ariaSnapshot()).not.toContain("Order 01777");
    // Findable: window.find searches until-found content. Unlike the list's placeholders it does not
    // scroll the grid to a placeholder nested in a row of cells (measured, headless Chromium), and it
    // never fires `beforematch`, so the reveal path is the next test's.
    expect(await page.evaluate(() => (window as unknown as { find: (s: string) => boolean }).find("Order 01777"))).toBe(true);
  });

  test("a beforematch brings the data grid's real cell into view and marks it, on both axes", async ({ page }) => {
    await gotoHydrated(page, FIND_URL);
    const g = grid(page, "find-in-page");
    await expect(g.locator('[data-grid-cell="0:0"]')).toBeVisible();
    // A cell far down and past the inline end, so the grid must move both axes to mount it.
    await reveal(g, "1500:7");
    await expect(placeholder(g, "1500:7")).toHaveAttribute("hidden", "until-found");
    await expect.poll(() => inView(g, '[data-grid-cell="1500:7"]')).toBe(true);
    await expect(g.locator('[data-grid-cell="1500:7"]')).toHaveAttribute("data-find-match", "true");
    await reveal(g, "1520:2");
    await expect.poll(() => inView(g, '[data-grid-cell="1520:2"]')).toBe(true);
    await expect(g.locator("[data-grid-cell][data-find-match]")).toHaveCount(1);
  });

  test("a beforematch brings the gallery's real item into view and marks it", async ({ page }) => {
    await gotoHydrated(page, FIND_URL);
    const w = wrap(page, "find-in-page");
    await expect(w).toHaveAttribute("data-virtualize", "windowed");
    await expect(w.locator('[data-grid-index="0"]')).toBeVisible();
    await expect(placeholder(w, "2467")).toHaveText("Photo 2468");
    expect(await w.locator("[data-find-index]").count()).toBe(3_000);
    await reveal(w, "2467");
    await expect.poll(() => inView(w, '[data-grid-index="2467"]')).toBe(true);
    await expect(w.locator('[data-grid-index="2467"]')).toHaveAttribute("data-find-match", "true");
  });

  test("a browser reveal lands on the real item although the row estimate is off", async ({ page }) => {
    // The demo's `estimate_row_height` (64) is below the real row (72: the item's 56 + 8 margin, plus the
    // grid's own 8 px of row padding). The placeholder sits at the estimate and the browser scrolls to it
    // after `beforematch`; the rows mounted above the item then measure 8 px taller each and push the item
    // below where the browser scrolled: 129 px off at the 422nd grid row, so "Photo 2108" ended half out of the
    // 256 px viewport and the grid showed other photos. A `#:~:text=` fragment is the one reveal path
    // Playwright can drive that includes the browser's own scroll (the dispatched `beforematch` of the
    // tests above does not: it left the item in view).
    //
    // The fragment is applied by a navigation to the finished page, not by the load that opens it: the
    // placeholders mount after hydration, and a fragment carried by the first load is searched only in
    // the browser's own window around the load event, which a loaded machine (a debug wasm build takes
    // seconds to hydrate) overruns, so no `beforematch` ever fires. A same-document navigation to the
    // fragment searches the document as it is (backlog row 416).
    await page.addInitScript(() => {
      (window as unknown as { __fired: string[] }).__fired = [];
      document.addEventListener(
        "beforematch",
        (e) => (window as unknown as { __fired: string[] }).__fired.push((e.target as HTMLElement).dataset.findIndex ?? "?"),
        true,
      );
    });
    await gotoHydrated(page, FIND_URL);
    const w = wrap(page, "find-in-page");
    await expect(placeholder(w, "2107")).toHaveAttribute("hidden", "until-found");
    // The estimate really is off (the test is about that; fix the demo's numbers and it must say so).
    const sizes = await w.evaluate((el) => ({
      real: el.querySelector('[role="row"]')!.getBoundingClientRect().height,
      placeholder: el.querySelector("[data-find-index]")!.getBoundingClientRect().height,
    }));
    expect(sizes.real).toBeGreaterThan(sizes.placeholder);
    await gotoPrerendered(page, `${page.url().split("#")[0]}#:~:text=Photo%202108`);
    // Chromium fires `beforematch` on the placeholder of item 2107.
    await expect.poll(() => page.evaluate(() => (window as unknown as { __fired: string[] }).__fired)).toContain("2107");
    const item = '[data-grid-index="2107"]';
    await expect(w.locator(item)).toHaveAttribute("data-find-match", "true");
    await expect.poll(() => inView(w, item)).toBe(true);
    // It stays: rows still measuring around it do not move it out again. Sixty animation frames, not a
    // wall-clock wait: a loaded machine paints fewer of them in the same time, and the settle counts frames.
    expect(
      await w.evaluate(
        (el, sel) =>
          new Promise<boolean>((resolve) => {
            let frames = 0;
            const check = () => {
              const a = el.querySelector(sel)?.getBoundingClientRect();
              const b = el.getBoundingClientRect();
              if (!a || a.top < b.top - 1 || a.bottom > b.bottom + 1) return resolve(false);
              if (++frames >= 60) return resolve(true);
              requestAnimationFrame(check);
            };
            requestAnimationFrame(check);
          }),
        item,
      ),
    ).toBe(true);
  });

  test("a grid without text_value has no find layer", async ({ page }) => {
    await gotoHydrated(page, MAIN_URL);
    const g = page.locator('[data-virtual-grid="grid"]').first();
    await expect(g).toHaveAttribute("data-virtualize", "windowed");
    await expect(g.locator('[data-grid-cell="0:0"]')).toBeVisible();
    await expect(g.locator("[data-find-layer]")).toHaveCount(0);
  });
});

test.describe("virtual grids: content-visibility mode (row 309)", () => {
  test("under Auto a small data grid mounts every cell and keeps the keyboard model", async ({ page }) => {
    await gotoHydrated(page, CV_URL);
    const g = grid(page, "content-visibility");
    await expect(g).toHaveAttribute("data-virtualize", "content-visibility");
    // 60 rows x 12 columns, all in the DOM (the header row is not counted).
    expect(await g.locator('[role="gridcell"], [role="rowheader"]').count()).toBe(720);
    await expect(g.locator('[data-grid-cell="59:11"]')).toHaveText("R60 C11");
    await g.locator('[data-grid-cell="0:1"]').click();
    await expect(g.locator('[data-grid-cell="0:1"]')).toBeFocused();
    await page.keyboard.press("Control+End");
    await expect(g.locator('[data-grid-cell="59:11"]')).toBeFocused();
    await expect.poll(() => inView(g, '[data-grid-cell="59:11"]')).toBe(true);
    await page.keyboard.press("ArrowUp");
    await expect(g.locator('[data-grid-cell="58:11"]')).toBeFocused();
    await page.keyboard.press("Home");
    await expect(g.locator('[data-grid-cell="58:0"]')).toBeFocused();
    // One tab stop.
    expect(await g.locator('[tabindex="0"]').count()).toBe(1);
  });

  test("under Auto a small gallery mounts every tile and keeps the keyboard model", async ({ page }) => {
    await gotoHydrated(page, CV_URL);
    const w = wrap(page, "content-visibility");
    await expect(w).toHaveAttribute("data-virtualize", "content-visibility");
    expect(await w.locator('[role="gridcell"]').count()).toBe(240);
    await w.locator('[data-grid-index="0"]').click();
    await page.keyboard.press("Control+End");
    await expect(w.locator('[data-grid-index="239"]')).toBeFocused();
    await expect.poll(() => inView(w, '[data-grid-index="239"]')).toBe(true);
    await page.keyboard.press("ArrowUp");
    await expect(w.locator('[data-grid-index="233"]')).toBeFocused();
  });

  test("the server's markup already holds every cell", async ({ page }) => {
    // The component page's prerendered HTML, before any script: content-visibility mode renders on
    // the server.
    const response = await page.request.get(`${BASE_URL}/component/virtual_grid/`);
    const html = await response.text();
    expect(html).toContain('data-virtualize="content-visibility"');
    expect(html).toContain("R60 C11");
    expect(html).toContain("Tile 240");
  });
});
