import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => page.goto(`${BASE_URL}/component/?name=toast&`);

const R = "Update toast notifications";

test.describe("toast: show returns an id; update and close by id (row 138 step 0)", () => {
  test("update changes the SAME toast in place", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show", exact: true }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    await expect(t).toContainText("Syncing...");
    await expect(t).toHaveAttribute("data-type", "info");
    await t.evaluate((el) => ((el as HTMLElement).dataset.probe = "same-node"));

    await page.getByRole("button", { name: "Update", exact: true }).click();
    await expect(t).toContainText("Synced");
    await expect(t).toHaveAttribute("data-type", "success");
    await expect(t).toHaveCount(1);
    // Same element: updated, not removed and re-added.
    await expect(t).toHaveAttribute("data-probe", "same-node");
  });

  test("an id reaches only its own toast: the second show's id leaves the first alone", async ({ page }) => {
    await open(page);
    const show = page.getByRole("button", { name: "Show", exact: true });
    await show.click();
    await show.click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(2);
    await page.getByRole("button", { name: "Update", exact: true }).click();
    await expect(t.filter({ hasText: "Synced" })).toHaveCount(1);
    await expect(t.filter({ hasText: "Syncing..." })).toHaveCount(1);
  });

  test("close(id) removes it, and updating or closing the closed id is a no-op", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show", exact: true }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    await page.getByRole("button", { name: "Close", exact: true }).click();
    await expect(t).toHaveCount(0);
    // The demo still holds the dead id: neither call may resurrect or hit anything.
    await page.getByRole("button", { name: "Update", exact: true }).click();
    await page.getByRole("button", { name: "Close", exact: true }).click();
    await page.getByRole("button", { name: "Update", exact: true }).click();
    await expect(t).toHaveCount(0);
  });
});
