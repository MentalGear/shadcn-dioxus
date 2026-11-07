// Overlay engine regressions found by the row 436 code-quality review (F1, F2, F4, F11). Each test is red
// without its fix: see the row's Status for the before/after numbers.
import { test, expect } from "./fixtures";
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';

test.describe('overlay engine', () => {
  test('F1: Escape while a closed submenu is held for its close animation closes the parent menu', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`, { timeout: 20 * 60 * 1000 });
    const trigger = page.getByRole('button', { name: 'Actions', exact: true });
    await trigger.click();
    await expect(trigger).toHaveAttribute('data-open', '');
    const sub = page.getByRole('menuitem', { name: 'Export as' });
    const submenu = page.getByRole('menu', { name: 'Export as' });
    await sub.click();
    await expect(page.getByRole('menuitem', { name: 'PDF document' })).toBeFocused();

    await page.keyboard.press('ArrowLeft');
    // The closed submenu stays mounted for its close animation plus a hold; the Escape lands in that window.
    await expect(submenu).toHaveAttribute('data-closed', '');
    await page.keyboard.press('Escape');
    await expect(trigger).toHaveAttribute('data-closed', '');
  });

  test('F2: closed overlays at mount run no close-animation eval', async ({ page }) => {
    await page.addInitScript(() => {
      const w = window as any;
      w.__closeEvals = 0;
      w.__evals = 0;
      w.Function = new Proxy(w.Function, {
        construct(target, args, newTarget) {
          const body = String(args[args.length - 1] ?? '');
          if (args[0] === 'dioxus') {
            w.__evals++;
            if (body.includes('getAnimations()') && body.includes('requestAnimationFrame')) w.__closeEvals++;
          }
          return Reflect.construct(target, args, newTarget);
        },
      });
    });
    await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`, { timeout: 20 * 60 * 1000 });
    await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
    const counts = await page.evaluate(() => ({ close: (window as any).__closeEvals, all: (window as any).__evals }));
    expect(counts.all, 'the probe sees the page evals').toBeGreaterThan(0);
    expect(counts.close, 'close-animation evals at mount').toBe(0);
  });

  test('F4: a second right-click on the trigger moves the open context menu to the new point', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/context_menu/`, { timeout: 20 * 60 * 1000 });
    const trigger = page.getByRole('button', { name: 'right click here' });
    await trigger.evaluate((el) => {
      (el as HTMLElement).style.position = 'fixed';
      (el as HTMLElement).style.top = '60px';
      (el as HTMLElement).style.left = '40px';
      (el as HTMLElement).style.width = '400px';
      (el as HTMLElement).style.height = '200px';
    });
    const menu = page.getByRole('menu');
    await page.mouse.click(80, 100, { button: 'right' });
    await expect(menu).toHaveAttribute('data-open', '');
    await page.mouse.click(300, 220, { button: 'right' });
    await expect(menu).toHaveAttribute('data-open', '');
    await expect.poll(async () => {
      const box = await menu.boundingBox();
      return box ? [Math.round(box.x), Math.round(box.y)] : null;
    }, { message: 'the menu corner sits at the second point' }).toEqual([300, 220]);
  });

  test('F11: Escape during IME composition does not close an overlay', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`, { timeout: 20 * 60 * 1000 });
    const trigger = page.getByRole('button', { name: 'Actions', exact: true });
    await trigger.click();
    await expect(trigger).toHaveAttribute('data-open', '');
    const prevented = await page.evaluate(() => {
      const make = (init: KeyboardEventInit) => new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true, ...init });
      const a = make({ isComposing: true });
      const b = make({ keyCode: 229 });
      (document.activeElement ?? document.body).dispatchEvent(a);
      (document.activeElement ?? document.body).dispatchEvent(b);
      return [a.defaultPrevented, b.defaultPrevented];
    });
    expect(prevented).toEqual([false, false]);
    await expect(trigger).toHaveAttribute('data-open', '');
    await page.keyboard.press('Escape');
    await expect(trigger).toHaveAttribute('data-closed', '');
  });
});
