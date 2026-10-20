import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/spinner/`, { timeout: 20 * 60 * 1000 });
  await expect(page.getByRole("status", { name: "Loading", exact: true })).toBeVisible();
  await expect(page.getByText("Processing payment...", { exact: true })).toBeVisible();
  await expect(page.getByRole("status", { name: "Generating" })).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Spinner has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/spinner/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "spinner: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
