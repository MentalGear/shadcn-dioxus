/**
 * The shared windowing engine (backlog row 309): VirtualList and VirtualGrid consume one DOM
 * bridge and one "keep the focused item rendered" strategy, and each gained the other's strength.
 *
 *  - VirtualGrid / VirtualWrapGrid take the `fade: ScrollFade` prop VirtualList and ScrollArea
 *    have: the `dx-scroll-fade-*` mask lands on the grid's scroll container and follows its scroll
 *    offset whatever cells are mounted (the grid's `fade` demo).
 *  - A windowed VirtualList keeps the row that last received focus mounted when the window leaves
 *    it (as the grids keep their active cell): the same DOM node, drawn at its own position, and
 *    focus never falls to <body>.
 *
 * Functional spec: no millisecond thresholds (a debug build is fine). Nothing here bounds an overlay.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame } from "./demo-frame";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const GRID_URL = `${BASE_URL}/component/virtual_grid/`;
const LIST_URL = `${BASE_URL}/component/block/?name=virtual_list&variant=windowed`;

type Edge = "crisp" | "faded";

/** The block-axis fade depths in px at the scroller's current position, and the full depth. */
function readFade(el: Locator) {
  return el.evaluate((node) => {
    const cs = getComputedStyle(node);
    const extent = node.getBoundingClientRect().height;
    const toPx = (value: string) => {
      const outer = document.createElement("div");
      outer.style.cssText = `position:absolute;visibility:hidden;pointer-events:none;height:${extent}px;`;
      const inner = document.createElement("div");
      inner.style.height = value.trim() || "0px";
      outer.appendChild(inner);
      document.body.appendChild(outer);
      const px = inner.getBoundingClientRect().height;
      outer.remove();
      return px;
    };
    return {
      start: toPx(cs.getPropertyValue("--dx-scroll-fade-t")),
      end: toPx(cs.getPropertyValue("--dx-scroll-fade-b")),
      full: Math.min(extent * 0.12, 40),
      mask: cs.maskImage || cs.webkitMaskImage || "",
    };
  });
}

async function expectEdges(el: Locator, want: { start?: Edge; end: Edge }) {
  await expect
    .poll(
      async () => {
        const f = await readFade(el);
        const kind = (px: number) => (px < 1 ? "crisp" : Math.abs(px - f.full) <= 1.5 ? "faded" : `partial(${px.toFixed(1)}/${f.full.toFixed(1)})`);
        return want.start === undefined ? kind(f.end) : `${kind(f.start)}/${kind(f.end)}`;
      },
      { timeout: 10_000, message: "block-axis fade edges" },
    )
    .toBe(want.start === undefined ? want.end : `${want.start}/${want.end}`);
}

const scrollBlockTo = (el: Locator, where: "start" | "middle" | "end") =>
  el.evaluate((n, where) => {
    const max = n.scrollHeight - n.clientHeight;
    n.scrollTop = where === "start" ? 0 : where === "middle" ? max / 2 : max + 10_000;
  }, where);

async function settleAtEnd(el: Locator) {
  await expect
    .poll(async () => {
      await scrollBlockTo(el, "end");
      return el.evaluate((n) => n.scrollHeight - n.clientHeight - Math.round(n.scrollTop) <= 2);
    }, { timeout: 15_000 })
    .toBe(true);
}

const gridIn = (page: Page, demo: string) => demoFrame(page, "fade").locator(`[data-demo="${demo}"] [role="grid"]`);

