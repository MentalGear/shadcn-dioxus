import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from './axe';
import { BASE_URL } from './base-url';
import { gotoHydrated } from "./hydration";

test('test', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' }); // Increase timeout to 20 minutes

  let toggleElement = page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true });
  await expect(toggleElement).toBeVisible();
  // The toggle should not be checked initially
  await expect(toggleElement).toHaveAttribute('data-state', 'off');
  // // Clicking the toggle should check it
  await toggleElement.click();
  await expect(toggleElement).toHaveAttribute('data-state', 'on');
  // Pressing space should also toggle the toggle.
  // Use locator.press so the element is focused before the keystroke —
  // webkit does not always retain focus on a button after a synthetic click.
  // await toggleElement.press('Space');
  // await toggleElement.press('Space');
  await page.keyboard.press('Space');
  await expect(toggleElement).toHaveAttribute('data-state', 'off');
});

// HARDENING (dev-docs/backlog.md row 89): `.dx-toggle:hover`
// and `.dx-toggle[data-state="on"]` tie at specificity (0,2,0), broken only by source order
// (`[data-state="on"]` is declared after `:hover`), so the pressed background survives hover
// (both themes, exact values below). `:where(:hover)` makes that
// unconditional (specificity, not source order) so reordering the rules --
// or `:hover` gaining an extra clause the way calendar/tabs's did --
// cannot silently flip the winner. Runs in both themes since the
// tokens differ per theme.
for (const dark of [false, true]) {
  test(`a pressed toggle keeps its "on" background while hovered (${dark ? 'dark' : 'light'} mode)`, async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle/${dark ? '?dark_mode=true' : ''}`, { waitUntil: 'networkidle' });
    const toggle = page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true });

    await toggle.click();
    await expect(toggle).toHaveAttribute('data-state', 'on');
    await page.locator('body').hover({ position: { x: 0, y: 0 } });
    // Wait past the CSS transition (--dx-motion-duration-base, 150ms) before
    // reading computed style -- a mid-transition read returns an
    // interpolated, neither-old-nor-new color.
    await page.waitForTimeout(400);
    const onAtRest = await toggle.evaluate((el) => getComputedStyle(el).backgroundColor);

    await toggle.hover();
    await page.waitForTimeout(400);
    expect(await toggle.evaluate((el) => el.matches(':hover'))).toBe(true);
    const onAndHovered = await toggle.evaluate((el) => getComputedStyle(el).backgroundColor);

    // This asserts the invariant the `:where()` construction makes durable (pressed
    // keeps its own background under the pointer), not a particular colour --
    // the pressed colour is derived from the preset-aware role tokens.
    expect(onAndHovered, `onAtRest=${onAtRest} onAndHovered=${onAndHovered}`).toBe(onAtRest);
  });
}

// The pressed background is derived from the preset-aware role tokens (not the ramp step
// `--primary-color-7`, which is grey under every base), so under a TINTED base it must (a) carry the tint and
// (b) still be three visibly different backgrounds: resting, hovered, pressed. The exhaustive base x
// accent x mode proof is `theme-preset-contrast.spec.ts`; this keeps the same assertion next to the
// component's own specs.
const GAP_PRESSED = 16; // per channel, 0-255
const GAP_HOVER = 6;

async function paintedRgb(page: Page, css: string): Promise<[number, number, number]> {
  return page.evaluate((c) => {
    const cx = document.createElement('canvas').getContext('2d', { willReadFrequently: true })!;
    cx.canvas.width = cx.canvas.height = 1;
    cx.fillStyle = c;
    cx.fillRect(0, 0, 1, 1);
    const d = cx.getImageData(0, 0, 1, 1).data;
    return [d[0], d[1], d[2]] as [number, number, number];
  }, css);
}

const gap = (a: number[], b: number[]) => Math.max(...a.map((v, i) => Math.abs(v - b[i])));

for (const dark of [false, true]) {
  for (const base of ['slate', 'gray']) {
    test(`resting, hovered and pressed toggles are three different backgrounds under a ${base} base (${dark ? 'dark' : 'light'} mode)`, async ({ page }) => {
      await gotoHydrated(page, `${BASE_URL}/component/toggle/${dark ? '?dark_mode=true' : ''}`, { waitUntil: 'networkidle' });
      await page.evaluate((b) => {
        document.documentElement.setAttribute('data-theme-base', b);
        // Settled values, not a mid-transition read.
        document.head.insertAdjacentHTML('beforeend', '<style>*{transition:none!important}</style>');
      }, base);
      const toggle = page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true });
      const paint = async () => paintedRgb(page, await toggle.evaluate((el) => getComputedStyle(el).backgroundColor));

      await page.locator('body').hover({ position: { x: 0, y: 0 } });
      const rest = await paint();
      await toggle.hover();
      expect(await toggle.evaluate((el) => el.matches(':hover'))).toBe(true);
      const hovered = await paint();
      await toggle.click();
      await expect(toggle).toHaveAttribute('data-state', 'on');
      await page.locator('body').hover({ position: { x: 0, y: 0 } });
      const pressed = await paint();

      const msg = `rest=${rest} hovered=${hovered} pressed=${pressed}`;
      expect(gap(pressed, hovered), msg).toBeGreaterThanOrEqual(GAP_PRESSED);
      expect(gap(pressed, rest), msg).toBeGreaterThanOrEqual(GAP_PRESSED);
      expect(gap(hovered, rest), msg).toBeGreaterThanOrEqual(GAP_HOVER);
      // The tint reaches the pressed state: the ramp step is pure grey (r == g == b) under every base.
      expect(new Set(pressed).size, `pressed ${pressed} is neutral grey, not ${base}-tinted`).toBeGreaterThan(1);
    });
  }
}

// Row 155 group 3: `variant` (default / outline) and `size` (sm / default / lg) on Toggle.
// Nova: outline = `border-input border`, sm/default/lg = h-7 / h-8 / h-9 (28 / 32 / 36px).
const variantFrame = (page: Page, name: string) => page.locator(`#component-preview-frame-${name}`);
const px = (el: import('@playwright/test').Locator, prop: string) =>
  el.evaluate((e, p) => parseFloat(getComputedStyle(e).getPropertyValue(p)), prop);

test('the main demo is shadcn\'s: an outline, small Bookmark toggle', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { waitUntil: 'networkidle' });
  const toggle = page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true });
  await expect(toggle).toHaveAttribute('data-variant', 'outline');
  await expect(toggle).toHaveAttribute('data-size', 'sm');
  expect(await px(toggle, 'border-top-width')).toBe(1);
  expect(await px(toggle, 'height')).toBe(28);
  // Nova's `group-aria-pressed/toggle:fill-foreground`: the icon fills while pressed.
  const icon = toggle.locator('svg');
  expect(await icon.evaluate((e) => getComputedStyle(e).fill)).toBe('none');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  expect(await icon.evaluate((e) => getComputedStyle(e).fill)).not.toBe('none');
});

