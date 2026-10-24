import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Backlog row 155 g8: the month dropdown prints the SHORT month name ("Oct") while the
 * caption label keeps the full one, the ISO week-number demo, and the `--dx-calendar-cell-size`
 * slot (shadcn's `--cell-size`).
 */
const goto = async (page: any, variant: string) => {
  await gotoHydrated(page, `${BASE_URL}/component/block/?name=calendar&variant=${variant}&`, { timeout: 20 * 60 * 1000 });
  await expect(page.locator(".dx-calendar").first()).toBeVisible({ timeout: 30000 });
};

test.describe("short month names in month dropdowns", () => {
  test("options and the shown value are 'Jan'..'Dec' / 'May'; the plain caption keeps the full name", async ({ page }) => {
    await goto(page, "caption");
    const both = page.locator('[data-caption-layout="Dropdown"]');
    const select = both.getByRole("combobox", { name: "Month" });
    const options = select.locator("option");
    await expect(options).toHaveText(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]);
    await expect(both.locator(".dx-calendar-month-select-value")).toHaveText("May");
    await select.selectOption("10");
    await expect(both.locator(".dx-calendar-month-select-value")).toHaveText("Oct");
    // The select value stays the month number; the hidden live region and aria-labels stay long.
    await expect(select).toHaveValue("10");
    await expect(both.locator(".dx-calendar-sr-only")).toHaveText("October 2026");
    // Layouts without a month dropdown keep the full name.
    await expect(page.locator('[data-caption-layout="Label"] .dx-calendar-month-title')).toHaveText("May 2026");
    await expect(page.locator('[data-caption-layout="Dropdown years"] .dx-calendar-caption-text')).toHaveText("May");
    await expect(page.locator('[data-caption-layout="Dropdown years"] .dx-calendar-caption-text')).not.toHaveText("Ma");
  });
});

test.describe("ISO week numbers (variant=iso_week_numbers)", () => {
  test("Monday-first: January 2027 opens with week 53 of 2026, then 1..4", async ({ page }) => {
    await goto(page, "iso_week_numbers");
    const numbers = page.locator(".dx-calendar-grid-week .dx-calendar-grid-weeknum");
    await expect(numbers).toHaveText(["53", "1", "2", "3", "4"]);
    await expect(numbers.first()).toHaveAttribute("aria-label", "Week 53");
    // Monday leads the weekday header.
    await expect(page.locator(".dx-calendar-grid-day-header").first()).toHaveText("Mo");
  });
});

test.describe("cell size (variant=cell_size)", () => {
  test("--dx-calendar-cell-size retunes the day cells and the month width", async ({ page }) => {
    await goto(page, "cell_size");
    const cell = (name: string) =>
      page.locator(`[data-cell-size="${name}"] .dx-calendar-grid-cell`).first().evaluate((el) => el.getBoundingClientRect().width);
    const small = await cell("Default (28px)");
    const large = await cell("Large (2.5rem)");
    expect(Math.abs(small - 28)).toBeLessThanOrEqual(1);
    expect(Math.abs(large - 40)).toBeLessThanOrEqual(1);
  });
});
