import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame } from "./demo-frame";
import { gotoHydrated } from "./hydration";

// VirtualGrid hooks (backlog row 341), on the `selectable` demo: 10,000 orders in a sticky-header grid with a
// checkbox column (0), then Order, Status and Total columns whose headers are sort buttons. The three hooks:
// `column_sort` (aria-sort on the columnheader), `row_selected` (aria-selected + data-state on rows, keyed by
// the row's identity) and a control inside a cell (the control is the one tab stop, the arrows still move
// between cells, Space acts on it). Controls are located by role and position inside the demo frame (row 220).

const PAGE_URL = `${BASE_URL}/component/virtual_grid/`;

const grid = (page: Page) => demoFrame(page, "selectable").getByRole("grid");
const cell = (g: Locator, row: number | "header", column: number) =>
  g.locator(`[data-grid-cell="${row}:${column}"]`);
const rowAt = (g: Locator, rowindex: number) => g.locator(`[role="row"][aria-rowindex="${rowindex}"]`);
const checkboxIn = (g: Locator, row: number) => cell(g, row, 0).getByRole("checkbox");
const sortButton = (g: Locator, column: number) => cell(g, "header", column).getByRole("button");
const focused = (page: Page) =>
  page.evaluate(() => {
    const el = document.activeElement;
    return {
      cell: el?.closest("[data-grid-cell]")?.getAttribute("data-grid-cell") ?? null,
      role: el?.getAttribute("role") ?? null,
    };
  });
/** Every element in the grid's body that is in the tab order: cells and the controls inside them. */
const bodyTabStops = (g: Locator) =>
  g.evaluate((el) =>
    [...el.querySelectorAll('[data-grid-cell]:not([data-grid-cell^="header"]) , [data-grid-cell]:not([data-grid-cell^="header"]) *')]
      .filter((n) => (n as HTMLElement).tabIndex === 0)
      .map((n) => (n.closest("[data-grid-cell]") as HTMLElement).dataset.gridCell),
  );

test.describe("VirtualGrid header sort state (column_sort)", () => {
  test("only the sorted column's header carries aria-sort, and it follows the sort", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const g = grid(page);
    await expect(cell(g, "header", 1)).toBeVisible();
    for (const column of [0, 1, 2, 3]) await expect(cell(g, "header", column)).not.toHaveAttribute("aria-sort");

    await sortButton(g, 1).click();
    await expect(cell(g, "header", 1)).toHaveAttribute("aria-sort", "ascending");
    await expect(cell(g, "header", 2)).not.toHaveAttribute("aria-sort");

    await sortButton(g, 1).click();
    await expect(cell(g, "header", 1)).toHaveAttribute("aria-sort", "descending");

    await sortButton(g, 3).click();
    await expect(cell(g, "header", 3)).toHaveAttribute("aria-sort", "ascending");
    await expect(cell(g, "header", 1)).not.toHaveAttribute("aria-sort");
    await expect(g.locator('[role="columnheader"][aria-sort]')).toHaveCount(1);
  });
});

test.describe("VirtualGrid row selection (row_selected)", () => {
  test("every row says whether it is selected; a selected one is data-state=selected and painted", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const g = grid(page);
    await expect(checkboxIn(g, 0)).toBeVisible();
    // The first data row is aria-rowindex 2 (the header is 1).
    await expect(rowAt(g, 2)).toHaveAttribute("aria-selected", "false");
    await expect(rowAt(g, 2)).not.toHaveAttribute("data-state");
    await expect(g.locator('[role="row"][data-state="selected"]')).toHaveCount(0);
    const plain = await rowAt(g, 2).evaluate((el) => getComputedStyle(el).backgroundColor);

    await checkboxIn(g, 0).click();
    await expect(rowAt(g, 2)).toHaveAttribute("aria-selected", "true");
    await expect(rowAt(g, 2)).toHaveAttribute("data-state", "selected");
    await expect(rowAt(g, 3)).toHaveAttribute("aria-selected", "false");
    const painted = await rowAt(g, 2).evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(painted).not.toBe(plain);
    // The sticky cell is opaque and takes the same fill, so scrolled cells never show through it.
    expect(await cell(g, 0, 0).evaluate((el) => getComputedStyle(el).backgroundColor)).toBe(painted);
  });

  test("a selection survives a window move and follows its row through a sort", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const g = grid(page);
    await checkboxIn(g, 0).click();
    await expect(rowAt(g, 2)).toHaveAttribute("data-state", "selected");

    // Move the window far away and back: the row is re-mounted (or rejoins) still selected.
    await g.evaluate((el) => (el.scrollTop = 5000 * 36));
    await expect(cell(g, 5000, 1)).toBeVisible();
    await expect(rowAt(g, 2)).toHaveCount(0);
    await expect(g.locator('[role="row"][data-state="selected"]')).toHaveCount(0);
    await g.evaluate((el) => (el.scrollTop = 0));
    await expect(rowAt(g, 2)).toHaveAttribute("data-state", "selected");
    await expect(checkboxIn(g, 0)).toBeChecked();

    // Sort the Order column descending: item 0 has the lowest order number, so it moves to the last slot.
    // Selection is keyed by the item, not the slot: the first row is no longer selected, the last one is.
    await sortButton(g, 1).click();
    await sortButton(g, 1).click();
    await expect(cell(g, "header", 1)).toHaveAttribute("aria-sort", "descending");
    await expect(rowAt(g, 2)).toHaveAttribute("aria-selected", "false");
    await g.evaluate((el) => (el.scrollTop = 1e9));
    await expect(rowAt(g, 10001)).toHaveAttribute("aria-selected", "true");
    await expect(rowAt(g, 10001)).toHaveAttribute("data-state", "selected");
    await expect(g.locator('[role="row"][data-state="selected"]')).toHaveCount(1);
  });
});

