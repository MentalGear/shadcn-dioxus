import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/empty/`, { timeout: 20 * 60 * 1000 });
  // Scope to the demo frame. The component page also renders this component's
  // own source in a syntax-highlighted code viewer, so every visible string
  // here appears a second time as a highlighted token span.
  const demo = page.locator("#component-preview-frame").first();
  await expect(demo.getByText("No messages yet")).toBeVisible();
  await expect(demo.getByText("You don't have any messages.")).toBeVisible();
  await expect(demo.getByRole("button", { name: "New message" })).toBeVisible();
});
