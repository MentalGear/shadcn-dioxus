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

// Backlog row 188: shadcn's Spinner svg carries `size-4`, which Button's `svg:not([class*='size-'])`
// icon rule skips, so a nested Spinner keeps 16px (or its typed size) in every Button size. The ring
// wears `dx-size-own` for the same skip. The demo's Button is resized in place to walk the sizes.
test("a Spinner keeps its own edge inside every Button size", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/spinner/`, { timeout: 20 * 60 * 1000 });
  const spinner = page.locator("#component-preview-frame-button").first().getByRole("status", { name: "Generating" });
  await expect(spinner).toBeVisible({ timeout: 60_000 });
  for (const size of ["xs", "sm", "default", "lg", "icon-xs", "icon-sm", "icon", "icon-lg"]) {
    for (const [spinnerSize, edge] of [["default", 16], ["sm", 12], ["lg", 24]] as const) {
      const width = await spinner.evaluate(
        (e, [b, s]) => {
          e.closest(".dx-button")!.setAttribute("data-size", b);
          e.setAttribute("data-size", s);
          // Computed, not the box: the ring rotates, so its bounding box grows mid-turn.
          return parseFloat(getComputedStyle(e.querySelector("svg")!).width);
        },
        [size, spinnerSize] as const,
      );
      expect(width, `Button ${size}, Spinner ${spinnerSize}`).toBeCloseTo(edge, 0);
    }
  }
});
