import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { demoFrame, demoRole } from "./demo-frame";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

// Renderer identity (backlog row 361): `item_key` on VirtualList and TableRows. Everything that must
// survive a change of the view (the DOM node and so the state inside a row, the row that stays mounted
// while it holds focus, the measured sizes) follows the item, not the slot. Both `reorder` demos render
// the same list twice: with `item_key` (the default of the demo) and without it (the default of the
// library: a row is its slot), switched by the demo's last button.
//
// `reorder` demos, button positions: virtual_list = Reverse (0), Shuffle (1), and the item_key SWITCH (the
// first `switch`); table = Sort descending (0), Move first to last (1), item_key toggle (2).

const LIST_URL = `${BASE_URL}/component/virtual_list/`;
const TABLE_URL = `${BASE_URL}/component/table/`;

const press = (page: Page, n: number, variant: string) =>
  // dispatchEvent keeps keyboard focus where it is: a real click would move it to the button
  demoRole(page, "button", n, variant).dispatchEvent("click");

const itemButton = (frame: Locator, id: number) => frame.locator(`[data-item-button="${id}"]`);
const itemCheck = (frame: Locator, id: number) => frame.locator(`[data-item-check="${id}"]`);

test.describe("VirtualList item_key", () => {
  async function open(page: Page) {
    await gotoHydrated(page, LIST_URL, { timeout: 20 * 60 * 1000 });
    const frame = demoFrame(page, "reorder");
    await frame.scrollIntoViewIfNeeded();
    await expect(frame.locator('[role="list"]')).toHaveAttribute("data-virtualize", "windowed");
    await expect(itemButton(frame, 3)).toBeVisible();
    return frame;
  }

  test("focus, row state and the pinned row follow the item through a reverse", async ({ page }) => {
    const frame = await open(page);
    await itemCheck(frame, 3).check();
    await itemButton(frame, 3).focus();
    await press(page, 0, "reorder");
    // Item 3 is now the 197th row, far outside the window: it stays mounted because it holds focus.
    await expect(frame.locator('[data-virtual-pinned="true"]')).toHaveAttribute("data-virtual-index", "196");
    await expect(page.locator(":focus")).toHaveAttribute("data-item-button", "3");
    await expect(itemCheck(frame, 3)).toBeChecked();
    await expect(frame.locator('[data-virtual-index="0"] [data-item]')).toHaveAttribute("data-item", "199");
  });

  test("without item_key a row is its slot: focus and state stay on the slot", async ({ page }) => {
    const frame = await open(page);
    await demoRole(page, "switch", 0, "reorder").click(); // item_key off
    await expect(itemButton(frame, 3)).toBeVisible();
    await itemCheck(frame, 3).check();
    await itemButton(frame, 3).focus();
    await press(page, 0, "reorder");
    // Slot 3 now shows item 196 and still holds focus and the checked box.
    await expect(page.locator(":focus")).toHaveAttribute("data-item-button", "196");
    await expect(itemCheck(frame, 196)).toBeChecked();
    await expect(itemCheck(frame, 3)).toHaveCount(0);
  });

  test("a shuffle keeps the checked task's state and focus with the task", async ({ page }) => {
    const frame = await open(page);
    await itemCheck(frame, 3).check();
    await itemButton(frame, 3).focus();
    await press(page, 1, "reorder"); // Shuffle
    await expect(page.locator(":focus")).toHaveAttribute("data-item-button", "3");
    await expect(itemCheck(frame, 3)).toBeChecked();
    // The list really changed order: slot 3 no longer holds task 3.
    await expect(frame.locator('[data-virtual-index="3"] [data-item]')).not.toHaveAttribute("data-item", "3");
  });

  test("dragging a grip onto another row moves the task, and its checkbox goes with it", async ({ page }) => {
    const frame = await open(page);
    await itemCheck(frame, 0).check();
    const rows = frame.locator("[data-item]");
    const source = await measureAtRest(itemButton(frame, 0));
    const target = await measureAtRest(rows.nth(3));
    await page.mouse.move(source.x + source.width / 2, source.y + source.height / 2);
    await page.mouse.down();
    await page.mouse.move(source.x + 10, source.y + source.height / 2 + 10, { steps: 5 });
    await page.mouse.move(target.x + 120, target.y + target.height * 0.8, { steps: 15 });
    await expect(rows.nth(3)).toHaveAttribute("data-drop", "after");
    await page.mouse.up();
    await expect(frame.locator('[data-virtual-index="3"] [data-item]')).toHaveAttribute("data-item", "0");
    await expect(frame.locator('[data-virtual-index="0"] [data-item]')).toHaveAttribute("data-item", "1");
    await expect(itemCheck(frame, 0)).toBeChecked();
  });

  test("Space lifts, Down chooses the slot, Space drops, and focus stays on the moved task's grip", async ({ page }) => {
    const frame = await open(page);
    await itemButton(frame, 0).focus();
    await page.keyboard.press("Space");
    await expect(itemButton(frame, 0)).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Space");
    await expect(frame.locator('[data-virtual-index="2"] [data-item]')).toHaveAttribute("data-item", "0");
    await expect(itemButton(frame, 0)).toBeFocused();
    await expect(itemButton(frame, 0)).toHaveAttribute("aria-pressed", "false");
  });

  test("a reversed list keeps its measured heights: the canvas does not move", async ({ page }) => {
    const frame = await open(page);
    const scroller = frame.locator(".dx-virtual-list-container");
    // Measure every row once by scrolling the whole list past the viewport.
    // The canvas grows as rows are measured, so the end is read again on every step.
    for (let top = 0; top <= (await scroller.evaluate((el) => el.scrollHeight)); top += 300) {
      await scroller.evaluate((el, t) => (el.scrollTop = t), top);
      await page.waitForTimeout(40);
    }
    await scroller.evaluate((el) => (el.scrollTop = 0));
    await page.waitForTimeout(600);
    const before = await scroller.evaluate((el) => el.scrollHeight);
    // Sample the canvas height on every frame while the order flips.
    await scroller.evaluate((el) => {
      const samples: number[] = [];
      (window as any).__canvasSamples = samples;
      const tick = () => {
        samples.push(el.scrollHeight);
        if (samples.length < 90) requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    });
    await press(page, 0, "reorder");
    await page.waitForFunction(() => (window as any).__canvasSamples.length >= 90);
    const samples: number[] = await page.evaluate(() => (window as any).__canvasSamples);
    expect(Math.min(...samples)).toBe(before);
    expect(Math.max(...samples)).toBe(before);
    expect(await scroller.evaluate((el) => el.scrollTop)).toBe(0);
  });
});

test.describe("TableRows item_key", () => {
  async function open(page: Page) {
    await gotoHydrated(page, TABLE_URL, { timeout: 20 * 60 * 1000 });
    const frame = demoFrame(page, "reorder");
    await frame.scrollIntoViewIfNeeded();
    await expect(frame.locator("tbody")).toHaveAttribute("data-virtualize", "windowed");
    await expect(itemButton(frame, 5)).toBeVisible();
    return frame;
  }

  test("focus and row state follow the item when the first row moves to the end", async ({ page }) => {
    const frame = await open(page);
    await itemCheck(frame, 5).check();
    await itemButton(frame, 5).focus();
    await press(page, 1, "reorder");
    await expect(frame.locator('tr[data-virtual-index="4"] [data-item-button]')).toHaveAttribute(
      "data-item-button",
      "5",
    );
    await expect(page.locator(":focus")).toHaveAttribute("data-item-button", "5");
    await expect(itemCheck(frame, 5)).toBeChecked();
    await expect(itemCheck(frame, 6)).not.toBeChecked();
  });

  test("state follows the item through a full reverse", async ({ page }) => {
    const frame = await open(page);
    // A windowed body unmounts the rows that leave the window (an uncontrolled checkbox there is
    // reset, by design), and a full reverse sends the top rows to the bottom. So the item
    // that is checked sits in the middle of the 40 rows (19 -> row 20), which stays in the window.
    const scroller = frame.locator(".dx-table-container");
    await scroller.evaluate((el) => (el.scrollTop = 19 * 37 - 100));
    const id = 19;
    const middle = itemCheck(frame, id);
    await expect(middle).toBeVisible(); // the window has followed the scroll
    await middle.check();
    await press(page, 0, "reorder");
    // Item `id` is now row 39 - id: still in the window, same DOM node, still checked.
    await expect(frame.locator(`tr[data-virtual-index="${39 - id}"] [data-item-check]`)).toHaveAttribute(
      "data-item-check",
      String(id),
    );
    await expect(itemCheck(frame, id)).toBeChecked();
    await expect(itemCheck(frame, 39 - id)).not.toBeChecked();
  });

  test("without item_key a row is its slot: focus and state stay on the slot", async ({ page }) => {
    const frame = await open(page);
    await demoRole(page, "button", 2, "reorder").click(); // item_key off
    await expect(itemButton(frame, 5)).toBeVisible();
    await itemCheck(frame, 5).check();
    await itemButton(frame, 5).focus();
    await press(page, 1, "reorder");
    // Slot 5 now shows item 6 and still holds focus and the checked box.
    await expect(page.locator(":focus")).toHaveAttribute("data-item-button", "6");
    await expect(itemCheck(frame, 6)).toBeChecked();
    await expect(itemCheck(frame, 5)).not.toBeChecked();
  });
});
