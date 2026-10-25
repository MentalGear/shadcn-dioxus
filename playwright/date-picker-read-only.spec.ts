/**
 * DatePicker `read_only` (backlog row 424): like a native `readonly` input, a read-only picker keeps focus
 * and selection and refuses every edit -- typing, the Up/Down/Home/End steps, Backspace/Delete (the "clear"
 * keys), and the calendar popover (it does not open). `on_value_change` / `on_range_change` is never called:
 * the `read_only` demo counts them in `#read-only-changes` (`data-changes`).
 *
 * The `read_only` variant holds a `DatePicker` (segments 0-2: year, month, day) then a `DateRangePicker`
 * (3-8: start year, month, day, end year, month, day), each with a value (2026-05-07, 2026-05-07..11).
 * Controls: the editable main demo still steps with ArrowUp and still opens its calendar, so the negatives
 * below cannot pass because the keys or the popover are broken everywhere.
 */
import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole, demoRoles } from "./demo-frame";

const URL = `${BASE_URL}/component/date_picker/`;
const NAV_TIMEOUT = 20 * 60 * 1000;
const VARIANT = "read_only";

const segment = (page: Page, n: number) => demoRole(page, "spinbutton", n, VARIANT);
const trigger = (page: Page, n: number) => demoRole(page, "button", n, VARIANT); // 0: picker, 1: range picker
const changes = (page: Page) => page.locator("#read-only-changes");
const calendar = (page: Page) => page.getByRole("dialog");

async function readValues(page: Page): Promise<(string | null)[]> {
  const out: (string | null)[] = [];
  for (let n = 0; n < 9; n++) out.push(await segment(page, n).getAttribute("aria-valuenow"));
  return out;
}

const INITIAL = ["2026", "5", "7", "2026", "5", "7", "2026", "5", "11"];

test.describe("DatePicker read_only (backlog row 424)", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: NAV_TIMEOUT });
    await segment(page, 0).scrollIntoViewIfNeeded();
    expect(await readValues(page)).toEqual(INITIAL);
  });

  test("segments stay focusable and selectable, and are marked read-only", async ({ page }) => {
    for (const n of [0, 4, 8]) {
      const s = segment(page, n);
      await expect(s).toHaveAttribute("aria-readonly", "true");
      await expect(s).toHaveAttribute("contenteditable", "false");
      await s.click();
      await expect(s).toBeFocused();
    }
  });

  test("the arrow keys move focus between segments", async ({ page }) => {
    await segment(page, 0).click();
    await page.keyboard.press("ArrowRight");
    await expect(segment(page, 1)).toBeFocused();
    await page.keyboard.press("ArrowRight");
    await expect(segment(page, 2)).toBeFocused();
    await page.keyboard.press("ArrowLeft");
    await expect(segment(page, 1)).toBeFocused();
    expect(await readValues(page)).toEqual(INITIAL);
  });

  test("no key edits a segment: digits, Up/Down, Home/End, Backspace and Delete change nothing", async ({ page }) => {
    // One segment of each kind, in the single picker and in both ends of the range.
    for (const n of [0, 1, 2, 3, 5, 7, 8]) {
      await segment(page, n).click();
      await expect(segment(page, n)).toBeFocused();
      for (const key of ["1", "9", "ArrowUp", "ArrowDown", "Home", "End", "Backspace", "Delete", "0", "ArrowUp"]) {
        await page.keyboard.press(key);
      }
      await expect(segment(page, n)).toBeFocused();
    }
    // The value is re-read after a beat so a late write (an effect) would be seen too.
    await page.waitForTimeout(300);
    expect(await readValues(page)).toEqual(INITIAL);
    await expect(changes(page)).toHaveAttribute("data-changes", "0");
  });

  test("the calendar does not open: not from the trigger, not by Enter or Space on it", async ({ page }) => {
    for (const n of [0, 1]) {
      const t = trigger(page, n);
      await expect(t).toHaveAttribute("aria-disabled", "true");
      await t.focus();
      await expect(t).toBeFocused(); // read-only keeps the trigger focusable, unlike `disabled`
      await t.click({ force: true }); // Playwright treats aria-disabled as not actionable; a user can still press it
      await page.keyboard.press("Enter");
      await page.keyboard.press("Space");
      await page.waitForTimeout(500); // past the popover's mount and enter animation
      await expect(calendar(page)).toHaveCount(0);
      await expect(t).toHaveAttribute("aria-expanded", "false");
    }
    expect(await readValues(page)).toEqual(INITIAL);
    await expect(changes(page)).toHaveAttribute("data-changes", "0");
  });

  test("control: an editable picker's segments step and its trigger opens the calendar", async ({
    page,
  }) => {
    const year = demoRole(page, "spinbutton", 0, "main");
    await expect(year).not.toHaveAttribute("aria-valuenow", /.*/); // untouched
    await year.click();
    await page.keyboard.press("ArrowUp");
    await expect(year).toHaveAttribute("aria-valuenow", /^\d{4}$/);
    await expect(year).not.toHaveAttribute("aria-readonly", /.*/);

    await demoRoles(page, "button").first().click();
    await expect(calendar(page)).toBeVisible();
  });
});
