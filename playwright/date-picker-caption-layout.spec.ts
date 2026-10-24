import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/** Backlog row 155: `caption_layout` on DatePicker reaches the popover's calendar (variant=date_of_birth). */
test.describe("DatePicker caption_layout pass-through (variant=date_of_birth)", () => {
  const group = (page: any, name: string) => page.locator(`[data-caption-layout="${name}"]`);

  async function open(page: any, name: string) {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=date_picker&variant=date_of_birth&`, { timeout: 20 * 60 * 1000 });
    const g = group(page, name);
    await g.getByRole("button", { name: "Show Calendar" }).click();
    await expect(g.locator(".dx-calendar")).toBeVisible({ timeout: 30000 });
    return g;
  }

  test("dropdown: Month + Year selects with short month names", async ({ page }) => {
    const g = await open(page, "Month and year");
    await expect(g.locator(".dx-calendar select")).toHaveCount(2);
    const month = g.getByRole("combobox", { name: "Month" });
    await expect(month.locator("option").nth(9)).toHaveText("Oct");
    await month.selectOption("2");
    await g.getByRole("combobox", { name: "Year" }).selectOption("1990");
    await expect(g.locator('.dx-calendar-grid-cell[data-month="current"]').first()).toHaveAttribute("aria-label", /February 1, 1990$/);
  });

  test("dropdown-months: only a month select; dropdown-years: only a year select", async ({ page }) => {
    const months = await open(page, "Month only");
    await expect(months.getByRole("combobox", { name: "Month" })).toHaveCount(1);
    await expect(months.getByRole("combobox", { name: "Year" })).toHaveCount(0);
  });

  test("dropdown-years", async ({ page }) => {
    const years = await open(page, "Year only");
    await expect(years.getByRole("combobox", { name: "Year" })).toHaveCount(1);
    await expect(years.getByRole("combobox", { name: "Month" })).toHaveCount(0);
  });

  test("the default (main variant) is a plain label", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/block/?name=date_picker&variant=main&`, { timeout: 20 * 60 * 1000 });
    await page.getByRole("button", { name: "Show Calendar" }).first().click();
    await expect(page.locator(".dx-calendar").first()).toBeVisible({ timeout: 30000 });
    await expect(page.locator(".dx-calendar select")).toHaveCount(0);
  });
});