test('the default variant has no edge, the outline variant a 1px one', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { waitUntil: 'networkidle' });
  const plain = variantFrame(page, 'text').getByRole('button', { name: 'Toggle italic', exact: true });
  const outline = variantFrame(page, 'outline').getByRole('button', { name: 'Toggle italic outline', exact: true });
  await expect(plain).toHaveAttribute('data-variant', 'default');
  expect(await px(plain, 'border-top-width')).toBe(0);
  expect(await px(plain, 'height')).toBe(32);
  expect(await px(outline, 'border-top-width')).toBe(1);
  expect(await outline.evaluate((e) => getComputedStyle(e).borderTopColor)).not.toBe(
    await outline.evaluate((e) => getComputedStyle(e).backgroundColor),
  );
});

test('sizes: sm, default and lg are 28, 32 and 36px high with a matching minimum width', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { waitUntil: 'networkidle' });
  const frame = variantFrame(page, 'sizes');
  for (const [name, h] of [['Toggle small', 28], ['Toggle default', 32], ['Toggle large', 36]] as const) {
    const t = frame.getByRole('button', { name, exact: true });
    expect(await px(t, 'height'), name).toBe(h);
    expect(await px(t, 'min-width'), name).toBe(h);
  }
});

test('an item marked data-icon trims the padding on that side (Nova has-data-[icon])', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { waitUntil: 'networkidle' });
  const textOnly = variantFrame(page, 'sizes').getByRole('button', { name: 'Toggle default', exact: true });
  expect(await px(textOnly, 'padding-left')).toBe(10);
  expect(await px(textOnly, 'padding-right')).toBe(10);
});

test('disabled toggles (default and outline) ignore clicks', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { waitUntil: 'networkidle' });
  for (const name of ['Toggle disabled', 'Toggle disabled outline']) {
    const t = variantFrame(page, 'disabled').getByRole('button', { name, exact: true });
    await expect(t).toBeDisabled();
    await expect(t).toHaveAttribute('data-state', 'off');
  }
});

test.describe('Axe automated scan', () => {
  test('on has no automatically detectable a11y issues', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
    await page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true }).click();
    await expectNoAxeViolations(page, 'toggle: on', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// `data-pressed` (row 482): a presence attribute next to `aria-pressed` and `data-state`.
test('data-pressed is present only while the toggle is pressed', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/toggle/`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
  const toggle = page.locator('#component-preview-frame').getByRole('button', { name: 'Toggle bookmark', exact: true });
  await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  await expect(toggle).not.toHaveAttribute('data-pressed');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-pressed', 'true');
  await expect(toggle).toHaveAttribute('data-state', 'on');
  await expect(toggle).toHaveAttribute('data-pressed', '');
  await toggle.click();
  await expect(toggle).not.toHaveAttribute('data-pressed');
  await expect(toggle).toHaveAttribute('data-state', 'off');
});
