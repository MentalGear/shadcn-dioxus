import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from './axe';
import { BASE_URL } from './base-url';
import { assertSharedBackdrop, captureBackdrop, resolveOverlayMs } from "./assert-backdrop-fade";
import { measureAtRest } from "./animations";

test('test', async ({ page }) => {
  await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  // Opened from the keyboard, so the first control takes focus. A pointer/touch open focuses the
  // panel instead (row 206; modal-initial-focus.spec.ts) and would not pop the mobile keyboard.
  await page.getByRole('button', { name: 'Show Dialog' }).focus();
  await page.keyboard.press('Enter');
  // Assert the dialog is open
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  // The close button is the LAST child of the panel (shadcn's DOM order), so a keyboard open
  // lands on the first control of the body ("Open Nested Dialog"), never on the X.
  const closeButton = dialog.getByRole('button', { name: 'Close', exact: true });
  await expect(dialog.getByRole('button', { name: 'Open Nested Dialog' })).toBeFocused();
  await expect(closeButton).not.toBeFocused();
  // The focus trap's tab cycle has two stops: Tab moves to the close button, and the next Tab
  // wraps back round to the first control.
  //
  // Phase 4.2 (docs/plan.md): the modal `Dialog` is a native `<dialog>` on the web arm, so this
  // cycle goes through Chromium's own focus trap. Chromium's native trap parks focus on `<body>`
  // for exactly one Tab stop after the last focusable element before wrapping to the first
  // (docs/phase4-spike-findings.md experiment 4a) -- invisible to the user and it does not let
  // focus escape the dialog, so this is a harness correction for the new trap's documented shape.
  await page.keyboard.press('Tab');
  await expect(closeButton).toBeFocused();
  await page.keyboard.press('Tab');
  await expect
    .poll(() => page.evaluate(() => document.activeElement === document.body))
    .toBe(true);
  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: 'Open Nested Dialog' })).toBeFocused();
  // Hitting escape should close the dialog
  await page.keyboard.press('Escape');
  // Assert the dialog can no longer be found
  await expect(dialog).toHaveCount(0);

  // Reopen the dialog
  await page.getByRole('button', { name: 'Show Dialog' }).click();
  // Assert the dialog is open again
  await expect(dialog).toBeVisible();
  // Click the close button
  await closeButton.click();
  // Assert the dialog is closed after clicking close
  await expect(dialog).toHaveCount(0);

  // Reopen the dialog
  await page.getByRole('button', { name: 'Show Dialog' }).click();
  await expect(dialog).toBeVisible();
  // Clicking far outside the dialog content should dismiss it.
  await page.mouse.click(2, 2);
  await expect(dialog).toHaveCount(0);
});

