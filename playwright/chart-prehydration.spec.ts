/**
 * CHART PRE-HYDRATION == HYDRATED GEOMETRY (dev-docs/backlog row 124). The prerendered `/charts/*` pages are loaded
 * twice: once in a JavaScript-disabled context (the static HTML exactly as a visitor sees it before the wasm
 * arrives) and once hydrated (measured render, at rest). Every chart card must have the same box in both, and so must
 * its pieces (the `.dx-chart` box, the svg root, the legend, the card header, a Select trigger), within 0.5 px.
 * Positions inside a card are taken relative to the card, so a cascade from an earlier card cannot hide or fake a
 * difference; the card's own height and width are compared as well. Contents of the svg (marks, ticks, text) are a
 * function of the measured width and are not compared -- only the svg root's box is.
 *
 * Set `CHP_DUMP=1` to print every card's numbers (the row-124 measurement).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

type Box = { x: number; y: number; w: number; h: number };
type Card = { key: string; parts: Record<string, Box> };

const ROUTES = ["/charts/", ...["bar", "line", "pie", "radar", "radial", "tooltip"].map((t) => `/charts/${t}/`)];
const WIDTHS = [1440, 390];
const TOL = 0.5;

const COLLECT = () => {
  const r = (n: number) => Math.round(n * 100) / 100;
  const cards: Card[] = [];
  const root = document.getElementById("main-content")!;
  const owners = new Set<Element>();
  for (const svg of root.querySelectorAll('svg[data-slot="chart-svg"]')) {
    const card = svg.closest('[data-slot="card"]') ?? svg.closest(".dx-chart")!.parentElement!;
    owners.add(card);
  }
  const SELECTORS: Record<string, string> = {
    chart: ".dx-chart",
    svg: 'svg[data-slot="chart-svg"]',
    legend: ".dx-chart-legend",
    header: '[data-slot="card-header"]',
    content: '[data-slot="card-content"]',
    footer: '[data-slot="card-footer"]',
    trigger: ".dx-select-trigger",
  };
  let i = 0;
  for (const card of owners) {
    const c = card.getBoundingClientRect();
    const parts: Record<string, Box> = { card: { x: 0, y: 0, w: r(c.width), h: r(c.height) } };
    for (const [name, sel] of Object.entries(SELECTORS)) {
      card.querySelectorAll(sel).forEach((e, n) => {
        const b = e.getBoundingClientRect();
        parts[`${name}${n}`] = { x: r(b.x - c.x), y: r(b.y - c.y), w: r(b.width), h: r(b.height) };
      });
    }
    const title = card.querySelector('[data-slot="card-title"]')?.textContent?.trim() ?? "";
    cards.push({ key: `#${i++} ${title}`, parts });
  }
  return cards;
};

for (const route of ROUTES) {
  test(`chart prehydration geometry == hydrated ${route}`, async ({ browser }) => {
    test.skip(browser.browserType().name() !== "chromium", "measured in Chromium");
    const report: string[] = [];
    for (const width of WIDTHS) {
      const viewport = { width, height: 900 };
      // Pre-hydration: no JavaScript at all.
      const noJs = await browser.newContext({ viewport, javaScriptEnabled: false });
      const p1 = await noJs.newPage();
      // goto-ok: JavaScript disabled in this context: the page can never hydrate
      await p1.goto(`${BASE_URL}${route}`, { waitUntil: "load" });
      await p1.waitForSelector('svg[data-slot="chart-svg"]');
      const before: Card[] = await p1.evaluate(`(${COLLECT.toString()})()`);
      await noJs.close();

      const js = await browser.newContext({ viewport });
      const p2 = await js.newPage();
      await gotoHydrated(p2, `${BASE_URL}${route}`);
      await p2.waitForSelector('svg[data-slot="chart-svg"][data-measured="true"]');
      await p2.evaluate(() => document.fonts.ready.then(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))));
      await p2.waitForTimeout(800);
      const after: Card[] = await p2.evaluate(`(${COLLECT.toString()})()`);
      await js.close();

      if (process.env.CHP_DUMP) console.log(`${width}px ${route}: ${before.length} cards`, before.filter((c) => c.parts.legend0 || c.parts.trigger0).map((c) => `${c.key} h=${c.parts.card.h}->${after[before.indexOf(c)]?.parts.card.h}${c.parts.legend0 ? ` legend ${c.parts.legend0.h}` : ""}`).join(" | "));
      expect(before.length, `${route} ${width}px: no chart cards`).toBeGreaterThan(0);
      expect(after.length, `${route} ${width}px: card count differs`).toBe(before.length);
      for (let n = 0; n < before.length; n++) {
        const a = before[n], b = after[n];
        const names = new Set([...Object.keys(a.parts), ...Object.keys(b.parts)]);
        for (const part of names) {
          const pa = a.parts[part], pb = b.parts[part];
          if (!pa || !pb) {
            // The Select's visually hidden native twin and the like are not parts; a missing part IS a difference.
            report.push(`${width}px ${route} ${a.key} ${part}: present ${!!pa} -> ${!!pb}`);
            continue;
          }
          for (const k of ["x", "y", "w", "h"] as const) {
            const d = Math.abs(pa[k] - pb[k]);
            if (process.env.CHP_DUMP && d > 0.05) console.log(`${width}px ${route} ${a.key} ${part}.${k} ${pa[k]} -> ${pb[k]}`);
            if (d > TOL) report.push(`${width}px ${route} ${a.key} ${part}.${k}: ${pa[k]} -> ${pb[k]} (${(pb[k] - pa[k]).toFixed(2)})`);
          }
        }
      }
    }
    expect(report, "pre-hydration geometry differs from hydrated").toEqual([]);
  });
}
