/**
 * Backlog row 305 (iOS): scrolling a drawer's content must not drag the drawer sideways. The drag
 * decides its axis first
 * travelled `AXIS_LOCK_PX`, commits only when that movement is predominantly along the dismiss axis
 * (and no scrollable ancestor can scroll that way), and is otherwise abandoned for that pointer, so
 * `start`/`move` are never sent and the panel never shifts while content scrolls.
 *
 * Two drivers, for all four sides: synthetic touch Pointer Events (deterministic geometry, the wobble
 * and the diagonal) and, on Chromium, real touch input through CDP `Input.dispatchTouchEvent` (a real
 * browser scroll with sideways wobble). WebKit, where the bug shows, is not available here:
 * a real iPhone re-test is still required.
 */
import { test, expect } from './fixtures';
import { BASE_URL } from './base-url';
import { gotoHydrated } from './hydration';
import { awaitAnimationsSettled } from './animations';

const URL = `${BASE_URL}/component/drawer/`;
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const CONTENT = '[data-slot="drawer-content"]';
const ROOT = '[data-slot="drawer-root"]';

test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

type Side = 'bottom' | 'top' | 'left' | 'right';
const OPENERS: Record<Side, string> = {
  bottom: 'Move Goal',
  top: 'Open from Top',
  left: 'Open from Left',
  right: 'Open from Right',
};
const vertical = (side: Side) => side === 'bottom' || side === 'top';
/** Unit vector (x, y) toward the closing edge. */
const TOWARD_CLOSE: Record<Side, [number, number]> = { bottom: [0, 1], top: [0, -1], left: [-1, 0], right: [1, 0] };

async function open(page: import('@playwright/test').Page, side: Side) {
  await gotoHydrated(page, URL, GOTO_OPTS);
  await page.getByRole('button', { name: OPENERS[side] }).click();
  await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
  await awaitAnimationsSettled(page.locator(CONTENT));
  // Content that scrolls ALONG THE CROSS AXIS (what a user scrolls), plus a row that scrolls along
  // the dismiss axis is not needed: the panel itself (`overflow: auto`) is the cross-axis scroller.
  await page.locator(CONTENT).evaluate((el, v) => {
    const spacer = document.createElement('div');
    spacer.setAttribute('data-test-spacer', '');
    spacer.style.cssText = v ? 'width: 3000px; height: 10px; flex: none' : 'height: 3000px; width: 10px; flex: none';
    el.appendChild(spacer);
  }, vertical(side));
}

const transformOf = (page: import('@playwright/test').Page) =>
  page.locator(CONTENT).evaluate((el) => (el as HTMLElement).style.transform);

/** Dispatches a touch pointer gesture on the panel's own box: `points` are offsets from its centre. */
async function touchPath(page: import('@playwright/test').Page, points: [number, number][], release = true) {
  return page.evaluate(
    async ({ sel, points, release }) => {
      const el = document.querySelector(sel) as HTMLElement;
      const r = el.getBoundingClientRect(); // rest-ok: places the pointer on an opened, settled drawer
      const x0 = r.x + r.width / 2;
      const y0 = r.y + r.height / 2;
      const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
      const init = (x: number, y: number, up = false) => ({
        bubbles: true, cancelable: true, composed: true, pointerId: 51, pointerType: 'touch',
        isPrimary: true, clientX: x, clientY: y, button: 0, buttons: up ? 0 : 1,
      });
      const seen: string[] = [];
      el.dispatchEvent(new PointerEvent('pointerdown', init(x0, y0)));
      for (const [dx, dy] of points) {
        await sleep(30);
        el.dispatchEvent(new PointerEvent('pointermove', init(x0 + dx, y0 + dy)));
        await sleep(30); // a frame for the forwarded offset
        seen.push(el.style.transform);
      }
      const [lx, ly] = points[points.length - 1];
      if (release) el.dispatchEvent(new PointerEvent('pointerup', init(x0 + lx, y0 + ly, true)));
      return seen;
    },
    { sel: CONTENT, points, release },
  );
}

