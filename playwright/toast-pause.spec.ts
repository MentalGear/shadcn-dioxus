import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const R = "Pausing toast notifications"; // each toast demo owns one provider; the demo's toast lasts 3 s
const region = (page: Page) => page.getByRole("region", { name: R });
const toast = (page: Page) => region(page).getByRole("alertdialog");

async function show(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/toast/`);
  await page.getByRole("button", { name: "Show timed toast" }).click();
  await expect(toast(page)).toHaveCount(1);
}

test.describe("toast: pause on hover and focus (row 263 item 1)", () => {
  test("without hover or focus the toast is gone after its duration", async ({ page }) => {
    await show(page);
    await page.mouse.move(2, 2);
    await expect(toast(page)).toHaveCount(0, { timeout: 5000 });
  });

  test("hovering holds the toast past its duration; leaving resumes with the REMAINING time", async ({ page }) => {
    await show(page);
    await page.waitForTimeout(1500); // about half of the 3 s used
    await toast(page).hover();
    await page.waitForTimeout(3500); // well past the duration
    await expect(toast(page)).toHaveCount(1);
    await page.mouse.move(2, 2);
    // Remaining is about 1.5 s: still there a bit before that, gone a bit after. A restart in full
    // (3 s) would fail the second check.
    await page.waitForTimeout(600);
    await expect(toast(page)).toHaveCount(1);
    await expect(toast(page)).toHaveCount(0, { timeout: 2000 });
  });

  test("focus inside the toaster holds the toast; blurring resumes", async ({ page }) => {
    await show(page);
    await page.mouse.move(2, 2);
    await toast(page).focus();
    await page.waitForTimeout(4000);
    await expect(toast(page)).toHaveCount(1);
    await toast(page).evaluate((el) => (el as HTMLElement).blur());
    await expect(toast(page)).toHaveCount(0, { timeout: 5000 });
  });
});