test('a press inside the dialog released over its backdrop does not dismiss it (text selection, drags); a real backdrop click does', async ({ page }) => {
  // A press inside the panel and a release over the `::backdrop` produce a `click` whose target is
  // the dialog itself, outside its box -- identical, by target and coordinates, to a backdrop click.
  // `use_dialog_backdrop_dismiss` (primitives/src/lib.rs) now also requires the PRESS to have begun
  // on the backdrop (the Drawer's "drag up on the handle closes it" report was this).
  await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
  await page.getByRole('button', { name: 'Show Dialog', exact: true }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  const title = dialog.getByText('Item information');
  await title.waitFor();
  const box = await measureAtRest(title);
  if (!box) throw new Error('dialog title has no bounding box');
  await page.mouse.move(box.x + 4, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + 4, 8, { steps: 6 });
  await page.mouse.up();
  await page.waitForTimeout(700);
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute('data-state', 'open');

  // Press AND release on the backdrop: dismissed.
  await page.mouse.click(2, 2);
  await expect(dialog).toHaveCount(0);
});

test('dialog stays open when clicking non-focusable content inside it', async ({ page }) => {
  // Regression: `use_outside_dismiss` (shared with Popover) served pointerdown
  // and focusin with one handler. Clicking a non-focusable region inside the
  // dialog (e.g. this demo's "Item information" title) blurs the currently
  // focused control, and the browser moves focus to the nearest focusable
  // *ancestor* -- outside the dialog's root while still containing it. The
  // shared handler read that as focus leaving and closed the dialog.
  await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
  await page.getByRole('button', { name: 'Show Dialog' }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();

  await dialog.getByText('Item information').click();

  await expect(dialog).toBeVisible();
});

test.describe('Axe automated scan', () => {
  test('loaded (dialog closed) has no automatically detectable a11y issues', async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
    // Wait for render before scanning -- see input.spec.ts's identical
    // comment for why (avoids a false pre-hydration "no main"/"no h1").
    await expect(page.getByRole('button', { name: 'Show Dialog' })).toBeVisible();
    await expectNoAxeViolations(page, 'dialog: loaded', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test('open has no automatically detectable a11y issues', async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
    await page.getByRole('button', { name: 'Show Dialog' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expectNoAxeViolations(page, 'dialog: open', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// The scrim every modal overlay shares (assert-backdrop-fade.ts has the full
// claim; popover/dialog/alert-dialog/sheet/drawer specs all run the identical
// assertion): the dialog's own `::backdrop` is the only painter, black/10 with a
// 4px blur, and it fades in and out for the same length. Before this, Dialog
// stacked its own animated wrapper scrim on the UA's un-animated one and the
// exit snapped.
test.describe("Scrim", () => {
  test("Dialog paints the shared scrim and fades it in and out", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
    const trigger = page.getByRole("button", { name: "Show Dialog", exact: true });
    await expect(trigger).toBeVisible();
    const capture = await captureBackdrop(
      page,
      () => trigger.click(),
      () => page.keyboard.press("Escape"),
    );
    assertSharedBackdrop(capture, await resolveOverlayMs(page), "dialog");
  });
});

test.describe('Parts (row 155, group 4)', () => {
  const open = async (page, label: string, variant: string) => {
    await page.goto(`${BASE_URL}/component/?name=dialog&`, { timeout: 20 * 60 * 1000 });
    await page.getByRole('button', { name: label, exact: true }).focus();
    await page.keyboard.press('Enter');
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();
    return dialog;
  };

  test('the default close button is the real Button, top-end, last in DOM order, and closes', async ({ page }) => {
    const dialog = await open(page, 'Show Dialog', 'main');
    const close = dialog.locator('[data-slot="dialog-close"]');
    await expect(close).toHaveCount(1);
    await expect(close).toHaveClass(/dx-button/);
    await expect(close).toHaveAttribute('data-style', 'ghost');
    await expect(close).toHaveAttribute('data-size', 'icon-sm');
    await expect(close).toHaveAttribute('type', 'button');
    await expect(close.locator('svg')).toHaveCount(1);
    // Last focusable in DOM order.
    const lastIsClose = await dialog.evaluate((el) => {
      const f = Array.from(el.querySelectorAll<HTMLElement>('button, [tabindex="0"], input'));
      return f[f.length - 1]?.getAttribute('data-slot') === 'dialog-close';
    });
    expect(lastIsClose).toBe(true);
    // Top-end corner: 8px from the panel's top and end edges.
    const [d, c] = await Promise.all([measureAtRest(dialog), measureAtRest(close)]);
    expect(Math.abs(c!.y - d!.y - 8)).toBeLessThanOrEqual(1);
    expect(Math.abs(d!.x + d!.width - (c!.x + c!.width) - 8)).toBeLessThanOrEqual(1);
    expect(Math.round(c!.width)).toBe(28);
    await close.click();
    await expect(dialog).toHaveCount(0);
  });

  test('show_close_button: false renders no X', async ({ page }) => {
    const dialog = await open(page, 'No Close Button', 'no_close');
    await expect(dialog.locator('[data-slot="dialog-close"]')).toHaveCount(0);
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
  });

  test('a form dialog opened by keyboard focuses its first field, not the X', async ({ page }) => {
    const dialog = await open(page, 'Open Dialog', 'edit_profile');
    await expect(dialog.getByLabel('Name', { exact: true })).toBeFocused();
    await expect(dialog.locator('[data-slot="dialog-header"]')).toHaveCount(1);
    await expect(dialog.locator('[data-slot="dialog-footer"]')).toHaveCount(1);
    await dialog.getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog).toHaveCount(0);
  });

  test('the footer is a muted band that cancels the panel padding; a row from sm up', async ({ page }) => {
    const dialog = await open(page, 'Sticky Footer', 'sticky_footer');
    const footer = dialog.locator('[data-slot="dialog-footer"]');
    const [d, f] = await Promise.all([measureAtRest(dialog), measureAtRest(footer)]);
    expect(Math.abs(f!.x - d!.x)).toBeLessThanOrEqual(1);
    expect(Math.abs(f!.width - d!.width)).toBeLessThanOrEqual(1);
    expect(Math.abs(f!.y + f!.height - (d!.y + d!.height))).toBeLessThanOrEqual(1);
    const css = await footer.evaluate((el) => {
      const s = getComputedStyle(el);
      return { dir: s.flexDirection, justify: s.justifyContent, top: s.borderTopWidth, bg: s.backgroundColor };
    });
    expect(css.dir).toBe('row');
    expect(css.justify).toBe('flex-end');
    expect(css.top).toBe('1px');
    expect(css.bg).not.toBe('rgba(0, 0, 0, 0)');
    // The body scrolls, the footer does not.
    await expect(dialog.getByRole('region')).toBeVisible();
    await dialog.getByRole('button', { name: 'Close', exact: true }).last().click();
    await expect(dialog).toHaveCount(0);
  });

  test('DialogFooter column-reverses below sm', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    const dialog = await open(page, 'Open Dialog', 'edit_profile');
    const dir = await dialog.locator('[data-slot="dialog-footer"]').evaluate((el) => getComputedStyle(el).flexDirection);
    expect(dir).toBe('column-reverse');
  });

  test('the share dialog: X plus a start-aligned footer Close, both close', async ({ page }) => {
    const dialog = await open(page, 'Share', 'share');
    await expect(dialog.getByRole('button', { name: 'Close', exact: true })).toHaveCount(2);
    await expect(dialog.getByLabel('Link')).toHaveValue('https://ui.shadcn.com/docs/installation');
    await dialog.getByRole('button', { name: 'Close', exact: true }).first().click();
    await expect(dialog).toHaveCount(0);
  });
});