for (const side of ['bottom', 'top', 'left', 'right'] as const) {
  test.describe(`${side} drawer axis lock`, () => {
    test('a cross-axis scroll with a small wobble along the dismiss axis never moves the panel', async ({ page }) => {
      await open(page, side);
      const v = vertical(side);
      // Scrolling the cross axis: big steps across, +-3px of wobble along the dismiss axis (under the
      // 6px lock distance, and then beaten by the cross movement).
      const pts: [number, number][] = [];
      for (let i = 1; i <= 10; i++) {
        const cross = -12 * i;
        const wobble = (i % 2 ? 3 : -3);
        pts.push(v ? [cross, wobble] : [wobble, cross]);
      }
      const seen = await touchPath(page, pts);
      expect(seen.every((t) => !t.includes('translate'))).toBe(true);
      await page.waitForTimeout(700);
      await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
      expect(await transformOf(page)).toBe('');
    });

    test('a gesture that starts cross-axis never drags, even if it later turns toward the closing edge', async ({ page }) => {
      await open(page, side);
      const v = vertical(side);
      const [ux, uy] = TOWARD_CLOSE[side];
      const pts: [number, number][] = [v ? [-10, 2] : [2, -10]];
      for (let i = 1; i <= 8; i++) pts.push([(v ? -10 : 2) + ux * 30 * i, (v ? 2 : -10) + uy * 30 * i]);
      const seen = await touchPath(page, pts);
      expect(seen.every((t) => !t.includes('translate'))).toBe(true);
      await page.waitForTimeout(700);
      await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
    });

    test('movement under the lock distance does not move the panel', async ({ page }) => {
      await open(page, side);
      const [ux, uy] = TOWARD_CLOSE[side];
      const seen = await touchPath(page, [[ux * 2, uy * 2], [ux * 5, uy * 5]]);
      expect(seen.every((t) => !t.includes('translate'))).toBe(true);
      await page.waitForTimeout(700);
      await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
    });

    test('a clear swipe toward the closing edge dismisses it', async ({ page }) => {
      await open(page, side);
      const [ux, uy] = TOWARD_CLOSE[side];
      // A little cross-axis wobble is fine as long as the dismiss axis wins.
      const pts: [number, number][] = [];
      for (let i = 1; i <= 8; i++) pts.push([ux * 60 * i + (i % 2) * uy * 4, uy * 60 * i + (i % 2) * ux * 4]);
      const seen = await touchPath(page, pts);
      expect(seen.some((t) => t.includes('translate'))).toBe(true);
      await expect(page.locator(ROOT)).toHaveCount(0);
    });

    test('real touch scroll with sideways wobble (CDP): the content scrolls, the panel stays put', async ({ page, browserName }) => {
      test.skip(browserName !== 'chromium', 'CDP touch input is Chromium only');
      await open(page, side);
      const v = vertical(side);
      const box = (await page.locator(CONTENT).boundingBox())!; // rest-ok: opened and settled
      const cdp = await page.context().newCDPSession(page);
      const x0 = box.x + box.width / 2;
      const y0 = box.y + box.height / 2;
      const send = (type: string, x?: number, y?: number) =>
        cdp.send('Input.dispatchTouchEvent', { type, touchPoints: type === 'touchEnd' ? [] : [{ x: x!, y: y! }] });
      await send('touchStart', x0, y0);
      const styles: string[] = [];
      for (let i = 1; i <= 12; i++) {
        const cross = -15 * i;
        const wobble = i % 2 ? 4 : -4;
        await send('touchMove', v ? x0 + cross : x0 + wobble, v ? y0 + wobble : y0 + cross);
        await page.waitForTimeout(25);
        styles.push(await transformOf(page));
      }
      await send('touchEnd');
      expect(styles.every((t) => !t.includes('translate'))).toBe(true);
      await expect(page.locator(ROOT)).toHaveAttribute('data-state', 'open');
      const scrolled = await page.locator(CONTENT).evaluate((el, v) => (v ? el.scrollLeft : el.scrollTop), v);
      expect(scrolled).toBeGreaterThan(0);
    });
  });
}
