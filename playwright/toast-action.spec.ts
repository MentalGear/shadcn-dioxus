import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";
import { gotoHydrated } from "./hydration";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => gotoHydrated(page, `${BASE_URL}/component/toast/`);

const R = "Action toast notifications";

test.describe("toast: action button (row 138 step 2)", () => {
  test("a real Button; click runs the callback and dismisses", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show action toast" }).click();
    const t = toasts(page, R);
    const undo = t.getByRole("button", { name: "Undo", exact: true });
    await expect(undo).toBeVisible();
    await expect(undo).toHaveClass(/dx-button/);
    await undo.click();
    await expect(t).toHaveCount(0);
    await expect(page.getByText("Undone: 1")).toBeVisible();
  });

  // F6 itself is not used here: this page mounts eight providers (one per demo), each registering F6,
  // and the last one to mount wins the focus. The toast is focused directly instead.
  test("keyboard: Tab from the toast reaches Undo, Enter activates and dismisses", async ({ page }) => {
    await open(page);
    await page.getByRole("button", { name: "Show action toast" }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    await t.focus();
    let reached = false;
    for (let i = 0; i < 6 && !reached; i++) {
      await page.keyboard.press("Tab");
      reached = await page.evaluate(
        () => document.activeElement?.textContent?.trim() === "Undo" && !!document.activeElement.closest('[role="alertdialog"]'),
      );
    }
    expect(reached, "Undo is in the tab order").toBe(true);
    await page.keyboard.press("Enter");
    await expect(t).toHaveCount(0);
    await expect(page.getByText("Undone: 1")).toBeVisible();
  });
});
