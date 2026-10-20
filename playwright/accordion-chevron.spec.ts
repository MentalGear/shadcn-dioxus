/**
 * Backlog row 299: a collapsed accordion header does not underline on hover (an open one does), and
 * `chevron: AccordionChevron::Start` puts the chevron before the label. The docs component page's
 * accordion, and its "Manual installation files" Collapsible, use the start chevron.
 */
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };

test('a collapsed accordion header does not underline on hover; an open one does', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/accordion/`, GOTO_OPTS);
  const triggers = page.locator('.dx-component-examples button.dx-accordion-trigger');
  const trigger = triggers.first();
  await expect(trigger).toBeVisible();
  const decoration = () => trigger.evaluate((el) => getComputedStyle(el).textDecorationLine);
  await trigger.hover();
  const collapsed = await trigger.getAttribute('aria-expanded');
  expect(collapsed).toBe('false');
  expect(await decoration()).toBe('none');
  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await trigger.hover();
  await expect.poll(decoration).toContain('underline');
});

test('the docs page accordion and manual-install collapsible lead with a chevron that rotates open', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/button/`, GOTO_OPTS);
  const trigger = page.locator('h2#installation > button.dx-accordion-trigger');
  await expect(trigger).toHaveAttribute('data-chevron', 'start');
  const icon = trigger.locator('> .dx-accordion-expand-icon');
  const side = async (t: typeof trigger) =>
    t.evaluate((el) => {
      const i = el.querySelector('.dx-accordion-expand-icon')!.getBoundingClientRect(); // rest-ok: compares positions inside one static row
      const r = el.getBoundingClientRect(); // rest-ok: same row
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
  const pos = await trigger.evaluate((el) => {
    const i = el.querySelector('.dx-accordion-expand-icon')!.getBoundingClientRect(); // rest-ok: static row
    const r = el.getBoundingClientRect(); // rest-ok: static row
    return i.left - r.left > r.right - i.right;
  });
  expect(pos).toBe(true);
});
