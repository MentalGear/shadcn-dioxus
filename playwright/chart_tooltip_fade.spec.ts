import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

/**
 * The chart tooltip fades in and out on the theme's floating tokens (backlog row 472).
 *
 * Read from computed style only: `transition-duration` is the token's value, and `opacity` is 0
 * while the tooltip is closed and 1 once it is open. No timing is measured. Under
 * `prefers-reduced-motion: reduce` the duration is the theme's reduced token (0.01 ms).
 */

const URL = `${BASE_URL}/component/chart_tooltip/`;

/** A CSS time (`0.15s`, `150ms`, `1e-05s`, `0.15s, 0.15s`) as milliseconds, one number per entry. */
function timesMs(value: string): number[] {
  return value.split(",").map((part) => {
    const t = part.trim();
    const n = parseFloat(t);
    return t.endsWith("ms") ? n : n * 1000;
  });
}

/** Round to 3 decimals so `0.15s` (150.00000000000003 ms) compares equal to `150ms`. */
const round = (ms: number) => Math.round(ms * 1000) / 1000;

/** The value a theme token resolves to on `:root`, in milliseconds. */
async function readTokenMs(page: Page, token: string): Promise<number> {
  const raw = await page.evaluate(
    (name) => getComputedStyle(document.documentElement).getPropertyValue(name).trim(),
    token,
  );
  const [ms] = timesMs(raw);
  if (!Number.isFinite(ms)) throw new Error(`${token} did not resolve to a time: "${raw}"`);
  return round(ms);
}

function tooltipOf(page: Page): Locator {
  return page.locator('#component-preview-frame [data-slot="chart-tooltip"]').first();
}

async function gotoChart(page: Page) {
  await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
}

for (const reduced of [false, true]) {
  test.describe(reduced ? "chart tooltip fade (reduced motion)" : "chart tooltip fade", () => {
    test.beforeEach(async ({ page }) => {
      await page.emulateMedia({ reducedMotion: reduced ? "reduce" : "no-preference" });
    });

    test("closed at rest with opacity 0, and its transition uses the floating token", async ({ page }) => {
      await gotoChart(page);
      const tooltip = tooltipOf(page);
      // The main demo opens its tooltip on load (defaultIndex). Enter the chart, then leave it, to close it.
      await page.locator('#component-preview-frame [data-slot="chart-hit-band"]').first().hover();
      await page.mouse.move(0, 0);
      await expect(tooltip).toHaveAttribute("data-state", "closed");

      const expectedMs = await readTokenMs(
        page,
        reduced ? "--dx-motion-duration-reduced" : "--dx-floating-duration",
      );
      await expect(tooltip).toHaveCSS("opacity", "0");

      const durations = await tooltip.evaluate((el) => getComputedStyle(el).transitionDuration);
      for (const ms of timesMs(durations)) {
        expect(round(ms)).toBeCloseTo(expectedMs, 3);
      }
    });

    test("opens to opacity 1 on hover and fades back out when the pointer leaves", async ({ page }) => {
      await gotoChart(page);
      const tooltip = tooltipOf(page);
      await page.locator('#component-preview-frame [data-slot="chart-hit-band"]').first().hover();
      await expect(tooltip).toHaveAttribute("data-state", "open");
      await expect(tooltip).toHaveCSS("opacity", "1");

      await page.mouse.move(0, 0);
      await expect(tooltip).toHaveAttribute("data-state", "closed");
      // The fade-out finishes at `display: none`; the element is gone once it has played.
      await expect(tooltip).toBeHidden();
    });
  });
}
