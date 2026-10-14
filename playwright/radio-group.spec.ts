import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";

// The main demo is the `#component-preview-frame` (variants get `-<name>`
// suffixes). Scoped there: "Default" would otherwise also match the
// description variant's "Default spacing" (names match as substrings).
const mainDemo = (page: import("@playwright/test").Page) => page.locator("#component-preview-frame");

test("test", async ({ page }) => {
  await page.goto(`${BASE_URL}/component/?name=radio_group&`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  await mainDemo(page).getByRole('radio', { name: 'Default', exact: true }).click();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Comfortable', exact: true })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Compact', exact: true })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Default', exact: true })).toBeFocused();
});

test.describe("Axe automated scan", () => {
  test("loaded (none selected) has no automatically detectable a11y issues", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=radio_group&`, { timeout: 20 * 60 * 1000 });
    // Wait for render before scanning -- see input.spec.ts's identical
    // comment for why (avoids a false pre-hydration "no main"/"no h1").
    await expect(mainDemo(page).getByRole('radio', { name: 'Default', exact: true })).toBeVisible();
    await expectNoAxeViolations(page, "radio-group: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test("an item selected has no automatically detectable a11y issues", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=radio_group&`, { timeout: 20 * 60 * 1000 });
    await mainDemo(page).getByRole('radio', { name: 'Default', exact: true }).click();
    await expectNoAxeViolations(page, "radio-group: Default selected", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
