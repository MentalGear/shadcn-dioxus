import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/input/`, { timeout: 20 * 60 * 1000 });

  await page.getByRole('textbox', { name: 'Enter your name' }).fill('name');
  await expect(page.locator('#input-greeting')).toContainText('Hello, name!');
});
