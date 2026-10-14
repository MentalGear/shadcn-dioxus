import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from './axe';
import { BASE_URL } from './base-url';

test('test', async ({ page }) => {
  await page.goto(`${BASE_URL}/component/?name=switch&`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  await page.waitForLoadState('networkidle');
  let switchElement = page.locator('#component-preview-frame').getByRole('switch', { name: 'Airplane Mode', exact: true });
  await expect(switchElement).toBeVisible();
  // The switch should not be checked initially
  await expect(switchElement).toHaveAttribute('data-state', 'unchecked');
  // Clicking the switch should check it
  await switchElement.click();
  await expect(switchElement).toHaveAttribute('data-state', 'checked');
  // Pressing space should also toggle the switch.
  // Use locator.press so the element is focused before the keystroke —
  // webkit does not always retain focus on a button after a synthetic click.
  await switchElement.press('Space');
  await expect(switchElement).toHaveAttribute('data-state', 'unchecked');
});

test.describe('Axe automated scan', () => {
  test('loaded (unchecked) has no automatically detectable a11y issues', async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=switch&`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
    await expectNoAxeViolations(page, 'switch: unchecked', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test('checked has no automatically detectable a11y issues', async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=switch&`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
    await page.locator('#component-preview-frame').getByRole('switch', { name: 'Airplane Mode', exact: true }).click();
    await expectNoAxeViolations(page, 'switch: checked', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});
