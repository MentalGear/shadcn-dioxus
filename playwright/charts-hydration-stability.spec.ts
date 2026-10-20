/**
 * CHARTS HYDRATION STABILITY (dev-docs/backlog.md row 215): the `/charts/` gallery, prerendered, is the same page
 * once the wasm has hydrated it -- no box moves or resizes when the page goes from the static HTML to the live app.
 *
 * Row 204's layout-stability spec could not include `/charts/`: its charts re-measured at hydration, and 120-630
 * boxes changed at every width. Three causes, all of one class, "the server render lays out from an assumption
 * that the first measured render then corrects":
 *
 *   1. The chart svg's height. The server draws a chart at an assumed width (369 px) and the svg scaled with its
 *      viewBox ratio; the measured render uses the real width and a rounded height, and with a legend the 28 px
 *      reserved for it did not scale -- the svg (and every card around it) changed height by 0.4-3 px.
 *      Now the box is CSS alone, the same declaration in every render (`chart.rs`, `svg_style`).
 *   2. The time-range select. The prerendered trigger has no selected label yet ("Select an option") and is as wide
 *      as that; hydration adds the label and the trigger, the title beside it and the whole card header re-flowed.
 *      The demos give the trigger a fixed width, as shadcn's does (`w-[160px]`).
 *   3. The drawing itself: a stand-in at the assumed width, replaced by the measured one. Its text was a different
 *      size (the svg scales uniformly) and a different set of ticks.
 *
 * What this spec does about the drawing: nothing, on purpose. The marks, ticks and labels INSIDE a chart's svg
 * are a function of the measured width, which no prerender can know; they are excluded here (and the stand-in's
 * text is held back by the stylesheet until the measured render replaces it). Every OTHER box on the page, and
 * the svg root itself, must not change.
 *
 * Excluded, with the reason:
 *   - descendants of `svg[data-slot="chart-svg"]` (the drawing, above);
 *   - `select[data-slot="select-native"]`: the Select's visually hidden native twin (`opacity: 0`, absolutely
 *     positioned, `pointer-events: none`) gets its `<option>`s at hydration, which changes its own width;
 *   - `.dx-chart-tooltip` and its content: a tooltip opened by `default_index` (the tooltip gallery) is anchored
 *     at a data point of the drawing, so it follows the stand-in's geometry until the measured one replaces it;
 *   - the label `span` inside a `.dx-select-trigger`: the Select primitive prerenders the placeholder ("Select an
 *     option") and writes the selected label at hydration, so the TEXT's own box differs (the trigger, fixed-width,
 *     does not). That is the Select's gap, not the chart's (open item in the row 215 notes).
 *
 * The method is row 204's: record the boxes of every element in `#main-content` on EVERY animation frame from the
 * first one (an init script, so the prerendered frame is included), hold the wasm back until the prerendered page
 * has been painted (+200 ms) so the prerendered frame is on screen at every width, and fail when any box changes
 * after the first paint and the end of the parse (+20 ms) -- through hydration and the 1.5 s after it. A box
 * counts as changed when x, y, width or height differs by more than 0.5 px.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8145 npx playwright test --config=ssg.local.config.ts \
 *     charts-hydration-stability.spec.ts --workers=2
 *
 * `CH_STEP=8` is the full 8 px sweep of `/charts/`; `CH_WIDTHS=480,1280` runs just those widths; `CH_DUMP=1` prints
 * every late box event of a failing width. Chromium only. Debug builds are fine: this counts box changes, not
 * milliseconds.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";

type Box = { x: number; y: number; w: number; h: number };
type BoxEvent = { t: number; f: number; id: number; cls: string; from: Box; to: Box };

/** Installed before any page script, so the very first frame is recorded. Reads boxes only. */
const RECORDER = `
(() => {
  const rec = { events: [], frames: 0, parsedAt: -1, charts: 0 };
  window.__chartsRec = rec;
  const prev = new Map();
  const ids = new Map();
  const r = (n) => Math.round(n * 10) / 10;
  const skipped = (e) =>
    e.closest('svg[data-slot="chart-svg"]') !== e && e.closest('svg[data-slot="chart-svg"]') !== null ||
    e.closest('select[data-slot="select-native"]') !== null ||
    e.closest('.dx-chart-tooltip') !== null ||
    (e.tagName === 'SPAN' && e.parentElement && e.parentElement.classList.contains('dx-select-trigger'));
  function tick(now) {
    rec.frames++;
    const main = document.getElementById('main-content');
    if (main) {
      let n = 0;
      for (const e of main.querySelectorAll('*')) {
        if (skipped(e)) continue;
        if (++n > 4000) break;
        const b = e.getBoundingClientRect();
        const cur = { x: r(b.x), y: r(b.y + window.scrollY), w: r(b.width), h: r(b.height) };
        const p = prev.get(e);
        if (p && (Math.abs(p.x - cur.x) > 0.5 || Math.abs(p.y - cur.y) > 0.5 || Math.abs(p.w - cur.w) > 0.5 || Math.abs(p.h - cur.h) > 0.5)) {
          if (!ids.has(e)) ids.set(e, ids.size);
          rec.events.push({ t: Math.round(now), f: rec.frames, id: ids.get(e), cls: e.tagName.toLowerCase() + '.' + String(e.getAttribute('class') || e.getAttribute('data-slot') || '').split(' ')[0].slice(0, 40), from: p, to: cur });
        }
        prev.set(e, cur);
      }
    }
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);
  document.addEventListener('DOMContentLoaded', () => { rec.parsedAt = Math.round(performance.now()); });
})();
`;

