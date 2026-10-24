import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

/**
 * Backlog rows 344 and 353: an element's initial pose must be decided during render, never after
 * the first paint.
 *
 * 344: opening a chart tab by client-side navigation mounts a client-created chart. It renders
 * `data-animate="pending"`, the entrance's start pose, from its first frame (rendering the final
 * drawing and letting an effect set `data-animate` would restart it from zero: full -> gone -> animate). The recorder samples every `requestAnimationFrame` after the click and
 * keeps, per chart svg, the signature of its first painted frame; that frame must already be the
 * collapsed pose, never the final one the chart ends up in.
 *
 * 353: the Carousel's Next arrow was server-rendered `disabled` (slide count unknown until the
 * items register), which would paint it at 0.5 opacity and fade it to 1 after hydration. Its first frame
 * is the enabled pose, and its opacity is 1 on every sampled frame.
 *
 * Functional (what is painted on every frame), so a debug build is fine.
 */

const NAV = { timeout: 20 * 60 * 1000 };
const KINDS = ["area", "bar", "line", "pie", "radar", "radial"] as const;

declare global {
  interface Window {
    __poseRec: { frames: Record<string, string[]>; attrs: Record<string, string[]> };
    __poseStop: () => void;
  }
}

/** Start sampling every frame: chart signature + data-animate per svg, in document order. */
async function startChartRecorder(page: Page) {
  await page.evaluate(() => {
    const rec: Window["__poseRec"] = { frames: {}, attrs: {} };
    window.__poseRec = rec;
    let stop = false;
    window.__poseStop = () => {
      stop = true;
    };
    const sig = (svg: Element) => {
      const css = (sel: string, prop: string) => {
        const el = svg.querySelector(sel);
        return el ? (getComputedStyle(el) as unknown as Record<string, string>)[prop] : "-";
      };
      return [
        css('[data-slot="chart-bar"]', "transform"),
        css('[data-slot="chart-series"]', "clipPath"),
        css('[data-kind="radar"] [data-series] > *', "transform"),
        css('[data-slot="chart-sweep-mask"] path', "strokeDasharray"),
        css('[data-slot="chart-arc-labels"]', "opacity"),
      ].join("|");
    };
    const ids = new WeakMap<Element, string>();
    let n = 0;
    const tick = () => {
      if (stop) return;
      for (const svg of Array.from(document.querySelectorAll('[data-slot="chart-svg"]'))) {
        // Only charts that appear AFTER the click: the old tab's charts were recorded as "old".
        if (!ids.has(svg)) ids.set(svg, `c${n++}@${document.querySelector('[data-slot="chart-svg"]') === svg ? 0 : 1}`);
        const id = ids.get(svg)!;
        (rec.frames[id] ??= []).push(sig(svg));
        (rec.attrs[id] ??= []).push(svg.getAttribute("data-animate") ?? "none");
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}

for (const kind of KINDS) {
  test(`${kind} tab opened by client-side navigation: the first frame is the entrance's start pose`, async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    const from = kind === "bar" ? "area" : "bar";
    await gotoHydrated(page, `${BASE_URL}/charts/${from}/`, NAV);
    // Let the source tab's own entrance finish so the recorder only sees the new tab's charts.
    await page.waitForTimeout(1500);
    await startChartRecorder(page);
    // Ids of charts alive before the click are excluded by recording their count first.
    const before = await page.evaluate(() => document.querySelectorAll('[data-slot="chart-svg"]').length);
    await page.locator(`a.dx-charts-tab[href*="/charts/${kind}/"]`).first().click();
    await expect(page).toHaveURL(new RegExp(`/charts/${kind}/`));
    await page.waitForTimeout(2500);
    const rec = await page.evaluate(() => {
      window.__poseStop();
      return window.__poseRec;
    });
    // The new tab's first card is the first svg that carries the pending/true attribute stream and
    // whose recording began after the click: pick the svg ids not present in the first `before`.
    const ids = Object.keys(rec.frames);
    const fresh = ids.slice(before);
    expect(fresh.length, "new charts were mounted by the navigation").toBeGreaterThan(0);
    const id = fresh[0];
    const frames = rec.frames[id];
    const attrs = rec.attrs[id];
    const final = frames[frames.length - 1];
    expect(attrs[0], `first frame data-animate of ${id}: ${attrs.slice(0, 6)}`).toBe("pending");
    expect(frames[0], "the first painted frame is not the final drawing").not.toBe(final);
    // Once the entrance has played the chart rests in its final drawing, and never goes back.
    expect(attrs[attrs.length - 1]).toBe("true");
    const firstFinal = frames.findIndex((f) => f === final);
    expect(firstFinal).toBeGreaterThan(0);
    expect(frames.slice(firstFinal).every((f) => f === final), "no collapse after the final drawing").toBe(true);
  });
}

test("carousel: Next is enabled at opacity 1 in the server HTML and on every frame after hydration", async ({
  page,
  request,
}) => {
  const url = `${BASE_URL}/component/carousel/`;
  // SSR markup (what JS-off sees): the arrow is not disabled.
  const html = await (await request.get(url)).text();
  const nextTag = html.match(/<button[^>]*aria-label="Next slide"[^>]*>/);
  expect(nextTag, "Next arrow is in the server HTML").not.toBeNull();
  expect(nextTag![0]).not.toMatch(/\bdisabled\b/);

  // Sample the first frames of a hydrating document.
  await page.addInitScript(() => {
    const samples: { opacity: number; disabled: boolean }[] = [];
    (window as unknown as { __nextSamples: typeof samples }).__nextSamples = samples;
    const tick = () => {
      const el = document.querySelector('[aria-label="Next slide"]') as HTMLButtonElement | null;
      if (el) samples.push({ opacity: Number(getComputedStyle(el).opacity), disabled: el.disabled });
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  await gotoHydrated(page, url, NAV);
  await page.waitForTimeout(1500);
  const samples = await page.evaluate(
    () => (window as unknown as { __nextSamples: { opacity: number; disabled: boolean }[] }).__nextSamples,
  );
  expect(samples.length).toBeGreaterThan(5);
  const bad = samples.filter((s) => s.opacity !== 1 || s.disabled);
  expect(bad.length, `frames where the first carousel's Next was dimmed or disabled: ${JSON.stringify(bad.slice(0, 3))}`).toBe(0);
});
