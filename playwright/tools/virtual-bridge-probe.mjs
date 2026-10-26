// Scroll-throughput probe for the windowed engine's DOM bridge (backlog row 434, lane 2D).
//
// What it measures, per subject and round: a 2 s programmatic "fling" (one `scrollTop` step per animation
// frame, sweeping the whole container back and forth, so the container fires one `scroll` event per frame the
// way a real fling does) while recording
//   - frame deltas (requestAnimationFrame) -> frames over 16.7 / 20 / 33 / 50 ms, p50 / p95 / max,
//   - Long Tasks (>= 50 ms) -> count and total, from a PerformanceObserver installed before the page's scripts,
//   - main-thread task time per frame (CDP `Performance.getMetrics` TaskDuration, as `frame-cost.ts`),
//   - scroll events delivered, rows mounted at the end (a sanity check that the window followed the scroll).
// It also reports `calibrationMs` (how fast the box ran a fixed workload) so a result can be read against the load
// it was measured under. Serve the build before and the build after (a RELEASE build each) and compare the per-subject minimum over the rounds (load only adds time).
//
//   node playwright/tools/virtual-bridge-probe.mjs --bases before=http://127.0.0.1:8091,after=http://127.0.0.1:8092 \\
//     --rounds 7 --out probe.json
// With two builds the rounds are interleaved (ABBA), so both see the same machine load.
import { createRequire } from "node:module";
import { writeFileSync } from "node:fs";

const require = createRequire(import.meta.url);
const { chromium } = require("@playwright/test");
const { sandboxLaunchOptions } = require("../sandbox-chromium.cjs");

const args = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, arg, i, all) => {
    if (arg.startsWith("--")) pairs.push([arg.slice(2), all[i + 1]]);
    return pairs;
  }, []),
);
// `--bases before=http://127.0.0.1:8091,after=http://127.0.0.1:8092`: the builds to compare, run interleaved.
const bases = Object.fromEntries(
  (args.bases ?? "run=http://127.0.0.1:8090").split(",").map((pair) => {
    const [label, url] = [pair.slice(0, pair.indexOf("=")), pair.slice(pair.indexOf("=") + 1)];
    return [label, url.replace(/\/$/, "")];
  }),
);
const rounds = Number(args.rounds ?? 7);
const flingMs = Number(args.fling ?? 2000);

const SUBJECTS = [
  { name: "virtual_list", route: "/component/virtual_list/", selector: ".dx-virtual-list-container" },
  { name: "virtual_grid", route: "/component/virtual_grid/", selector: '[data-virtual-grid="grid"][data-virtualize="windowed"]' },
  { name: "virtual_grid (wrap)", route: "/component/virtual_grid/", selector: '[data-virtual-grid="wrap"][data-virtualize="windowed"]' },
  { name: "virtual_grid (justified)", route: "/component/virtual_grid/", selector: '[data-virtual-grid="justified"]' },
  { name: "data_table (large)", route: "/component/data_table/", selector: "#component-preview-frame-large [data-virtual-grid=\"grid\"]" },
];

const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const pct = (xs, p) => [...xs].sort((a, b) => a - b)[Math.min(xs.length - 1, Math.floor(xs.length * p))];

async function calibrationMs(page) {
  return page.evaluate(() => {
    const runs = [];
    for (let run = 0; run < 7; run++) {
      const start = performance.now();
      let x = 0;
      for (let i = 0; i < 20_000_000; i++) x = (x * 31 + i) | 0;
      if (x === 42) console.log(x);
      runs.push(performance.now() - start);
    }
    return runs.sort((a, b) => a - b)[3];
  });
}

const browser = await chromium.launch({ ...sandboxLaunchOptions(), args: ["--enable-precise-memory-info"] });
const result = { bases, rounds, flingMs, subjects: {} };

