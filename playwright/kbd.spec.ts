import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/kbd/`, { timeout: 20 * 60 * 1000 });
  await expect(page.locator("kbd", { hasText: "Ctrl" })).toBeVisible();
  await expect(page.locator("kbd", { hasText: "B" })).toBeVisible();
  await expect(page.getByText("to open the command palette.")).toBeVisible();
});
