#!/usr/bin/env node
// Duration-balanced shards (backlog row 152a). Playwright's own --shard=i/n balances by test COUNT, so a shard that
// draws the carousel family takes several times longer than one that draws the 400 one-second specs. This sorts the
// spec files by the measured seconds in playwright/durations.json and deals them greedily (longest first) onto the
// currently lightest shard (LPT), so the shards finish within a few percent of each other.
//
//   node playwright/tools/shard.mjs <index>/<count> [--out FILE] [--print-loads] [--table FILE]
//   PW_SPECS_FILE=FILE pnpm exec playwright test --config=suite.local.config.ts     (the config reads the list)
//
// A spec file missing from durations.json is dealt with the median measured duration (the budget gate,
// scripts/check-slow-spec-budget.sh, makes that a red so the table is refreshed).
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PW = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const [spec, ...rest] = process.argv.slice(2);
const m = /^(\d+)\/(\d+)$/.exec(spec ?? "");
if (!m) { console.error("usage: shard.mjs <index>/<count> [--out FILE] [--print-loads]"); process.exit(2); }
const index = Number(m[1]), count = Number(m[2]);
if (index < 1 || index > count) { console.error("shard index out of range"); process.exit(2); }
const out = rest.includes("--out") ? rest[rest.indexOf("--out") + 1] : null;

const walk = (dir, acc = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (["node_modules", "fixtures", "reference", "tools", "test-results", "playwright-report"].includes(e.name)) continue;
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, acc); else if (e.name.endsWith(".spec.ts")) acc.push(path.relative(PW, p).replace(/\\/g, "/"));
  }
  return acc;
};
const specs = walk(PW);
// --table FILE: a durations.json (default) or a `playwright test --list --reporter=json` file (weight = test count x 2.5 s,
// the bootstrap before any durations exist).
const tablePath = rest.includes("--table") ? rest[rest.indexOf("--table") + 1] : path.join(PW, "durations.json");
const raw = JSON.parse(fs.readFileSync(tablePath, "utf8"));
let table = raw.files;
if (!table) {
  table = {};
  const visit = (suite) => {
    for (const sp of suite.specs ?? []) { const f = sp.file.replace(/\\/g, "/"); (table[f] ??= { seconds: 0 }).seconds += 2.5 * (sp.tests?.length ?? 1); }
    for (const x of suite.suites ?? []) visit(x);
  };
  for (const x of raw.suites) visit(x);
}
const known = specs.map((f) => table[f]?.seconds).filter((x) => x != null).sort((a, b) => a - b);
const median = known.length ? known[Math.floor(known.length / 2)] : 10;
const weight = (f) => table[f]?.seconds ?? median;
const shards = Array.from({ length: count }, () => ({ load: 0, files: [] }));
for (const f of [...specs].sort((a, b) => weight(b) - weight(a) || a.localeCompare(b))) {
  const s = shards.reduce((a, b) => (b.load < a.load ? b : a));
  s.files.push(f); s.load += weight(f);
}
const mine = shards[index - 1].files.sort();
if (rest.includes("--print-loads")) console.error(shards.map((s, i) => `shard ${i + 1}/${count}: ${s.files.length} files, ${s.load.toFixed(0)} s`).join("\n"));
if (out) fs.writeFileSync(out, mine.join("\n") + "\n"); else console.log(mine.join("\n"));
