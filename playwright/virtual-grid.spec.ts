import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { demoFrame } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// VirtualGrid (backlog row 244): the `main` demo is a 10,000-row x 50-column data grid with a sticky
// header row and a sticky first column; the `gallery` demo is a VirtualWrapGrid of 5,000 generated
// images in rows of a selectable column count. Both window their rows; the data grid also windows
// its columns. Controls are located by role and position inside the demo frames (row 220).

const PAGE_URL = `${BASE_URL}/component/?name=virtual_grid&`;

const dataGrid = (page: Page) => demoFrame(page).getByRole("grid");
const gallery = (page: Page) => demoFrame(page, "gallery").getByRole("grid");
const cell = (grid: Locator, row: number | "header", column: number) =>
  grid.locator(`[data-grid-cell="${row}:${column}"]`);
const mountedCells = (grid: Locator) => grid.locator("[data-grid-cell]").count();
const focusedCell = (page: Page) =>
  page.evaluate(() => document.activeElement?.getAttribute("data-grid-cell") ?? null);

/** Scroll a grid container to `top` and `left` (a distance from the inline start, so negative in RTL). */
async function scrollGrid(grid: Locator, top: number, left: number) {
  await grid.evaluate(
    (el, [t, l]) => {
      el.scrollTop = t;
      el.scrollLeft = l;
    },
    [top, left] as const,
  );
}

/** The cell's box and the grid's visible body (below the sticky header, beside the sticky column). */
async function placement(grid: Locator, row: number, column: number) {
  return grid.evaluate(
    (el, key) => {
      const target = el.querySelector(`[data-grid-cell="${key}"]`)!.getBoundingClientRect();
      const box = el.getBoundingClientRect();
      const header = el.querySelector("[data-grid-header]")?.getBoundingClientRect();
      return {
        top: target.top,
        bottom: target.bottom,
        left: target.left,
        right: target.right,
        bodyTop: header ? header.bottom : box.top + el.clientTop,
        bodyBottom: box.top + el.clientTop + el.clientHeight,
        gridLeft: box.left + el.clientLeft,
        gridRight: box.left + el.clientLeft + el.clientWidth,
      };
    },
    `${row}:${column}`,
  );
}

/**
 * Whether the cell is fully inside the grid's scrollport, clear of the sticky header (the grid may itself
 * be partly below the page fold, so this is not `toBeInViewport`).
 */
async function inGridView(grid: Locator, row: number, column: number) {
  const p = await placement(grid, row, column);
  return (
    p.top >= p.bodyTop - 1 &&
    p.bottom <= p.bodyBottom + 1 &&
    p.left >= p.gridLeft - 1 &&
    p.right <= p.gridRight + 1
  );
}

/**
 * The grid's own sheet reached the page: the scroll container IS a scroll container. Without `style.css`
 * the grid is `overflow: visible`, nothing scrolls and the windowing never advances (the 2026-10 preview:
 * a release build embeds a component's CSS source instead of linking it, so only a `document::Link` in the
 * wrapper ships it -- a debug build masks the omission, so `check-component-catalog.sh` rule 2b guards the
 * source and this guards a release build the spec is run against).
 */
async function expectStyled(grid: Locator) {
  const style = await grid.evaluate((el) => {
    const s = getComputedStyle(el);
    return { overflowY: s.overflowY, position: s.position };
  });
  expect(style).toEqual({ overflowY: "auto", position: "relative" });
}

