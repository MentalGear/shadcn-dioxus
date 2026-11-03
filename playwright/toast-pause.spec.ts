import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { pauseAppClock } from "./app-clock";

const R = "Pausing toast notifications"; // each toast demo owns one provider; the demo's toast lasts 3 s
const region = (page: Page) => page.getByRole("region", { name: R });
const toast = (page: Page) => region(page).getByRole("alertdialog");

async function show(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/toast/`);
  await pauseAppClock(page); // the countdown runs on the page clock, which the test advances
  await page.getByRole("button", { name: "Show timed toast" }).click();
  await page.clock.runFor(100);
  await expect(toast(page)).toHaveCount(1);
}

test.describe("toast: pause on hover and focus (row 263 item 1)", () => {
  test("without hover or focus the toast is gone after its duration", async ({ page }) => {
    await show(page);
    await page.mouse.move(2, 2);
    await page.clock.runFor(2_700);
    await expect(toast(page)).toHaveCount(1);
    await page.clock.runFor(1_000); // past the 3 s and the exit
    await expect(toast(page)).toHaveCount(0);
  });

  test("hovering holds the toast past its duration; leaving resumes with the REMAINING time", async ({ page }) => {
    await show(page);
    await page.clock.runFor(1500); // about half of the 3 s used
    await toast(page).hover();
    await page.clock.runFor(3500); // well past the duration
    await expect(toast(page)).toHaveCount(1);
    await page.mouse.move(2, 2);
    // Remaining is about 1.5 s: still there a bit before that, gone a bit after. A restart in full
    // (3 s) would fail the second check.
    await page.clock.runFor(600);
    await expect(toast(page)).toHaveCount(1);
    await page.clock.runFor(1_400);
    await expect(toast(page)).toHaveCount(0);
  });

  test("focus inside the toaster holds the toast; blurring resumes", async ({ page }) => {
    await show(page);
    await page.mouse.move(2, 2);
    await toast(page).focus();
    await page.clock.runFor(4000);
    await expect(toast(page)).toHaveCount(1);
    await toast(page).evaluate((el) => (el as HTMLElement).blur());
    await page.clock.runFor(3_500);
    await expect(toast(page)).toHaveCount(0);
  });
});