/** One fling on a fresh page of `base`: the numbers of the module header. */
async function oneRound(subject, base) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  try {
    const page = await context.newPage();
    await page.addInitScript(() => {
      window.__longTasks = [];
      new PerformanceObserver((list) => {
        for (const e of list.getEntries()) window.__longTasks.push({ start: e.startTime, duration: e.duration });
      }).observe({ type: "longtask", buffered: true });
    });
    await page.goto(base + subject.route, { waitUntil: "networkidle" });
    await page.waitForSelector('html[data-hydrated="true"]', { timeout: 30000 });
    const container = page.locator(subject.selector).first();
    if ((await container.count()) === 0) return { error: "container not found" };
    await container.scrollIntoViewIfNeeded();
    const cdp = await context.newCDPSession(page);
    await cdp.send("Performance.enable");
    const taskSeconds = async () => (await cdp.send("Performance.getMetrics")).metrics.find((m) => m.name === "TaskDuration").value;
    const calibration = await calibrationMs(page);
    await page.waitForTimeout(900);
    await page.evaluate(() => (window.__longTasks.length = 0));
    const before = await taskSeconds();
    const run = await container.evaluate(
      (el, ms) =>
        new Promise((resolve) => {
          const deltas = [];
          let scrolls = 0;
          const onScroll = () => scrolls++;
          el.addEventListener("scroll", onScroll, { passive: true });
          const range = Math.max(1, el.scrollHeight - el.clientHeight);
          const step = Math.max(40, Math.round(el.clientHeight / 6));
          let dir = 1;
          let last = performance.now();
          const t0 = last;
          const tick = (now) => {
            deltas.push(now - last);
            last = now;
            const next = el.scrollTop + dir * step;
            if (next > range || next < 0) dir = -dir;
            el.scrollTop = Math.min(Math.max(next, 0), range);
            if (now - t0 < ms) requestAnimationFrame(tick);
            else {
              el.removeEventListener("scroll", onScroll);
              resolve({ deltas: deltas.slice(1), scrolls });
            }
          };
          requestAnimationFrame(tick);
        }),
      flingMs,
    );
    const mainThreadMs = ((await taskSeconds()) - before) * 1000;
    await page.waitForTimeout(700);
    const longTasks = await page.evaluate(() => window.__longTasks.slice());
    const rows = await container.evaluate((el) => el.querySelectorAll("[data-virtual-index], [data-grid-cell]").length);
    const d = run.deltas;
    return {
      calibrationMs: calibration,
      frames: d.length,
      over16_7: d.filter((x) => x > 16.7 + 0.3).length,
      over20: d.filter((x) => x > 20).length,
      over33: d.filter((x) => x > 33).length,
      over50: d.filter((x) => x > 50).length,
      p50: median(d),
      p95: pct(d, 0.95),
      max: Math.max(...d),
      longTasks: longTasks.length,
      longTaskMs: longTasks.reduce((a, b) => a + b.duration, 0),
      mainThreadMsPerFrame: mainThreadMs / d.length,
      scrollEvents: run.scrolls,
      rowsMounted: rows,
    };
  } finally {
    await context.close();
  }
}

const KEYS = ["over16_7", "over20", "over33", "over50", "p95", "max", "longTasks", "longTaskMs", "mainThreadMsPerFrame", "scrollEvents", "rowsMounted"];
const labels = Object.keys(bases);
for (const subject of SUBJECTS) {
  const per = Object.fromEntries(labels.map((l) => [l, []]));
  for (let round = 0; round < rounds; round++) {
    // ABBA: alternate which build goes first so a drifting load does not favour one.
    const order = round % 2 === 0 ? labels : [...labels].reverse();
    for (const l of order) per[l].push(await oneRound(subject, bases[l]));
  }
  result.subjects[subject.name] = Object.fromEntries(
    labels.map((l) => {
      const runs = per[l];
      if (runs.some((r) => r.error)) return [l, { error: runs.find((r) => r.error).error }];
      const col = (k) => runs.map((r) => r[k]);
      return [
        l,
        {
          calibrationMs: median(col("calibrationMs")),
          runs,
          min: Object.fromEntries(KEYS.map((k) => [k, Math.min(...col(k))])),
          median: Object.fromEntries(KEYS.map((k) => [k, median(col(k))])),
        },
      ];
    }),
  );
}
await browser.close();
if (args.out) writeFileSync(args.out, JSON.stringify(result, null, 2));
const f = (n, d = 1) => Number(n).toFixed(d);
for (const [name, byLabel] of Object.entries(result.subjects)) {
  for (const [l, s] of Object.entries(byLabel)) {
    if (s.error) {
      console.log(`${name.padEnd(26)} ${l.padEnd(7)} ${s.error}`);
      continue;
    }
    console.log(
      `${name.padEnd(26)} ${l.padEnd(7)} cal ${f(s.calibrationMs, 0)}ms | min: >16.7ms ${s.min.over16_7}, >33ms ${s.min.over33}, long tasks ${s.min.longTasks} (${f(s.min.longTaskMs, 0)}ms), task ${f(s.min.mainThreadMsPerFrame, 2)}ms/frame, p95 ${f(s.min.p95)}ms | median: >16.7ms ${s.median.over16_7}, >33ms ${s.median.over33}, long tasks ${s.median.longTasks}, task ${f(s.median.mainThreadMsPerFrame, 2)}ms/frame, scroll events ${s.median.scrollEvents}, rows ${s.median.rowsMounted}`,
    );
  }
}