test.describe("data grid (rows and columns windowed)", () => {
  test("mounts a few hundred of 500,000 cells and announces the whole grid", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = dataGrid(page);
    await expectStyled(grid);
    await expect(grid).toHaveAttribute("aria-rowcount", "10001"); // + the header row
    await expect(grid).toHaveAttribute("aria-colcount", "50");
    await expect(cell(grid, 0, 1)).toBeVisible();
    const mounted = await mountedCells(grid);
    expect(mounted).toBeGreaterThan(50);
    expect(mounted).toBeLessThan(1000);
    // Windowed on both axes: the far corner is not in the DOM until it is scrolled to.
    await expect(cell(grid, 9999, 49)).toHaveCount(0);
    await expect(cell(grid, 0, 49)).toHaveCount(0);
    // Every mounted row and cell carries its position in the whole grid.
    await expect(grid.locator('[role="row"][aria-rowindex="1"]')).toHaveAttribute("data-grid-header", "true");
    await expect(grid.locator('[role="row"][aria-rowindex="2"]')).toHaveCount(1);
    await expect(cell(grid, 0, 3)).toHaveAttribute("aria-colindex", "4");
    await expect(cell(grid, 0, 0)).toHaveAttribute("role", "rowheader");
    await expect(cell(grid, "header", 2)).toHaveAttribute("role", "columnheader");
  });

  test("a scroll to the far corner renders it and keeps the DOM small", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = dataGrid(page);
    await expect(cell(grid, 0, 1)).toBeVisible();
    await scrollGrid(grid, 1e9, 1e9);
    await expect(cell(grid, 9999, 49)).toBeVisible();
    await expect(grid.locator('[role="row"][aria-rowindex="10001"]')).toHaveCount(1);
    // The sticky header and column are still there, at the far corner.
    await expect(cell(grid, "header", 49)).toBeVisible();
    await expect(cell(grid, 9999, 0)).toBeVisible();
    expect(await mountedCells(grid)).toBeLessThan(1000);
    await expect(cell(grid, 0, 1)).toHaveCount(0);
    // And from the middle.
    await scrollGrid(grid, 5000 * 36, 25 * 112);
    await expect(cell(grid, 5000, 26)).toBeVisible();
    expect(await mountedCells(grid)).toBeLessThan(1000);
  });

  test("the keyboard moves one tab stop through the grid and the focused cell stays rendered", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = dataGrid(page);
    await cell(grid, 0, 1).click();
    expect(await focusedCell(page)).toBe("0:1");
    // Exactly one cell is in the tab order.
    await expect(grid.locator('[data-grid-cell][tabindex="0"]')).toHaveCount(1);
    await expect(cell(grid, 0, 1)).toHaveAttribute("tabindex", "0");

    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowDown");
    await expect.poll(() => focusedCell(page)).toBe("1:2");
    await page.keyboard.press("Home");
    await expect.poll(() => focusedCell(page)).toBe("1:0");
    await page.keyboard.press("End");
    await expect.poll(() => focusedCell(page)).toBe("1:49");
    await page.keyboard.press("PageDown");
    const afterPage = (await focusedCell(page))!;
    const [pagedRow] = afterPage.split(":").map(Number);
    expect(pagedRow).toBeGreaterThan(5);
    expect(afterPage.endsWith(":49")).toBe(true);

    // Ctrl+End: the last cell, far outside the window, is rendered, focused and scrolled into view
    // below the sticky header and beside the sticky column.
    await page.keyboard.press("Control+End");
    await expect.poll(() => focusedCell(page)).toBe("9999:49");
    await expect.poll(() => inGridView(grid, 9999, 49)).toBe(true);
    expect(await mountedCells(grid)).toBeLessThan(1000);

    // Scroll back to the top: the focused cell is far outside the window, and still in the DOM
    // with focus, so Tab and the arrows keep working from it.
    await scrollGrid(grid, 0, 0);
    await expect(cell(grid, 0, 1)).toBeVisible();
    await expect(cell(grid, 9999, 49)).toHaveCount(1);
    expect(await focusedCell(page)).toBe("9999:49");
    await page.keyboard.press("ArrowUp");
    await expect.poll(() => focusedCell(page)).toBe("9998:49");
    await expect.poll(() => inGridView(grid, 9998, 49)).toBe(true);

    await page.keyboard.press("Control+Home");
    await expect.poll(() => focusedCell(page)).toBe("0:0");
    await expect.poll(() => inGridView(grid, 0, 0)).toBe(true);
    // At the edges the keys stop.
    await page.keyboard.press("ArrowUp");
    await page.keyboard.press("ArrowLeft");
    expect(await focusedCell(page)).toBe("0:0");
  });

  test("a column scrolled under the sticky column is revealed beside it", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = dataGrid(page);
    await cell(grid, 0, 1).click();
    await page.keyboard.press("End");
    await expect.poll(() => focusedCell(page)).toBe("0:49");
    // Walk back to column 1: it ends up just right of the sticky row-number column, not under it.
    await page.keyboard.press("Home");
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focusedCell(page)).toBe("0:1");
    const sticky = await cell(grid, 0, 0).evaluate((el) => el.getBoundingClientRect().right);
    const p = await placement(grid, 0, 1);
    expect(p.left).toBeGreaterThanOrEqual(sticky - 1);
  });
});

test.describe("right-to-left", () => {
  test("the sticky column moves to the right, the arrows mirror, and the far inline end renders", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    await page.locator("#dx-direction-toggle").click();
    await expect(demoFrame(page)).toHaveAttribute("dir", "rtl");
    const grid = dataGrid(page);
    await expect(cell(grid, 0, 1)).toBeVisible();
    // The sticky first column sits at the inline start, which is the right.
    const p = await placement(grid, 0, 0);
    expect(Math.abs(p.right - p.gridRight)).toBeLessThan(2);
    const next = await placement(grid, 0, 1);
    expect(next.right).toBeLessThanOrEqual(p.left + 1);

    await cell(grid, 0, 1).click();
    await page.keyboard.press("ArrowLeft"); // inline end in RTL: the next column
    await expect.poll(() => focusedCell(page)).toBe("0:2");
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focusedCell(page)).toBe("0:1");

    // scrollLeft is negative in RTL; the grid windows by the distance from the inline start.
    await scrollGrid(grid, 0, -1e9);
    await expect(cell(grid, 0, 49)).toBeVisible();
    await expect(cell(grid, 0, 2)).toHaveCount(0);
    expect(await mountedCells(grid)).toBeLessThan(1000);
    await page.keyboard.press("End");
    await expect.poll(() => focusedCell(page)).toBe("0:49");
    await expect.poll(() => inGridView(grid, 0, 49)).toBe(true);
  });
});

