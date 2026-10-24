#!/usr/bin/env node
// Git merge driver for playwright/durations.json (backlog row 400): a 3-way merge BY ENTRY.
//
//   node playwright/tools/durations-merge.mjs %O %A %B [%P]     writes the merge into %A, exit 0 (clean) or 1
//   node playwright/tools/durations-merge.mjs --self-test
//
// The table is per-spec measured seconds (written by durations.mjs) plus `meta` totals DERIVED from the entries
// (`files`, `tests`, `total_seconds`). Two lanes that each re-measure their own specs change different entries and
// the same `meta` lines, so a textual merge always conflicts on the totals. The merge instead takes, per spec file,
// the side that changed relative to the base (ours when both changed it differently), keeps entries added or removed
// by one side, recomputes the derived `meta` fields from the merged entries, and takes the other `meta` keys the same
// way (budget, cap, date, note). The output is formatted as durations.mjs writes it (indent 1, entries sorted by
// code point, total_seconds rounded to whole seconds), so a clean merge is also byte-stable.
// Registered by scripts/setup-merge-drivers.sh; scripts/check-merge-drivers.sh runs the self-test and a real merge.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import assert from "node:assert/strict";

const J = (x) => JSON.stringify(x);
const byCodePoint = (a, b) => (a < b ? -1 : a > b ? 1 : 0);

function mergeTables(base, ours, theirs) {
  const pick = (b, o, t) => (J(o) === J(b) ? t : J(t) === J(b) ? o : o); // changed side wins, ours when both changed
  const notes = [];
  const files = {};
  const keys = new Set([...Object.keys(base.files ?? {}), ...Object.keys(ours.files), ...Object.keys(theirs.files)]);
  for (const k of keys) {
    const b = base.files?.[k], o = ours.files[k], t = theirs.files[k];
    if (J(o) !== J(b) && J(t) !== J(b) && J(o) !== J(t)) notes.push(k);
    const v = pick(b, o, t);
    if (v !== undefined) files[k] = v;
  }
  const sorted = Object.fromEntries(Object.entries(files).sort(([a], [b]) => byCodePoint(a, b)));
  const meta = {};
  for (const k of new Set([...Object.keys(ours.meta ?? {}), ...Object.keys(theirs.meta ?? {})])) {
    const v = pick(base.meta?.[k], ours.meta?.[k], theirs.meta?.[k]);
    if (v !== undefined) meta[k] = v;
  }
  const entries = Object.values(sorted);
  meta.tests = entries.reduce((s, e) => s + e.tests, 0);
  meta.files = entries.length;
  meta.total_seconds = Math.round(entries.reduce((s, e) => s + e.seconds, 0));
  return { table: { meta, files: sorted }, bothChanged: notes };
}

const render = (table) => JSON.stringify(table, null, 1) + "\n";

