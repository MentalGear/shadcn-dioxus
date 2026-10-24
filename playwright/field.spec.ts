import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/field/`, { timeout: 20 * 60 * 1000 });
  // Label/control association: clicking the label focuses its input.
  const username = page.getByLabel("Username");
  await expect(username).toBeVisible();
  await username.click();
  await expect(username).toBeFocused();

  // The invalid field's error message is present and associated by role.
  await expect(page.getByRole("alert")).toHaveText("Enter a valid email address.");

  // The horizontal field: checkbox beside its label/description.
  await expect(page.getByRole("checkbox", { name: "Marketing emails" })).toBeVisible();
});