test.describe("VirtualGrid fade (shared with VirtualList and ScrollArea)", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, GRID_URL, GOTO_OPTS);
    await demoFrame(page, "fade").scrollIntoViewIfNeeded();
  });

  test("wrap grid: the fade follows the scroll position over 500 windowed rows", async ({ page }) => {
    const grid = gridIn(page, "fade-wrap");
    await expect(grid).toHaveAttribute("data-scroll-fade", "y");
    await expect(grid).toHaveClass(/dx-scroll-fade-y/);
    await expect(grid).toHaveClass(/dx-virtual-grid/); // merged, not replaced
    await expect(grid.locator('[role="gridcell"]').first()).toBeVisible({ timeout: 60_000 });
    expect(await grid.locator('[role="row"]').count()).toBeLessThan(40);

    await scrollBlockTo(grid, "start");
    await expectEdges(grid, { start: "crisp", end: "faded" });

    await scrollBlockTo(grid, "middle");
    await expectEdges(grid, { start: "faded", end: "faded" });
    // The window really moved, and stayed small.
    await expect
      .poll(() => grid.locator('[role="row"]').last().getAttribute("aria-rowindex").then(Number), { timeout: 10_000 })
      .toBeGreaterThan(100);
    expect(await grid.locator('[role="row"]').count()).toBeLessThan(40);

    await settleAtEnd(grid);
    await expectEdges(grid, { start: "faded", end: "crisp" });
    await expect(grid.locator('[data-grid-index="1999"]')).toBeVisible();
  });

  test("data grid: a bottom-only fade leaves the sticky header crisp and sharpens at the end", async ({ page }) => {
    const grid = gridIn(page, "fade-grid");
    await expect(grid).toHaveAttribute("data-scroll-fade", "bottom");
    await expect(grid).toHaveClass(/dx-scroll-fade-b/);
    await expect(grid.locator('[data-grid-cell="0:0"]')).toBeVisible({ timeout: 60_000 });
    expect((await readFade(grid)).mask).toContain("linear-gradient");

    await scrollBlockTo(grid, "middle");
    await expectEdges(grid, { end: "faded" });
    await expect.poll(() => readFade(grid).then((f) => f.start), { timeout: 10_000 }).toBeLessThan(1);
    // Rows and columns are still windowed under the mask.
    expect(await grid.locator("[data-grid-cell]").count()).toBeLessThan(600);

    await settleAtEnd(grid);
    await expectEdges(grid, { end: "crisp" });
  });
});

test.describe("VirtualList keeps its focused row rendered (shared with the grids' active cell)", () => {
  test("a focused row survives the window leaving it, as the same node, and rejoins the flow", async ({ page }) => {
    await gotoHydrated(page, LIST_URL, GOTO_OPTS);
    const list = page.locator('[data-vl-demo="windowed"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "windowed");
    const row = list.locator('[data-virtual-index="3"]');
    await expect(row).toBeVisible({ timeout: 60_000 });

    // Focus the row (as a link or button inside it would be) and mark the node.
    await row.evaluate((el) => {
      (el as HTMLElement).tabIndex = -1;
      (el as HTMLElement).focus();
      (el as unknown as { __probe: number }).__probe = 309;
    });
    const before = await row.evaluate((el) => el.getBoundingClientRect().top - el.closest('[role="list"]')!.firstElementChild!.getBoundingClientRect().top);

    // Scroll far away: the window moves past row 1,000 ...
    await list.evaluate((el) => (el.scrollTop = 500_000));
    await expect
      .poll(() => list.locator('[role="listitem"]:not([data-virtual-pinned])').first().getAttribute("data-virtual-index").then(Number), { timeout: 15_000 })
      .toBeGreaterThan(1000);
    // ... and row 3 is still there: pinned, the same node, still focused, at its own position.
    await expect(row).toHaveAttribute("data-virtual-pinned", "true");
    expect(await row.evaluate((el) => (el as unknown as { __probe?: number }).__probe)).toBe(309);
    expect(await row.evaluate((el) => document.activeElement === el)).toBe(true);
    const pinnedAt = await row.evaluate((el) => el.getBoundingClientRect().top - el.closest('[role="list"]')!.firstElementChild!.getBoundingClientRect().top);
    expect(Math.abs(pinnedAt - before)).toBeLessThan(300);
    // One extra row, not a growing DOM.
    expect(await list.getByRole("listitem").count()).toBeLessThan(80);

    // Back to the top: the row rejoins the flow without remounting, and keeps focus.
    await list.evaluate((el) => (el.scrollTop = 0));
    await expect(row).not.toHaveAttribute("data-virtual-pinned", /.*/, { timeout: 15_000 });
    expect(await row.evaluate((el) => (el as unknown as { __probe?: number }).__probe)).toBe(309);
    expect(await row.evaluate((el) => document.activeElement === el)).toBe(true);
    expect(await row.evaluate((el) => getComputedStyle(el).position)).toBe("static");
  });
});
