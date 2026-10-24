import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";
import { ACCENT, applyModeAndAccent, paintedColour, resolvedToken, type Mode } from "./drop-indicator";

// The `reorder` variant of data_table (backlog row 363): five tasks T-1..T-5, a drag-handle column and
// a Columns menu (Header is not hideable; Status, Reviewer, ID are).
const URL = `${BASE_URL}/component/block/?name=data_table&variant=reorder&`;
const goto = (page: Page) => gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });

const rows = (page: Page) => page.locator('[data-slot="table"] tbody tr');
const ids = async (page: Page) => (await rows(page).locator("td:last-child").allTextContents()).map((t) => t.trim());
const handle = (page: Page, key: string) => page.locator(`[data-dx-reorder-handle="${key}"]`);
const headers = (page: Page) => page.locator('[data-slot="table"] thead th');
const columnsButton = (page: Page) => page.getByRole("button", { name: "Columns" });

test.describe("DataTable reorderable rows", () => {
  test("a drag handle column leads every row and names its row", async ({ page }) => {
    await goto(page);
    await expect(rows(page)).toHaveCount(5);
    await expect(page.locator("[data-dx-reorder-handle]")).toHaveCount(5);
    await expect(handle(page, "T-2")).toHaveAttribute("aria-label", "Reorder row T-2");
    await expect(handle(page, "T-2")).toHaveAttribute("aria-roledescription", "drag handle");
    expect(await ids(page)).toEqual(["T-1", "T-2", "T-3", "T-4", "T-5"]);
  });

  test("dragging a handle onto another row moves the row", async ({ page }) => {
    await goto(page);
    const source = await measureAtRest(handle(page, "T-1"));
    const target = await measureAtRest(rows(page).nth(3));
    await page.mouse.move(source.x + source.width / 2, source.y + source.height / 2);
    await page.mouse.down();
    await page.mouse.move(source.x + 10, source.y + source.height / 2 + 10, { steps: 5 });
    await page.mouse.move(target.x + 120, target.y + target.height * 0.8, { steps: 15 });
    await expect(rows(page).nth(3)).toHaveAttribute("data-drop", "after");
    await page.mouse.up();
    await expect.poll(() => ids(page)).toEqual(["T-2", "T-3", "T-4", "T-1", "T-5"]);
  });

  test("Space lifts, Down chooses the slot, Space drops, and focus stays on the moved row's handle", async ({ page }) => {
    await goto(page);
    await handle(page, "T-1").focus();
    await page.keyboard.press("Space");
    await expect(handle(page, "T-1")).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowDown");
    await expect(page.getByRole("status")).toContainText("position 3");
    await page.keyboard.press("Space");
    await expect.poll(() => ids(page)).toEqual(["T-2", "T-3", "T-1", "T-4", "T-5"]);
    await expect(handle(page, "T-1")).toBeFocused();
    await expect(handle(page, "T-1")).toHaveAttribute("aria-pressed", "false");
  });

  test("Escape cancels a lift and the order is unchanged", async ({ page }) => {
    await goto(page);
    await handle(page, "T-2").focus();
    await page.keyboard.press("Enter");
    await page.keyboard.press("ArrowUp");
    await page.keyboard.press("Escape");
    expect(await ids(page)).toEqual(["T-1", "T-2", "T-3", "T-4", "T-5"]);
    await expect(handle(page, "T-2")).toHaveAttribute("aria-pressed", "false");
  });

  test("with nothing lifted the arrow keys move focus between handles", async ({ page }) => {
    await goto(page);
    await handle(page, "T-1").focus();
    await page.keyboard.press("ArrowDown");
    await expect(handle(page, "T-2")).toBeFocused();
    await page.keyboard.press("End");
    await expect(handle(page, "T-5")).toBeFocused();
    await page.keyboard.press("Home");
    await expect(handle(page, "T-1")).toBeFocused();
  });
});

test.describe("DataTable drop indicator colour", () => {
  // The drop line is `--dx-drop-indicator` (default `var(--dx-primary-mark)`, the accent clamped to 3:1 on every surface, row 389).
  for (const mode of ["light", "dark"] as Mode[]) {
    test(`the drop line is --dx-primary-mark under the ${ACCENT} accent, ${mode}`, async ({ page }) => {
      await goto(page);
      await applyModeAndAccent(page, mode);
      await handle(page, "T-1").focus();
      await page.keyboard.press("Space");
      await page.keyboard.press("ArrowDown");
      const target = rows(page).locator("xpath=..").locator("tr[data-drop]");
      await expect(target).toHaveCount(1);
      const mark = await resolvedToken(page, "--dx-primary-mark");
      expect(await resolvedToken(page, "--dx-drop-indicator")).toBe(mark);
      expect(await paintedColour(page, 'tr[data-drop] > td', "boxShadow")).toBe(mark);
      await page.keyboard.press("Escape");
    });
  }
});

test.describe("DataTable Columns menu", () => {
  test("a checkbox item hides and shows its column, header and cells together", async ({ page }) => {
    await goto(page);
    await expect(headers(page)).toHaveText(["Reorder", "Header", "Status", "Reviewer", "ID"]);
    await columnsButton(page).click();
    const status = page.getByRole("menuitemcheckbox", { name: "Status" });
    await expect(status).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("menuitemcheckbox", { name: "Header" })).toHaveCount(0);
    await status.click();
    await expect(status).toHaveAttribute("aria-checked", "false");
    await expect(headers(page)).toHaveText(["Reorder", "Header", "Reviewer", "ID"]);
    await expect(rows(page).first().locator("td")).toHaveCount(4);
    await status.click();
    await expect(status).toHaveAttribute("aria-checked", "true");
    await expect(headers(page)).toHaveText(["Reorder", "Header", "Status", "Reviewer", "ID"]);
  });

  test("Alt+Down on a menu item moves the column, header and cells, and keeps focus on it", async ({ page }) => {
    await goto(page);
    await columnsButton(page).click();
    const status = page.getByRole("menuitemcheckbox", { name: "Status" });
    await status.focus();
    await page.keyboard.press("Alt+ArrowDown");
    await expect(headers(page)).toHaveText(["Reorder", "Header", "Reviewer", "Status", "ID"]);
    await expect(rows(page).first().locator("td").nth(3)).toHaveText("In process");
    await expect(page.getByRole("menuitemcheckbox", { name: "Status" })).toBeFocused();
    await expect(page.getByRole("status")).toContainText("Moved the Status column");
    await page.keyboard.press("Alt+ArrowUp");
    await expect(headers(page)).toHaveText(["Reorder", "Header", "Status", "Reviewer", "ID"]);
  });
});
