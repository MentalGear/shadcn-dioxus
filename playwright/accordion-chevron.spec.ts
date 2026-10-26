/**
 * Backlog row 299: an accordion header does not underline on hover (row 423 removed the open-header underline), and
 * `chevron: AccordionChevron::Start` puts the chevron before the label. The docs component page's
 * accordion, and its "Manual installation files" Collapsible, use the start chevron.
 */
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';
import { readStatic } from "./animations";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

test('an accordion header does not underline on hover, collapsed or open', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO_OPTS);
  const trigger = page.locator('.dx-component-examples .dx-accordion-item button.dx-accordion-trigger').nth(1);
  await expect(trigger).toBeVisible();
  const decoration = () => trigger.evaluate((el) => getComputedStyle(el).textDecorationLine);
  await trigger.hover();
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  expect(await decoration()).toBe('none');
  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await trigger.hover();
  await expect.poll(decoration).toBe('none');
});

test('the docs page accordion and manual-install collapsible lead with a chevron that rotates open', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO_OPTS);
  const trigger = page.locator('h2#installation > button.dx-accordion-trigger');
  await expect(trigger).toHaveAttribute('data-chevron', 'start');
  const icon = trigger.locator('> .dx-accordion-expand-icon');
  const side = async (t: typeof trigger) =>
    readStatic(t, (el) => {
      const i = el.querySelector('.dx-accordion-expand-icon')!.getBoundingClientRect();
      const r = el.getBoundingClientRect();
      return { iconLeft: i.left - r.left, iconRight: r.right - i.right };
    });
  const closed = await side(trigger);
  expect(closed.iconLeft).toBeLessThan(closed.iconRight);
  expect(await icon.evaluate((el) => getComputedStyle(el).rotate)).toBe('-90deg');
  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await expect.poll(() => icon.evaluate((el) => getComputedStyle(el).rotate)).toMatch(/^(0deg|none)$/);

  const manual = page.locator('.dx-component-manual-install > button.dx-collapsible-trigger');
  await expect(manual).toHaveAttribute('data-chevron', 'start');
  await expect(manual.locator('svg')).toHaveCount(1);
  const m = await side(manual);
  expect(m.iconLeft).toBeLessThan(m.iconRight);
  await manual.click();
  await expect(manual).toHaveAttribute('aria-expanded', 'true');
  await expect.poll(() => manual.locator('.dx-accordion-expand-icon').evaluate((el) => getComputedStyle(el).rotate)).toMatch(/^(0deg|none)$/);
});

test('the default accordion keeps its trailing chevron', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO_OPTS);
  const trigger = page.locator('.dx-component-examples button.dx-accordion-trigger').first();
  await expect(trigger).toHaveAttribute('data-chevron', 'end');
  const pos = await readStatic(trigger, (el) => {
    const i = el.querySelector('.dx-accordion-expand-icon')!.getBoundingClientRect();
    const r = el.getBoundingClientRect();
    return i.left - r.left > r.right - i.right;
  });
  expect(pos).toBe(true);
});
