#!/usr/bin/env node
// Per-file durations from a Playwright JSON report (backlog row 152a).
//
//   PLAYWRIGHT_JSON_OUTPUT_FILE=/abs/report.json pnpm exec playwright test --config=suite.local.config.ts --reporter=json,line
//   node playwright/tools/durations.mjs /abs/report.json [--write playwright/durations.json [--merge]] [--top 30] [--meta k=v ...]
// --merge: the report is a partial run (a lane's own specs): only those files' entries are replaced.
//
// Prints the slowest files and the time by category; --write stores the per-file table that
// `shard.mjs` balances on and `scripts/check-slow-spec-budget.sh` reads. A file's `seconds` is the SUM of its tests'
// durations (worker time, not wall time): with W workers the suite wall time is about total / W.
import fs from "node:fs";
import path from "node:path";

// Entries are sorted by code point, not localeCompare: the order must not depend on the machine's ICU data, because
// playwright/tools/durations-merge.mjs (the git merge driver, row 400) writes the same order.
const byCodePoint = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const argv = process.argv.slice(2);
const reportPaths = []; while (argv.length && !argv[0].startsWith("--")) reportPaths.push(argv.shift());
const rest = argv; const reportPath = reportPaths[0];
if (!reportPath) { console.error("usage: durations.mjs report.json... [--write out.json] [--top N] [--meta k=v ...]"); process.exit(2); }
let write = null, top = 30, merge = false; const meta = {};
for (let i = 0; i < rest.length; i++) {
  if (rest[i] === "--write") write = rest[++i];
  else if (rest[i] === "--merge") merge = true;
  else if (rest[i] === "--top") top = Number(rest[++i]);
  else if (rest[i] === "--meta") { while (i + 1 < rest.length && /^[\w.-]+=/.test(rest[i + 1])) { const [k, ...v] = rest[++i].split("="); meta[k] = v.join("="); } }
}
// several reports (a chunked run) are one run: their suites are concatenated, wall time is the sum of the chunks
const reports = reportPaths.map((p) => JSON.parse(fs.readFileSync(p, "utf8")));
const report = { suites: reports.flatMap((r) => r.suites), stats: { duration: reports.reduce((a, r) => a + (r.stats?.duration ?? 0), 0) } };
const files = {};
const tests = [];
function visit(suite) {
  for (const spec of suite.specs ?? []) {
    for (const t of spec.tests ?? []) {
      const ms = (t.results ?? []).reduce((a, r) => a + (r.duration ?? 0), 0);
      const f = (spec.file ?? suite.file).replace(/\\/g, "/");
      const e = (files[f] ??= { tests: 0, seconds: 0, passed: 0, failed: 0, skipped: 0, retried: 0 });
      e.tests++; e.seconds += ms / 1000;
      if (t.status === "skipped") e.skipped++; else if (t.status === "unexpected") e.failed++; else e.passed++;
      if ((t.results ?? []).length > 1) e.retried++;
      tests.push({ file: f, title: spec.title, ms, status: t.status });
    }
  }
  for (const s of suite.suites ?? []) visit(s);
}
for (const s of report.suites) visit(s);
for (const e of Object.values(files)) e.seconds = Math.round(e.seconds * 10) / 10;

const category = (f) => {
  if (/^oracle\/tier1/.test(f)) return "oracle tier1 (APG)";
  if (/^oracle\/tier2/.test(f)) return "oracle tier2 (HTML)";
  if (/^oracle\/tier3/.test(f)) return "oracle tier3 (Radix)";
  if (/^oracle\//.test(f)) return "oracle hydration-parity";
  if (/all-routes/.test(f)) return "all-routes";
  if (/layout-stability/.test(f)) return "layout-stability";
  if (/computed-style|theme-|dark-surface|focus-ring|user-select|popup-style/.test(f)) return "computed-style / theme";
  if (/first-paint/.test(f)) return "first-paint";
  if (/carousel/.test(f)) return "carousel family";
  if (/chart|radar|pie|radial|area_|bar_|line_/.test(f)) return "chart family";
  if (/toast/.test(f)) return "toast family";
  return "per-component specs";
};
const cats = {};
for (const [f, e] of Object.entries(files)) { const c = (cats[category(f)] ??= { files: 0, tests: 0, seconds: 0 }); c.files++; c.tests += e.tests; c.seconds += e.seconds; }
const total = Object.values(files).reduce((a, e) => a + e.seconds, 0);
const axe = tests.filter((t) => /a11y|axe/i.test(t.title));
const axeS = axe.reduce((a, t) => a + t.ms, 0) / 1000;

const sorted = Object.entries(files).sort((a, b) => b[1].seconds - a[1].seconds);
console.log(`total test time ${total.toFixed(0)} s over ${tests.length} tests in ${sorted.length} files; wall ${((report.stats?.duration ?? 0) / 1000).toFixed(0)} s`);
console.log(`\nslowest ${top} files:\nseconds  tests  share  file`);
for (const [f, e] of sorted.slice(0, top)) console.log(`${e.seconds.toFixed(0).padStart(7)}  ${String(e.tests).padStart(5)}  ${(100 * e.seconds / total).toFixed(1).padStart(4)}%  ${f}`);
console.log("\nby category:\nseconds  tests  files  share  category");
for (const [c, e] of Object.entries(cats).sort((a, b) => b[1].seconds - a[1].seconds)) console.log(`${e.seconds.toFixed(0).padStart(7)}  ${String(e.tests).padStart(5)}  ${String(e.files).padStart(5)}  ${(100 * e.seconds / total).toFixed(1).padStart(4)}%  ${c}`);
console.log(`\naxe-titled tests: ${axe.length}, ${axeS.toFixed(0)} s (${(100 * axeS / total).toFixed(1)}%)`);
const ts = tests.map((t) => t.ms).sort((a, b) => a - b);
console.log(`median test ${(ts[Math.floor(ts.length / 2)] / 1000).toFixed(2)} s; tests > 10 s: ${ts.filter((x) => x > 10000).length} (${(ts.filter((x) => x > 10000).reduce((a, b) => a + b, 0) / 1000).toFixed(0)} s)`);
const bad = tests.filter((t) => t.status === "unexpected" || t.status === "flaky");
console.log(`unexpected/flaky: ${bad.length}`);
if (write && merge) {
  // A partial run (a lane's own specs): replace only those files' entries, keep the rest and the budget fields.
  const old = JSON.parse(fs.readFileSync(write, "utf8"));
  const merged = { ...old.files, ...files };
  const sum = Object.values(merged).reduce((a, e) => a + e.seconds, 0);
  const out = { meta: { ...old.meta, ...meta, tests: Object.values(merged).reduce((a, e) => a + e.tests, 0), files: Object.keys(merged).length, total_seconds: Math.round(sum) },
    files: Object.fromEntries(Object.entries(merged).sort(([a], [b]) => byCodePoint(a, b))) };
  fs.writeFileSync(write, JSON.stringify(out, null, 1) + "\n");
  console.log(`merged ${Object.keys(files).length} files into ${write} (total ${Math.round(sum)} s, budget ${old.meta.budget_seconds ?? "unset"} s)`);
} else if (write) {
  const out = { meta: { ...meta, tests: tests.length, files: sorted.length, total_seconds: Math.round(total) }, files: Object.fromEntries(Object.entries(files).sort(([a], [b]) => byCodePoint(a, b))) };
  fs.writeFileSync(write, JSON.stringify(out, null, 1) + "\n");
  console.log(`wrote ${write}`);
}