const KEY_WIDTHS = [
  320, 360, 390, 479, 480, 481, 639, 640, 641, 767, 768, 769, 820, 834, 1023, 1024, 1025, 1100, 1180, 1280, 1366, 1440,
];
function sweep(step: number): number[] {
  const out: number[] = [];
  for (let w = 320; w <= 1440; w += step) out.push(w);
  return out;
}
const FULL = !!process.env.CH_STEP;
const STEP = Number(process.env.CH_STEP ?? 40);
const ALL_WIDTHS = [...new Set([...sweep(STEP), ...KEY_WIDTHS])].sort((a, b) => a - b);
const SOME_WIDTHS = [360, 390, 640, 768, 1024, 1280, 1440];
const HEIGHT = 800;
const ONLY = (process.env.CH_WIDTHS ?? "").split(",").filter(Boolean).map(Number);

const GROUPS: { route: string; scheme: "light" | "dark"; widths: number[] }[] = [
  { route: "/charts/", scheme: "light", widths: ALL_WIDTHS },
  { route: "/charts/", scheme: "dark", widths: FULL ? ALL_WIDTHS : SOME_WIDTHS },
  // The other tabs: fixed-height charts, polar charts, charts with a legend above and below, the tooltip gallery.
  ...["bar", "line", "pie", "radar", "radial", "tooltip"].map((tab) => ({
    route: `/charts/${tab}/`,
    scheme: "light" as const,
    widths: FULL ? ALL_WIDTHS : SOME_WIDTHS,
  })),
];

const fmt = (b: Box) => `${b.x},${b.y} ${b.w}x${b.h}`;

for (const { route, scheme, widths } of GROUPS) {
  test(`charts hydration stability ${route} ${scheme} (${widths.length} widths)`, async ({ browser }) => {
    test.skip(browser.browserType().name() !== "chromium", "frame-by-frame box recording is measured in Chromium");
    test.setTimeout(45 * 60 * 1000);
    const context = await browser.newContext({ viewport: { width: widths[0], height: HEIGHT }, colorScheme: scheme });
    await context.addCookies([{ name: "dx_theme", value: scheme, url: BASE_URL }]);
    const page = await context.newPage();
    await page.addInitScript(RECORDER);
    // Hydration waits until the prerendered page has been painted (+200 ms): the slow-wasm case, and the only one
    // in which every prerendered-vs-hydrated difference is on screen. Without it a fast machine hydrates before its
    // first paint and the same defect passes at that width.
    await page.route("**/*.wasm", async (route) => {
      for (let i = 0; i < 400 && !(await page.evaluate(() => performance.getEntriesByType("paint").length).catch(() => 0)); i++) await page.waitForTimeout(50);
      await page.waitForTimeout(200);
      await route.continue().catch(() => {});
    });
    const failures: string[] = [];

    for (const width of ONLY.length ? ONLY : widths) {
      await page.setViewportSize({ width, height: HEIGHT });
      // goto-ok: observes the page from commit to hydration, so it must not wait for the flag first
      await page.goto(`${BASE_URL}${route}`, { waitUntil: "commit" });
      await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: 120_000 });
      await page.evaluate(() => document.fonts.ready.then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
      await page.waitForTimeout(1500); // no input: whatever still moves by itself shows up in this window
      const { events, paint, parsedAt, charts, measured } = await page.evaluate(() => ({
        events: (window as any).__chartsRec.events as BoxEvent[],
        parsedAt: (window as any).__chartsRec.parsedAt as number,
        paint: performance.getEntriesByType("paint").map((p) => ({ name: p.name, t: Math.round(p.startTime) })),
        charts: document.querySelectorAll('#main-content svg[data-slot="chart-svg"]').length,
        measured: document.querySelectorAll('#main-content svg[data-slot="chart-svg"][data-measured="true"]').length,
      }));
      // The spec must see what it is about: every chart on the page was hydrated and measured.
      expect(charts, `${route} ${width}px: no chart on the page`).toBeGreaterThan(0);
      expect(measured, `${route} ${width}px: charts that never reached their measured render`).toBe(charts);
      const firstPaint = (paint.find((p) => p.name === "first-contentful-paint") ?? paint[0])?.t;
      expect(firstPaint, `${route} ${width}px: no paint entry`).toBeDefined();
      const settledAt = Math.max(firstPaint!, parsedAt);
      const late = events.filter((e) => e.t > settledAt + 20);
      if (late.length === 0) continue;
      if (process.env.CH_DUMP) console.log(`${route} ${scheme} ${width}px`, JSON.stringify(late.slice(0, 60).map((e) => [e.t, e.f, e.cls, fmt(e.from), fmt(e.to)])));
      const perBox = new Map<number, BoxEvent[]>();
      for (const e of late) perBox.set(e.id, [...(perBox.get(e.id) ?? []), e]);
      const oscillating = [...perBox.values()].filter((v) => v.length > 1).length;
      const classes = [...new Set(late.map((e) => e.cls))].slice(0, 5).join(" | ");
      const first = late[0];
      failures.push(
        `${width}px: ${perBox.size} box(es) changed after first paint (${oscillating} more than once), ` +
          `e.g. "${first.cls}" ${fmt(first.from)} -> ${fmt(first.to)}; classes: ${classes}`,
      );
    }
    await context.close();
    expect(failures, `${route} (${scheme}): the page changed after its first paint at ${failures.length} of ${widths.length} widths`).toEqual([]);
  });
}