function selfTest() {
  const e = (tests, seconds) => ({ tests, seconds, passed: tests, failed: 0, skipped: 0, retried: 0 });
  const mk = (files, extra = {}) => {
    const entries = Object.values(files);
    return { meta: { build: "debug", date: "d0", tests: entries.reduce((s, x) => s + x.tests, 0), files: entries.length,
      total_seconds: Math.round(entries.reduce((s, x) => s + x.seconds, 0)), budget_seconds: 1000, ...extra }, files };
  };
  const base = mk({ "a.spec.ts": e(1, 10), "b.spec.ts": e(2, 20), "c.spec.ts": e(3, 30) });
  // 1. disjoint edits and additions on both sides: both survive, totals are recomputed
  let o = mk({ ...base.files, "a.spec.ts": e(1, 12), "d.spec.ts": e(4, 5) });
  let t = mk({ ...base.files, "b.spec.ts": e(2, 25), "0.spec.ts": e(1, 1) });
  let r = mergeTables(base, o, t);
  assert.deepEqual(Object.keys(r.table.files), ["0.spec.ts", "a.spec.ts", "b.spec.ts", "c.spec.ts", "d.spec.ts"]);
  assert.equal(r.table.files["a.spec.ts"].seconds, 12);
  assert.equal(r.table.files["b.spec.ts"].seconds, 25);
  assert.equal(r.table.meta.files, 5);
  assert.equal(r.table.meta.tests, 1 + 1 + 2 + 3 + 4);
  assert.equal(r.table.meta.total_seconds, Math.round(1 + 12 + 25 + 30 + 5));
  assert.deepEqual(r.bothChanged, []);
  // 2. the same entry changed on both sides: ours wins and it is reported
  o = mk({ ...base.files, "a.spec.ts": e(1, 11) });
  t = mk({ ...base.files, "a.spec.ts": e(1, 13) });
  r = mergeTables(base, o, t);
  assert.equal(r.table.files["a.spec.ts"].seconds, 11);
  assert.deepEqual(r.bothChanged, ["a.spec.ts"]);
  // 3. the same entry changed identically on both sides is not a conflict
  r = mergeTables(base, o, mk({ ...base.files, "a.spec.ts": e(1, 11) }));
  assert.deepEqual(r.bothChanged, []);
  // 4. a deletion on one side stands (b), an edit on the other side stands (c)
  o = mk({ "a.spec.ts": e(1, 10), "c.spec.ts": e(3, 30) });
  t = mk({ ...base.files, "c.spec.ts": e(3, 31) });
  r = mergeTables(base, o, t);
  assert.deepEqual(Object.keys(r.table.files), ["a.spec.ts", "c.spec.ts"]);
  assert.equal(r.table.files["c.spec.ts"].seconds, 31);
  // 5. meta: a budget raised on one side stands, ours wins on both; derived fields never come from a side
  o = mk(base.files);
  t = mk(base.files, { budget_seconds: 2000 });
  assert.equal(mergeTables(base, o, t).table.meta.budget_seconds, 2000);
  t = mk(base.files, { budget_seconds: 2000 });
  o = mk(base.files, { budget_seconds: 3000 });
  assert.equal(mergeTables(base, o, t).table.meta.budget_seconds, 3000);
  o = mk({ ...base.files, "a.spec.ts": e(1, 12) });
  o.meta.total_seconds = 99999;
  assert.equal(mergeTables(base, o, mk(base.files)).table.meta.total_seconds, 12 + 20 + 30);
  // 6. output formatting is durations.mjs's: indent 1, trailing newline, idempotent through a parse
  const text = render(r.table);
  assert.ok(text.startsWith('{\n "meta": {\n  "build"') && text.endsWith("}\n"));
  assert.equal(render(JSON.parse(text)), text);
  // 7. the file entry point: base/ours/theirs on disk, result in ours, exit 0; broken JSON exits 1 and leaves ours
  const d = fs.mkdtempSync(path.join(os.tmpdir(), "durmerge-"));
  try {
    const w = (n, v) => { fs.writeFileSync(path.join(d, n), typeof v === "string" ? v : render(v)); return path.join(d, n); };
    const [pb, po, pt] = [w("b", base), w("o", mk({ ...base.files, "a.spec.ts": e(1, 12) })), w("t", mk({ ...base.files, "b.spec.ts": e(2, 25) }))];
    assert.equal(run(pb, po, pt), 0);
    assert.equal(JSON.parse(fs.readFileSync(po, "utf8")).files["b.spec.ts"].seconds, 25);
    const bad = w("bad", "{ not json");
    const keep = fs.readFileSync(po, "utf8");
    assert.equal(run(pb, po, bad), 1);
    assert.equal(fs.readFileSync(po, "utf8"), keep);
  } finally { fs.rmSync(d, { recursive: true, force: true }); }
  console.log("durations-merge: self-test OK -- 7 cases");
}

function run(pb, po, pt) {
  let base, ours, theirs;
  try {
    const braw = fs.readFileSync(pb, "utf8");
    base = braw.trim() === "" ? { meta: {}, files: {} } : JSON.parse(braw); // add/add: the base is empty
    ours = JSON.parse(fs.readFileSync(po, "utf8"));
    theirs = JSON.parse(fs.readFileSync(pt, "utf8"));
    for (const t of [ours, theirs]) if (!t.files || typeof t.files !== "object") throw new Error("no `files` object");
  } catch (err) {
    console.error(`durations-merge: cannot read an input as a durations table (${err.message}); leaving the merge to the user`);
    return 1;
  }
  const { table, bothChanged } = mergeTables(base, ours, theirs);
  fs.writeFileSync(po, render(table));
  if (bothChanged.length) console.error(`durations-merge: both sides re-measured ${bothChanged.join(", ")}; kept ours`);
  return 0;
}

const argv = process.argv.slice(2);
if (argv[0] === "--self-test") selfTest();
else if (argv.length >= 3) process.exit(run(argv[0], argv[1], argv[2]));
else { console.error("usage: durations-merge.mjs BASE OURS THEIRS [PATH] | --self-test"); process.exit(2); }
