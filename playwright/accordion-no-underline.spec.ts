/**
 * Backlog row 423: an accordion header never underlines on hover, collapsed or open. shadcn's AccordionTrigger
 * has `hover:underline`; the accordion sheet does not carry it over, so the trigger and every element inside
 * it compute `text-decoration-line: none` while hovered.
 */
import type { Locator } from '@playwright/test';
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

// The trigger itself and every element inside it: a text decoration painted on the header would show on the label.
const decorations = (trigger: Locator) =>
  trigger.evaluate((el) => [el, ...el.querySelectorAll('*')].map((n) => getComputedStyle(n).textDecorationLine));

test('an accordion header does not underline on hover, collapsed or open', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO_OPTS);
  const trigger = page.locator('.dx-component-examples .dx-accordion-item button.dx-accordion-trigger').nth(1);
  await expect(trigger).toBeVisible();

  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await trigger.hover();
  await expect.poll(async () => new Set(await decorations(trigger))).toEqual(new Set(['none']));

  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await page.mouse.move(0, 0);
  await trigger.hover();
  await expect.poll(async () => new Set(await decorations(trigger))).toEqual(new Set(['none']));
});
