import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/kbd/`, { timeout: 20 * 60 * 1000 });
  await expect(page.locator("kbd", { hasText: "Ctrl" })).toBeVisible();
  await expect(page.locator("kbd", { hasText: "B" })).toBeVisible();
  await expect(page.getByText("to open the command palette.")).toBeVisible();
});

test.describe("Axe automated scan", () => {
  // Kbd has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/kbd/`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "kbd: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
