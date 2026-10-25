/**
 * Backlog row 384: the controlled `Accordion` (`value` / `on_value_change`, an item's `value`), driven
 * through the `controlled` variant of the accordion page. The demo owns the open list in a signal and
 * writes `on_value_change`'s list back; two buttons outside the accordion set that signal, and a status
 * line counts how often the callback ran.
 */
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { demoFrame } from './demo-frame';
import { gotoHydrated } from './hydration';

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

async function load(page: import('@playwright/test').Page) {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO_OPTS);
  const frame = demoFrame(page, 'controlled');
  await expect(frame).toBeVisible();
  const trigger = (name: string) => frame.getByRole('button', { name, exact: true });
  const status = frame.getByText(/^Open: .* Callback calls: \d+\.$/);
  return { frame, trigger, status };
}

test('the open item follows the signal that lives outside the accordion', async ({ page }) => {
  const { frame, trigger, status } = await load(page);
  await expect(trigger('Overview')).toHaveAttribute('aria-expanded', 'true');
  await expect(trigger('Pricing')).toHaveAttribute('aria-expanded', 'false');
  await expect(status).toHaveText('Open: overview. Callback calls: 0.');

  // An outside change opens another item and, single mode, closes the first; nobody clicked a
  // trigger, so the callback does not run.
  await frame.getByRole('button', { name: 'Open support', exact: true }).click();
  await expect(trigger('Support')).toHaveAttribute('aria-expanded', 'true');
  await expect(trigger('Overview')).toHaveAttribute('aria-expanded', 'false');
  await expect(status).toHaveText('Open: support. Callback calls: 0.');

  await frame.getByRole('button', { name: 'Close all', exact: true }).click();
  for (const name of ['Overview', 'Pricing', 'Support']) {
    await expect(trigger(name)).toHaveAttribute('aria-expanded', 'false');
  }
  await expect(status).toHaveText('Open: none. Callback calls: 0.');
});

test('a trigger click reports the new list and the written-back value drives the items', async ({ page }) => {
  const { trigger, status } = await load(page);

  await trigger('Pricing').click();
  await expect(trigger('Pricing')).toHaveAttribute('aria-expanded', 'true');
  await expect(trigger('Overview')).toHaveAttribute('aria-expanded', 'false');
  await expect(status).toHaveText('Open: pricing. Callback calls: 1.');

  // The item's own open flag is the controlled state too.
  await expect(demoFrame(page, 'controlled').locator('.dx-accordion-item').nth(1)).toHaveAttribute('data-open', 'true');

  // The open item closes: the callback reports an empty list.
  await trigger('Pricing').click();
  await expect(trigger('Pricing')).toHaveAttribute('aria-expanded', 'false');
  await expect(status).toHaveText('Open: none. Callback calls: 2.');

  // After an outside close the accordion is still live: a click opens through the callback again.
  await trigger('Support').click();
  await expect(trigger('Support')).toHaveAttribute('aria-expanded', 'true');
  await expect(status).toHaveText('Open: support. Callback calls: 3.');
});
