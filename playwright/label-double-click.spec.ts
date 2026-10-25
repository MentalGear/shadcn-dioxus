/**
 * Backlog row 384: like Radix's Label, a double click on a `Label` does not select its text
 * (`mousedown` is cancelled when `event.detail > 1`), while a drag still does. Dioxus's mouse events
 * carry no click count, so the primitive installs a scoped script once the pointer reaches the label.
 *
 * The themed `.dx-label` is `user-select: none` (shadcn's `select-none`, row 294), so nothing can be
 * selected in it whatever the primitive does; the tests lift that one declaration to exercise the
 * primitive's own guard, the part an unstyled `Label` user gets.
 */
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { demoFrame } from './demo-frame';
import { gotoHydrated } from './hydration';

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const selection = (page: import('@playwright/test').Page) => page.evaluate(() => window.getSelection()?.toString() ?? '');

async function loadLabel(page: import('@playwright/test').Page) {
  await gotoHydrated(page, `${BASE_URL}/component/label/`, GOTO_OPTS);
  await page.addStyleTag({ content: '.dx-label { -webkit-user-select: text !important; user-select: text !important; }' });
  const label = demoFrame(page).locator('label').first();
  await expect(label).toBeVisible();
  // The listener is installed when the pointer first reaches the label (before any click).
  await label.hover();
  await expect
    .poll(() => page.evaluate(() => [...((window as any).__dxJsListeners?.live?.values() ?? [])].includes('mousedown')))
    .toBe(true);
  return label;
}

test('a double click on a label does not select its text', async ({ page }) => {
  const label = await loadLabel(page);
  await page.evaluate(() => window.getSelection()?.removeAllRanges());
  await label.dblclick();
  expect(await selection(page)).toBe('');
});

test('a drag across a label still selects its text', async ({ page }) => {
  const label = await loadLabel(page);
  await page.evaluate(() => window.getSelection()?.removeAllRanges());
  const box = (await label.boundingBox())!; // rest-ok: a static label, no animation
  const y = box.y + box.height / 2;
  await page.mouse.move(box.x + 1, y);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 1, y, { steps: 6 });
  await page.mouse.up();
  expect(await selection(page)).toContain('Name');
});

test('control: a double click on ordinary text does select the word', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/label/`, GOTO_OPTS);
  const heading = page.locator('h1').first();
  await heading.dblclick();
  expect(await selection(page)).not.toBe('');
});