test.describe("gallery (items wrap into rows)", () => {
  test("windows the rows of 5,000 images and renders the last one when scrolled to", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = gallery(page);
    await expectStyled(grid);
    await expect(grid).toHaveAttribute("aria-colcount", "4");
    await expect(grid).toHaveAttribute("aria-rowcount", "1250");
    await expect(grid.locator('[data-grid-index="0"]')).toBeVisible();
    expect(await mountedCells(grid)).toBeLessThan(200);
    // Generated images, nothing fetched.
    await expect(grid.locator('[data-grid-index="0"] svg[role="img"]')).toHaveCount(1);
    await expect(grid.locator('[data-grid-index="4999"]')).toHaveCount(0);
    // The gallery is below the data grid: bring it into the page viewport first.
    await grid.scrollIntoViewIfNeeded();
    // Rows are measured as they render; scrolling to the end reaches the last item, whatever the
    // estimates were.
    await expect(async () => {
      await scrollGrid(grid, 1e9, 0);
      await expect(grid.locator('[data-grid-index="4999"]')).toBeInViewport({ timeout: 1000 });
    }).toPass({ timeout: 15000 });
    expect(await mountedCells(grid)).toBeLessThan(200);
  });

  test("keyboard: rows wrap the items, and the short last row clamps the column", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = gallery(page);
    await grid.locator('[data-grid-index="1"]').click();
    expect(await focusedCell(page)).toBe("0:1");
    await page.keyboard.press("ArrowDown");
    await expect.poll(() => focusedCell(page)).toBe("1:1");
    await page.keyboard.press("Control+End");
    await expect.poll(() => focusedCell(page)).toBe("1249:3");
    await expect.poll(() => inGridView(grid, 1249, 3)).toBe(true);
    await page.keyboard.press("Control+Home");
    await expect.poll(() => focusedCell(page)).toBe("0:0");
  });

  test("changing the column count re-flows the items", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const grid = gallery(page);
    await expect(grid.locator('[data-grid-index="0"]')).toBeVisible();
    // Column options: 2, 3, 4, 6 (radio order).
    await demoFrame(page, "gallery").getByRole("radio").nth(3).check();
    await expect(grid).toHaveAttribute("aria-colcount", "6");
    await expect(grid).toHaveAttribute("aria-rowcount", "834");
    await expect(grid.locator('[role="row"][aria-rowindex="1"] [data-grid-cell]')).toHaveCount(6);
    expect(await mountedCells(grid)).toBeLessThan(300);
  });
});

// Scroll chaining: at a grid's top or bottom edge the wheel must scroll the page (the grid once set
// `overscroll-behavior: contain` and trapped it); only the inline axis stays contained.
for (const [name, pick] of [["data grid", dataGrid], ["gallery", gallery]] as const) {
  test.describe(`${name} scroll chaining`, () => {
    test.use({ viewport: { width: 1280, height: 600 } });

    test("the block axis chains to the page at both edges", async ({ page }) => {
      await gotoHydrated(page, PAGE_URL);
      const grid = pick(page);
      await expectStyled(grid);
      const overscroll = await grid.evaluate((el) => {
        const s = getComputedStyle(el);
        return [s.overscrollBehaviorX, s.overscrollBehaviorY];
      });
      expect(overscroll).toEqual(["contain", "auto"]);
      const pageScrollable = await page.evaluate(
        // viewport-ok: page scroll extent, not an overlay box
        () => document.documentElement.scrollHeight - window.innerHeight,
      );
      expect(pageScrollable).toBeGreaterThan(1000);

      const hover = async () => {
        const box = (await grid.boundingBox())!;
        await page.mouse.move(box.x + box.width / 2, box.y + Math.min(box.height / 2, 150));
      };

      // Bottom edge: wheel down scrolls the page.
      await grid.scrollIntoViewIfNeeded();
      await scrollGrid(grid, 1e9, 0);
      await expect.poll(() => grid.evaluate((el) => el.scrollHeight - el.clientHeight - el.scrollTop)).toBeLessThan(2);
      const before = await page.evaluate(() => window.scrollY);
      await hover();
      await page.mouse.wheel(0, 400);
      await expect.poll(() => page.evaluate(() => window.scrollY)).toBeGreaterThan(before);

      // Top edge: wheel up scrolls the page back.
      await grid.scrollIntoViewIfNeeded();
      await scrollGrid(grid, 0, 0);
      const mid = await page.evaluate(() => window.scrollY);
      await hover();
      await page.mouse.wheel(0, -400);
      await expect.poll(() => page.evaluate(() => window.scrollY)).toBeLessThan(mid);
    });
  });
}

test.describe("Axe automated scan", () => {
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    await expect(cell(dataGrid(page), 0, 1)).toBeVisible();
    await expect(gallery(page).locator('[data-grid-index="0"]')).toBeVisible();
    await expectNoAxeViolations(page, "virtual_grid: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
