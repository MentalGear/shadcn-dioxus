import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { installFrameRecorder, readFrames } from "./frame-recorder";

/**
 * The chart tooltip fades in and out on the theme's floating tokens (backlog row 472).
 *
 * Read from computed style only: `transition-duration` is the token's value, and `opacity` is 0
 * while the tooltip is closed and 1 once it is open. Under `prefers-reduced-motion: reduce` the
 * duration is the theme's reduced token (0.01 ms).
 *
 * The behaviour cases sample `opacity` on every animation frame (`installFrameRecorder`, the
 * deterministic recorder, so these are not `@timing` cases: nothing bounds a duration, they ask
 * only which values a frame showed) after a real `mouse.move`: the fade passes through a value
 * strictly between 0 and 1 on enter and on leave, symmetrically, and under reduced motion in
 * neither direction.
 *
 * The mechanism case stands in for a Firefox/Safari run this container cannot do: an entry that
 * starts from `display: none` (or relies on `@starting-style`) fades out but not in on an engine
 * that does not animate from `display: none`. The closed tooltip is therefore asserted to be
 * rendered (`display` is not `none`) and `visibility: hidden`, and the stylesheet to carry no
 * `@starting-style` rule for it, so the entry interpolates from its own opacity in every engine.
 */

type Frame = { state: string | null; opacity: number; visibility: string; display: string };
const FRAMES = "__tooltipFrames";

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

async function recordTooltipFrames(page: Page) {
  await installFrameRecorder<Frame>(page, FRAMES, () => {
    const el = document.querySelector('#component-preview-frame [data-slot="chart-tooltip"]');
    if (!el) return null;
    const c = getComputedStyle(el);
    return {
      state: el.getAttribute("data-state"),
      opacity: Number(c.opacity),
      visibility: c.visibility,
      display: c.display,
    };
  });
}

/** The centre of the nth hit band, scrolled into view. */
async function bandCentre(page: Page, nth: number) {
  const band = page.locator('#component-preview-frame [data-slot="chart-hit-band"]').nth(nth);
  await band.scrollIntoViewIfNeeded();
  const box = await band.boundingBox();
  if (!box) throw new Error("hit band has no box");
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

/** Close the tooltip with a pointer visit, then wait (on frames, bounded) until it is gone (closed and transparent). */
async function closeTooltip(page: Page) {
  const c = await bandCentre(page, 0);
  await page.mouse.move(c.x, c.y);
  await page.mouse.move(0, 0);
  await expect
    .poll(async () => {
      const last = (await readFrames<Frame>(page, FRAMES)).at(-1);
      return last?.state === "closed" && last.opacity === 0;
    })
    .toBe(true);
}

/** Frames recorded after `from`, once the last of them satisfies `done` (polled on frames, bounded). */
async function framesUntil(page: Page, from: number, done: (last: Frame) => boolean): Promise<Frame[]> {
  await expect
    .poll(async () => {
      const all = await readFrames<Frame>(page, FRAMES);
      const last = all.at(-1);
      return all.length > from && !!last && done(last);
    })
    .toBe(true);
  return (await readFrames<Frame>(page, FRAMES)).slice(from);
}

const between = (f: Frame) => f.opacity > 0 && f.opacity < 1;

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

    test("opacity passes through intermediate values on enter and on leave, none under reduced motion", async ({ page }) => {
      await recordTooltipFrames(page);
      await gotoChart(page);
      await closeTooltip(page);
      const target = await bandCentre(page, 2);

      let from = (await readFrames<Frame>(page, FRAMES)).length;
      await page.mouse.move(target.x, target.y);
      const entered = await framesUntil(page, from, (f) => f.state === "open" && f.opacity === 1);
      expect(entered.at(-1)?.opacity).toBe(1);
      if (reduced) expect(entered.filter(between)).toHaveLength(0);
      else expect(entered.filter(between).length).toBeGreaterThan(0);

      from = (await readFrames<Frame>(page, FRAMES)).length;
      await page.mouse.move(0, 0);
      const left = await framesUntil(page, from, (f) => f.state === "closed" && f.opacity === 0);
      expect(left.at(-1)?.opacity).toBe(0);
      if (reduced) expect(left.filter(between)).toHaveLength(0);
      else expect(left.filter(between).length).toBeGreaterThan(0);
    });

    test("moving between points while the tooltip is shown does not restart the fade", async ({ page }) => {
      await recordTooltipFrames(page);
      await gotoChart(page);
      await closeTooltip(page);
      const a = await bandCentre(page, 1);
      await page.mouse.move(a.x, a.y);
      await framesUntil(page, 0, (f) => f.state === "open" && f.opacity === 1);

      const from = (await readFrames<Frame>(page, FRAMES)).length;
      const b = await bandCentre(page, 4);
      await page.mouse.move(b.x, b.y, { steps: 6 });
      // The tooltip moved to the new point (still open); give the frames that follow the move a bounded window.
      const after = await framesUntil(page, from + 10, (f) => f.state === "open");
      expect(after.every((f) => f.state === "open" && f.opacity === 1 && f.visibility === "visible")).toBe(true);
    });

    test("closed: rendered and visibility hidden, and no @starting-style rule for the tooltip", async ({ page }) => {
      await gotoChart(page);
      const tooltip = tooltipOf(page);
      await page.locator('#component-preview-frame [data-slot="chart-hit-band"]').first().hover();
      await page.mouse.move(0, 0);
      await expect(tooltip).toHaveAttribute("data-state", "closed");
      await expect(tooltip).toHaveCSS("visibility", "hidden");
      // Rendered, not `display: none`: the entry never starts from "was not rendered".
      await expect(tooltip).not.toHaveCSS("display", "none");
      // The tooltip is out of the accessibility tree and reached by neither pointer nor keyboard.
      await expect(tooltip).toHaveAttribute("aria-hidden", "true");
      await expect(tooltip).toHaveCSS("pointer-events", "none");

      const startingStyleRules = await page.evaluate(() => {
        const hits: string[] = [];
        const walk = (rules: CSSRuleList) => {
          for (const rule of Array.from(rules)) {
            if (rule.cssText.startsWith("@starting-style") && rule.cssText.includes("chart-tooltip")) hits.push(rule.cssText);
            const inner = (rule as CSSGroupingRule).cssRules;
            if (inner) walk(inner);
          }
        };
        for (const sheet of Array.from(document.styleSheets)) {
          try {
            walk(sheet.cssRules);
          } catch {
            /* a cross-origin sheet is unreadable and not ours */
          }
        }
        return hits;
      });
      expect(startingStyleRules).toEqual([]);
    });
  });
}
