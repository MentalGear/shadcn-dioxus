import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// The main demo is the `#component-preview-frame` (variants get `-<name>`
// suffixes). Scoped there: "Default" would otherwise also match the
// description variant's "Default spacing" (names match as substrings).
const mainDemo = (page: import("@playwright/test").Page) => page.locator("#component-preview-frame");

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/radio_group/`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  await mainDemo(page).getByRole('radio', { name: 'Default', exact: true }).click();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Comfortable', exact: true })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Compact', exact: true })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(mainDemo(page).getByRole('radio', { name: 'Default', exact: true })).toBeFocused();
});

test.describe("Axe automated scan", () => {
  test("an item selected has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/radio_group/`, { timeout: 20 * 60 * 1000 });
    await mainDemo(page).getByRole('radio', { name: 'Default', exact: true }).click();
    await expectNoAxeViolations(page, "radio-group: Default selected", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// `read_only` (row 482): the items stay focusable and the arrow keys move focus, but nothing selects.
test("a read-only radio group keeps its selection, stays focusable and is announced", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/radio_group/`, { timeout: 20 * 60 * 1000 });
  const frame = page.locator("#component-preview-frame-read_only");
  const group = frame.getByRole("radiogroup", { name: "Shipping speed" });
  const standard = frame.locator("#radio-group-read-only-standard");
  const express = frame.locator("#radio-group-read-only-express");
  await expect(group).toHaveAttribute("aria-readonly", "true");
  await expect(group).toHaveAttribute("data-readonly", "");
  await expect(standard).toHaveAttribute("aria-checked", "true");

  // A click on the other item does not select it.
  await express.click();
  await expect(express).toBeFocused();
  await expect(express).toHaveAttribute("aria-checked", "false");
  await expect(standard).toHaveAttribute("aria-checked", "true");

  // Space does not select it either, and an arrow key moves focus without selecting.
  await page.keyboard.press("Space");
  await expect(express).toHaveAttribute("aria-checked", "false");
  await standard.focus();
  await page.keyboard.press("ArrowDown");
  await expect(express).toBeFocused();
  await expect(express).toHaveAttribute("aria-checked", "false");
  await expect(standard).toHaveAttribute("aria-checked", "true");
});
