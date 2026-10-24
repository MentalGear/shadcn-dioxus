import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * Backlog row 155: the styled `DatePicker`/`DateRangePicker` carry the
 * calendar's formatting callbacks -- `on_format_weekday`, `on_format_month` and
 * `on_format_month_short` -- to the calendar inside the popover, so the internationalized demo's popover
 * calendar is localised, and the month dropdown takes its short names from `on_format_month_short` instead of
 * cutting the full name to three characters (which gives French "Jui" for both juin and juillet).
 */
const PAGE_URL = `${BASE_URL}/component/date_picker/`;
const NAV_TIMEOUT = 20 * 60 * 1000; // first run compiles the app

const FR_FULL = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"];

function frame(page: Page) {
  return page.locator("#component-preview-frame-internationalized");
}

async function openCalendar(page: Page) {
  await frame(page).getByRole("button", { name: "Show Calendar" }).click();
  const dialog = frame(page).getByRole("dialog");
  await expect(dialog.locator(".dx-calendar")).toBeVisible({ timeout: 30000 });
  return dialog;
}

test.describe("DatePicker month and weekday formatting reach the popover calendar (variant=internationalized)", () => {
  test("English: the month dropdown prints Jan..Dec", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL, { timeout: NAV_TIMEOUT });
    const dialog = await openCalendar(page);
    await expect(dialog.getByRole("combobox", { name: "Month" }).locator("option")).toHaveText([
      "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]);
  });

  test("French: short names come from on_format_month_short, full names and weekdays from the locale", async ({ page }) => {
    await gotoHydrated(page, PAGE_URL, { timeout: NAV_TIMEOUT });
    await frame(page).getByRole("button", { name: "Français" }).click();
    await expect(frame(page).locator('[lang="fr-FR"]').first()).toBeVisible();
    const dialog = await openCalendar(page);

    const month = dialog.getByRole("combobox", { name: "Month" });
    // Twelve distinct names: a three-character cut of the full name would print "Jui" twice.
    await expect(month.locator("option")).toHaveText([
      "Janv.", "Févr.", "Mars", "Avr.", "Mai", "Juin", "Juil.", "Août", "Sept.", "Oct.", "Nov.", "Déc.",
    ]);
    // The hidden live region keeps the full, localised month name.
    const live = (await dialog.locator(".dx-calendar-sr-only").first().textContent())!.trim();
    expect(FR_FULL).toContain(live.replace(/ \d{4}$/, ""));
    // The weekday header is the locale's (fr-FR.ftl), not the English default.
    await expect(dialog.locator(".dx-calendar-grid-day-header")).toHaveText(["Di", "Lu", "Ma", "Me", "Je", "Ve", "Sa"]);
  });
});