test.describe("VirtualGrid controls inside cells", () => {
  test("the control is the one tab stop; arrows move between cells; Space acts on the control", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const g = grid(page);
    await expect(checkboxIn(g, 0)).toBeVisible();
    // One tab stop in the body, and it is the active cell's control, not the cell.
    await expect.poll(() => bodyTabStops(g)).toEqual(["0:0"]);
    await expect(cell(g, 0, 0)).toHaveAttribute("tabindex", "-1");
    await expect(checkboxIn(g, 0)).toHaveAttribute("tabindex", "0");
    await expect(checkboxIn(g, 1)).toHaveAttribute("tabindex", "-1");

    // Click the checkbox: it toggles, its cell is active, and focus is on the control.
    await checkboxIn(g, 0).click();
    expect(await focused(page)).toEqual({ cell: "0:0", role: "checkbox" });
    await expect(checkboxIn(g, 0)).toBeChecked();

    // Space acts on the focused control.
    await page.keyboard.press("Space");
    await expect(checkboxIn(g, 0)).not.toBeChecked();

    // ArrowDown moves to the next cell's control (focus on the control), and the stop moves with it.
    await page.keyboard.press("ArrowDown");
    await expect.poll(() => focused(page)).toEqual({ cell: "1:0", role: "checkbox" });
    await expect.poll(() => bodyTabStops(g)).toEqual(["1:0"]);
    await page.keyboard.press("Space");
    await expect(checkboxIn(g, 1)).toBeChecked();
    await expect(rowAt(g, 3)).toHaveAttribute("data-state", "selected");

    // ArrowRight leaves the control for the next cell, which has none: the cell itself is the stop.
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => focused(page)).toEqual({ cell: "1:1", role: null });
    await expect.poll(() => bodyTabStops(g)).toEqual(["1:1"]);
    await expect(checkboxIn(g, 1)).toHaveAttribute("tabindex", "-1");
    // ArrowLeft returns to the control, and Space still acts on it.
    await page.keyboard.press("ArrowLeft");
    await expect.poll(() => focused(page)).toEqual({ cell: "1:0", role: "checkbox" });
    await page.keyboard.press("Space");
    await expect(checkboxIn(g, 1)).not.toBeChecked();

    // Far moves keep the promise: the control of a cell scrolled to by Ctrl+End is the one stop.
    await page.keyboard.press("Control+End");
    await expect.poll(() => focused(page)).toEqual({ cell: "9999:3", role: null });
    await page.keyboard.press("Home");
    await expect.poll(() => focused(page)).toEqual({ cell: "9999:0", role: "checkbox" });
    await expect.poll(() => bodyTabStops(g)).toEqual(["9999:0"]);
    await page.keyboard.press("Space");
    await expect(checkboxIn(g, 9999)).toBeChecked();
  });

  test("Tab leaves the grid after its one stop, and Shift+Tab does not land on another body control", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL);
    const g = grid(page);
    await checkboxIn(g, 0).click();
    await page.keyboard.press("ArrowDown");
    await expect.poll(() => focused(page)).toEqual({ cell: "1:0", role: "checkbox" });
    await page.keyboard.press("Tab");
    expect(await g.evaluate((el) => el.contains(document.activeElement))).toBe(false);
    await page.keyboard.press("Shift+Tab");
    // Back into the grid on its one body stop (the active cell's control).
    await expect.poll(() => focused(page)).toEqual({ cell: "1:0", role: "checkbox" });
  });
});
