/**
 * Scroll to an item (backlog row 398): `use_virtual_scroller` gives VirtualList and VirtualGrid a
 * public scroll-to-index. A far row that is not mounted lands at the requested alignment (the
 * demos' rows are all one height, so the landing is exact to the pixel), in a windowed list
 * (100,000 rows), in a content-visibility list (800 rows) and in a windowed grid with a pinned
 * header and column. Functional spec: no millisecond thresholds.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const LIST_URL = `${BASE_URL}/component/block/?name=virtual_list&variant=scroll_to`;
const GRID_URL = `${BASE_URL}/component/block/?name=virtual_grid&variant=scroll_to`;

async function choose(page: Page, name: string) {
  await page.getByRole("button", { name, exact: true }).click();
}

/** The row's box against the list's box. */
function rowBox(list: Locator, index: number) {
  return list.evaluate((node, index) => {
    const row = node.querySelector(`[data-virtual-index="${index}"]`);
    if (!row) return null;
    const box = node.getBoundingClientRect();
    const rect = row.getBoundingClientRect();
    return { top: rect.top - box.top - node.clientTop, bottom: rect.bottom - box.top - node.clientTop, height: node.clientHeight };
  }, index);
}

async function jumpToRow(page: Page, row: number) {
  await page.getByRole("spinbutton", { name: "Row number" }).fill(String(row));
  await page.getByRole("button", { name: "Scroll to row" }).click();
}

test.describe("VirtualList scroll-to-index", () => {
  test("a windowed list puts a far, unmounted row at the requested alignment", async ({ page }) => {
    await gotoHydrated(page, LIST_URL, GOTO_OPTS);
    const list = page.locator('[data-vl-demo="scroll-to"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "windowed");
    await expect(list.locator('[data-virtual-index="0"]')).toBeVisible({ timeout: 60_000 });
    expect(await rowBox(list, 4999)).toBeNull();

    await jumpToRow(page, 5000);
    await expect.poll(() => rowBox(list, 4999).then((box) => box?.top), { timeout: 15_000 }).toBe(0);
    await expect(page.locator("[data-vl-last-jump]")).toHaveText("row 5000, Start");

    await choose(page, "Center");
    await jumpToRow(page, 60000);
    await expect
      .poll(() => rowBox(list, 59999).then((box) => box && Math.abs((box.top + box.bottom) / 2 - box.height / 2)), { timeout: 15_000 })
      .toBeLessThanOrEqual(1);

    await choose(page, "End");
    await jumpToRow(page, 30000);
    await expect
      .poll(() => rowBox(list, 29999).then((box) => box && box.height - box.bottom), { timeout: 15_000 })
      .toBe(0);
  });

  test("Nearest leaves a visible row alone, and the last row reaches the end", async ({ page }) => {
    await gotoHydrated(page, LIST_URL, GOTO_OPTS);
    const list = page.locator('[data-vl-demo="scroll-to"] [role="list"]');
    await expect(list.locator('[data-virtual-index="0"]')).toBeVisible({ timeout: 60_000 });
    await jumpToRow(page, 5000);
    await expect.poll(() => rowBox(list, 4999).then((box) => box?.top), { timeout: 15_000 }).toBe(0);
    const before = await list.evaluate((node) => node.scrollTop);

    await choose(page, "Nearest");
    await jumpToRow(page, 5003);
    await expect(page.locator("[data-vl-last-jump]")).toHaveText("row 5003, Nearest");
    await page.waitForTimeout(500);
    expect(await list.evaluate((node) => node.scrollTop)).toBe(before);

    await page.locator("[data-vl-jump-last]").click();
    await expect
      .poll(() => list.evaluate((node) => node.scrollHeight - node.clientHeight - node.scrollTop), { timeout: 15_000 })
      .toBeLessThanOrEqual(1);
    await expect(list.locator('[data-virtual-index="99999"]')).toBeVisible();
  });

  test("a content-visibility list scrolls to the row too", async ({ page }) => {
    await gotoHydrated(page, LIST_URL, GOTO_OPTS);
    await choose(page, "800 rows");
    const list = page.locator('[data-vl-demo="scroll-to"] [role="list"]');
    await expect(list).toHaveAttribute("data-virtualize", "content-visibility");
    await choose(page, "Center");
    await jumpToRow(page, 600);
    const row = list.locator('[data-virtual-index="599"]');
    await expect
      .poll(
        () =>
          row.evaluate((el) => {
            const box = el.closest('[role="list"]')!.getBoundingClientRect();
            const rect = el.getBoundingClientRect();
            return Math.abs((rect.top + rect.bottom) / 2 - (box.top + box.bottom) / 2);
          }),
        { timeout: 15_000 },
      )
      .toBeLessThanOrEqual(2);
  });
});

test.describe("VirtualGrid scroll-to-cell", () => {
  test("a far cell lands past the pinned header and column", async ({ page }) => {
    await gotoHydrated(page, GRID_URL, GOTO_OPTS);
    const grid = page.locator('[data-vg-demo="scroll-to"] [role="grid"]');
    await expect(grid.locator('[data-grid-cell="0:0"]')).toBeVisible({ timeout: 60_000 });
    await page.getByRole("button", { name: "Scroll to cell" }).click();

    const cell = grid.locator('[data-grid-cell="4999:29"]');
    await expect(cell).toBeVisible({ timeout: 15_000 });
    const gap = () =>
      grid.evaluate((node) => {
        const cell = node.querySelector('[data-grid-cell="4999:29"]');
        const header = node.querySelector('[data-grid-header]');
        const pinned = node.querySelector('[data-grid-cell="4999:0"]');
        if (!cell || !header || !pinned) return null;
        const rect = cell.getBoundingClientRect();
        return { below: rect.top - header.getBoundingClientRect().bottom, beside: rect.left - pinned.getBoundingClientRect().right };
      });
    await expect.poll(gap, { timeout: 15_000 }).toEqual({ below: 0, beside: 0 });

    // A cell in the pinned column moves only the rows.
    const left = await grid.evaluate((node) => node.scrollLeft);
    await page.getByRole("spinbutton", { name: "Row number" }).fill("200");
    await page.getByRole("spinbutton", { name: "Column number" }).fill("1");
    await choose(page, "Center");
    await page.getByRole("button", { name: "Scroll to cell" }).click();
    await expect(grid.locator('[data-grid-cell="199:0"]')).toBeVisible({ timeout: 15_000 });
    expect(await grid.evaluate((node) => node.scrollLeft)).toBe(left);
  });
});
