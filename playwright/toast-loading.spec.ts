import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";
import { gotoHydrated } from "./hydration";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => gotoHydrated(page, `${BASE_URL}/component/toast/`);

const R = "Loading toast notifications";

test.describe("toast: loading type (row 138 step 1)", () => {
  test("shows a spinner and is not auto-dismissed past the default duration", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show loading" }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    await expect(t).toHaveAttribute("data-type", "loading");
    await expect(t.locator(".dx-spinner")).toBeVisible();
    // The spinner is decoration under the toast's own live region: hidden from assistive tech.
    await expect(t.locator(".dx-toast-spinner")).toHaveAttribute("aria-hidden", "true");
    // The default duration is 5 s; a loading toast outlives it.
    await page.waitForTimeout(6000);
    await expect(t).toHaveCount(1);
    // It is still closable by hand.
    await t.getByRole("button", { name: "close", exact: true }).click();
    await expect(t).toHaveCount(0);
  });
});
