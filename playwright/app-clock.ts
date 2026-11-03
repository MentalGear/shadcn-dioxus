/**
 * A page clock the test drives, for a spec whose verdict waits on an app timer (a toast's countdown, a tooltip's
 * delay, an autoplay tick, a debounce).
 *
 * The app's timers are `setTimeout` calls (`gloo_timers`, `crate::sleep::sleep`; there is no JS `setInterval`), so
 * Playwright's `page.clock` controls them: the moment a timer fires is the test's, not the machine's. A spec that
 * waits `waitForTimeout(<app duration>)` or polls for an element a timer removes reads the timer against the wall
 * clock, and three workers of load stretch the gap between the polls until the state it expects has come and gone.
 * `scripts/check-timing-tags.sh` fails such a wait unless the file uses `page.clock` or the line says
 * `timing-bound-ok: <reason>`.
 *
 *     await gotoHydrated(page, url);
 *     await pauseAppClock(page);          // timers from here on are the test's
 *     await page.getByRole("button").click();
 *     await page.clock.runFor(500);       // renders run on (fake) animation frames, so a click needs time to show
 *     await page.clock.runFor(2_600);     // the toast's 3 s countdown ends here, on every machine
 *     await page.clock.resume();          // CSS transitions run in real time, so hand the clock back to follow them
 *
 * Install it AFTER the page is hydrated (`page.clock.install` replaces the timer functions of the running page, so
 * the page's own hydration and the helpers' waits are not paused), and before the click that starts the timer under test.
 * The page's own `requestAnimationFrame` is faked as well; a sampler that must keep the real frames reads
 * `window.__nativeRaf`, which this saves before the install.
 */
import type { Page } from "@playwright/test";

export async function pauseAppClock(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as unknown as { __nativeRaf: typeof requestAnimationFrame }).__nativeRaf = window.requestAnimationFrame.bind(window);
  });
  await page.clock.install();
  await page.clock.pauseAt(new Date(Date.now() + 1_000));
}
