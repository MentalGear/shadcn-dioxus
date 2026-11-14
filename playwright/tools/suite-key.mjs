#!/usr/bin/env node
// Per-spec-file reuse of release-suite results (backlog row 513).
//
// A suite result is valid for one SITE KEY and one SPEC KEY. Both are hashes of git objects of one tree, so they are exact,
// cheap and the same for every checkout of that tree; neither is read from the working tree.
//
// SITE KEY: every input of the release build.
//   = the tree's entries (mode, blob id, path) minus the excluded prefixes below, plus every excluded file a Rust source
//     pulls in with `include_str!` / `include_bytes!` / `include!` (resolved against the including file), plus the tool's own
//     bytes and the build environment string the caller passes (--build-env: dx, rustc and cargo versions, RUSTFLAGS).
//   Excluded, and why none of them can change the built site:
//     playwright/  the specs and their data; no build step reads it (the Cargo workspace does not contain it; scripts/ and
//                  preview/ never name it). A spec reads it, so the SPEC KEY covers it.
//     dev-docs/    prose; the one build input in it, dev-docs/motion.md, is named by an `include_str!` of
//                  preview/src/motion_docs.rs and comes back into the key through the include scan. A new include of another
//                  file in it comes back the day it is written; an include this scan cannot read stops the key (exit 3).
//     docs/        the deploy's own output (the published copy of a build), read by nothing.
//     primitives/ts-test/  the page-side harness and the bun tests of the bridge: not shipped (scripts/check-js-volume.sh), named
//                  by no Cargo manifest, include or build script. The Playwright specs that import from it carry it in their
//                  SPEC KEY through the import closure.
//   An unreadable include, or an excluded path named by a read this scan cannot see, is the residue; see the self-test.
//
// SPEC KEY: everything one spec file's result depends on besides the site.
//   = the spec file, every module it loads transitively (import-graph.mjs; also files outside playwright/ that it imports),
//     the Playwright config and its imports, fixtures.ts and its imports, and every tracked file under playwright/ (outside
//     tools/ and durations.json) that is in NO spec's closure, whatever its extension (pnpm-lock.yaml, package.json, fixtures/,
//     oracle/, vendored scripts a page loads): defined by subtraction, so a file nothing proves unread is in every key. The
//     data a registered reader reads (DATA_READERS) is the only narrowing: only specs whose closure contains the reader take it.
//     Also: the environment variables the closure reads (a real parser finds every `process.env` access), the path of
//     PLAYWRIGHT_BASE_URL, the browser the config launches, the Node version and platform, `TIMING_PASS`, the parser in use and
//     the tool's own bytes.
//   A spec whose closure has a relative import that resolves to no file, a computed import(), or a bare use of process.env that
//   is not reviewed has NO key: it always runs and never gets a record.
//
// A RECORD is green by positive proof only. The test ids of a file in the run's report must equal the ids of an independent
// `playwright test --list` of the same files, config and environment (run-suite.sh writes list-N.json / tlist-N.json before the
// chunk runs), and every one of them must end expected or flaky. A file with a test that was skipped by any means, interrupted,
// did not run or has an unknown status gets no record and runs every time; a missing or failing list gets none either. A run's
// reports belong to that run (started-at), and the worktree must still be the planned tree when a record is written.
//
// Not in either key, on purpose: the host and port of PLAYWRIGHT_BASE_URL and SSG_SITE_DIR (a port and a path; the site's bytes
// are covered by the built-site hash recorded with every result), PW_WORKERS / PW_SPECS_FILE (parallelism and selection), durations.json (it
// only cuts chunks), tools/ (they run tests, they are not tests; their own bytes are in both keys).
//
//   node playwright/tools/suite-key.mjs site-key [--tree T] [--build-env S]
//   node playwright/tools/suite-key.mjs keys     [--tree T] [--config F]
//   node playwright/tools/suite-key.mjs plan     --store DIR [--tree T] [--config F] [--public DIR] [--out FILE] [--build-env S]
//   node playwright/tools/suite-key.mjs accept   --store DIR [--tree T] ...    exit 0 only when every spec file has a green record
//   node playwright/tools/suite-key.mjs record   --store DIR --out RUNDIR --chunks N [--repo R]
//   node playwright/tools/suite-key.mjs verdict  --out RUNDIR --chunks N [--store DIR --tree T]
//   node playwright/tools/suite-key.mjs hash-dir DIR
// Exit: 0 ok, 1 not accepted / red, 2 usage, 3 a key could not be derived (the caller runs the whole suite).
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { analyze, importClosure, parserName, SOURCE_RE } from "./import-graph.mjs";
import { isSpecPath } from "./spec-files.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..");
export const FORMAT = 2;
export const EXCLUDED = ["playwright/", "dev-docs/", "docs/", "primitives/ts-test/"];
export const isExcluded = (p) => EXCLUDED.some((e) => p.startsWith(e));
const TOOL_FILES = ["suite-key.mjs", "import-graph.mjs", "spec-files.mjs"];
const MODULE_RE = SOURCE_RE;

// Data a registered reader reads: only a spec whose import closure contains a reader takes the data into its key. Every other
// data file under playwright/ is in every spec key. The self-test pins that each reader names its data and nothing else does.
export const DATA_READERS = [
  { data: /^playwright\/witness\/bindings\//, readers: ["playwright/witness/binding.ts"], mentions: /["'`/]bindings["'`/]/ },
  { data: /^playwright\/fixtures\/carousel-wheel-recordings\.json$/, readers: ["playwright/wheel-replay.ts"], mentions: /carousel-wheel-recordings/ },
  { data: /^playwright\/fixtures\/carousel-overscroll-bench-rev40\.html$/, readers: ["playwright/carousel-wheel-replay.spec.ts"], mentions: /carousel-overscroll-bench/ },
  { data: /^playwright\/oracle\/attr-synth\//, readers: ["playwright/oracle/hydration-parity.spec.ts"], mentions: /attr-synth/ },
];
// Not tests and not read by tests.
const NOT_TEST_INPUT = (p) => /^playwright\/(?:tools\/|durations\.json$|node_modules\/|test-results\/|playwright-report\/)/.test(p);
// Environment variables that never enter a spec key (PLAYWRIGHT_BASE_URL enters by its path only).
export const ENV_NEUTRAL = new Set(["SSG_SITE_DIR", "PW_WORKERS", "PW_SPECS_FILE", "PW_TIMING_SKIP_WARNED",
  "PW_CHROMIUM", "PLAYWRIGHT_BROWSERS_PATH"]);
// A bare `process.env` (a default parameter, a spread) names no variable. Reviewed: the browser it selects is hashed through
// the browser path (sandbox-chromium.cjs). Any other file with one has no key.
export const BARE_ENV_REVIEWED = new Set(["playwright/sandbox-chromium.cjs"]);

const sha = (rows) => crypto.createHash("sha256").update(rows.join("\n")).digest("hex");
export const toolHash = () => sha(TOOL_FILES.map((f) => `${f} ${crypto.createHash("sha256").update(fs.readFileSync(path.join(HERE, f))).digest("hex")}`));

export class KeyError extends Error {}

// ---- a tree of git objects ---------------------------------------------------------------------------------------
export function openTree(repo, treeish) {
  const git = (args, input) => execFileSync("git", ["-C", repo, ...args], { input, maxBuffer: 1 << 30, stdio: ["pipe", "pipe", "pipe"] });
  let tree;
  try { tree = git(["rev-parse", "--verify", `${treeish}^{tree}`]).toString().trim(); } catch { throw new KeyError(`not a tree: ${treeish}`); }
  const entries = new Map();
  for (const rec of git(["ls-tree", "-r", "-z", "--full-tree", tree]).toString("utf8").split("\0").filter(Boolean)) {
    const m = /^(\d+) (\w+) ([0-9a-f]+)\t([\s\S]+)$/.exec(rec);
    if (!m) throw new KeyError(`unreadable tree entry: ${rec.slice(0, 80)}`);
    if (m[4].includes("\n")) throw new KeyError(`a newline in a path: ${JSON.stringify(m[4])}`);
    entries.set(m[4], { mode: m[1], type: m[2], oid: m[3] });
  }
  const cache = new Map();
  // one `git cat-file --batch` for every blob not cached yet
  const prefetch = (paths) => {
    const want = [...new Set(paths)].filter((p) => entries.has(p) && !cache.has(p));
    if (!want.length) return;
    const out = git(["cat-file", "--batch"], want.map((p) => entries.get(p).oid).join("\n") + "\n");
    let at = 0;
    for (const p of want) {
      const nl = out.indexOf(10, at);
      const [, type, size] = out.subarray(at, nl).toString().split(" ");
      if (type !== "blob") throw new KeyError(`${p} is not a blob`);
      cache.set(p, out.subarray(nl + 1, nl + 1 + Number(size)).toString("utf8"));
      at = nl + 1 + Number(size) + 1;
    }
  };
  const read = (p) => { if (!cache.has(p)) prefetch([p]); if (!cache.has(p)) throw new KeyError(`no such file in ${tree}: ${p}`); return cache.get(p); };
  const grepFiles = (pattern, glob) => {
    try { return git(["grep", "-l", "-E", pattern, tree, "--", glob]).toString().split("\n").filter(Boolean).map((l) => l.slice(tree.length + 1)); } catch { return []; }
  };
  return { repo, tree, entries, exists: (p) => entries.has(p), read, prefetch, grepFiles };
}

// ---- the site key -------------------------------------------------------------------------------------------------
function macroArgs(text, open) { // the text between the parentheses opening at `open`, string literals skipped
  let depth = 0, i = open, inStr = false;
  for (; i < text.length; i++) {
    const c = text[i];
    if (inStr) { if (c === "\\") i++; else if (c === '"') inStr = false; continue; }
    if (c === '"') inStr = true;
    else if (c === "(") depth++;
    else if (c === ")" && --depth === 0) return text.slice(open + 1, i);
  }
  return null;
}
function topLevelSplit(args) {
  const out = []; let depth = 0, inStr = false, cur = "";
  for (let i = 0; i < args.length; i++) {
    const c = args[i];
    if (inStr) { cur += c; if (c === "\\") cur += args[++i] ?? ""; else if (c === '"') inStr = false; continue; }
    if (c === '"') inStr = true;
    if (c === "(" || c === "{" || c === "[") depth++;
    if (c === ")" || c === "}" || c === "]") depth--;
    if (c === "," && depth === 0) { out.push(cur.trim()); cur = ""; } else cur += c;
  }
  if (cur.trim()) out.push(cur.trim());
  return out;
}
const LIT = /^"((?:[^"\\]|\\.)*)"$/;

/**
 * The files Rust sources of the key pull in by `include_str!` / `include_bytes!` / `include!`: { targets: Set(path), generated, unreadable }.
 * `generated` counts includes of a build-script output (`env!("OUT_DIR")`, produced from files already in the key).
 * Only a line that starts with `//` is skipped (it is a comment whatever the lines before it hold, so nothing real is hidden);
 * block comments are not stripped, because telling one from a string needs a Rust lexer: a mention inside one is read as an
 * include, which costs a re-run or stops the key when it cannot be read, never a missed include.
 */
export function rustIncludes(t) {
  const targets = new Set(), unreadable = []; let generated = 0;
  for (const f of t.grepFiles("include_(str|bytes)!|(^|[^a-z_])include!", "*.rs")) {
    if (isExcluded(f)) continue;
    const text = t.read(f).replace(/^[ \t]*\/\/.*$/gm, (c) => " ".repeat(c.length));
    for (const m of text.matchAll(/\binclude(?:_str|_bytes)?!\s*\(/g)) {
      // inside a string literal (a build script writing Rust): text, not an include; fine only when it names OUT_DIR
      const lineStart = text.lastIndexOf("\n", m.index) + 1;
      if ((text.slice(lineStart, m.index).match(/(?<!\\)"/g) ?? []).length % 2 === 1) {
        if (/env!\(\\?"OUT_DIR\\?"\)/.test(text.slice(m.index, m.index + 120))) generated++; else unreadable.push(`${f}: generated ${m[0]}...`);
        continue;
      }
      const args = macroArgs(text, m.index + m[0].length - 1);
      if (args === null) { unreadable.push(`${f}: unbalanced ${m[0]}`); continue; }
      const a = args.trim();
      let parts;
      if (LIT.test(a)) parts = [a];
      else if (/^concat!\s*\(/.test(a)) parts = topLevelSplit(a.slice(a.indexOf("(") + 1, a.lastIndexOf(")")));
      else { unreadable.push(`${f}: ${m[0]}${a.slice(0, 50)}`); continue; }
      if (/^env!\s*\(\s*"OUT_DIR"\s*\)$/.test(parts[0])) { generated++; continue; }
      const first = LIT.exec(parts[0]);
      if (!first) { unreadable.push(`${f}: ${m[0]}${a.slice(0, 50)}`); continue; }
      const dir = path.posix.dirname(f);
      if (parts.every((p) => LIT.test(p))) targets.add(path.posix.normalize(path.posix.join(dir, parts.map((p) => LIT.exec(p)[1]).join(""))));
      else {
        const resolved = path.posix.normalize(path.posix.join(dir, first[1]));
        if (isExcluded(resolved) || resolved.startsWith("..")) unreadable.push(`${f}: ${m[0]}${a.slice(0, 50)} names a path outside the key by a computed name`);
        // a computed name under an included directory (components/<kind>/variants/<v>/mod.rs) is inside the key already
      }
    }
  }
  return { targets, generated, unreadable };
}

export function siteKey(t, { buildEnv = "" } = {}) {
  const inc = rustIncludes(t);
  if (inc.unreadable.length) throw new KeyError(`include this scan cannot read:\n  ${inc.unreadable.join("\n  ")}`);
  const pulledBack = [...inc.targets].filter((p) => isExcluded(p)).sort();
  const rows = [];
  for (const [p, e] of [...t.entries].sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0))) {
    if (!isExcluded(p)) rows.push(`${e.mode} ${e.oid} ${p}`);
  }
  for (const p of pulledBack) rows.push(t.exists(p) ? `${t.entries.get(p).mode} ${t.entries.get(p).oid} ${p}` : `missing ${p}`);
  const key = sha([`site-key ${FORMAT}`, `tool ${toolHash()}`, `build-env ${buildEnv}`, ...rows]);
  return { key, files: rows.length - pulledBack.length, pulledBack, generatedIncludes: inc.generated };
}

// ---- the spec keys ------------------------------------------------------------------------------------------------
export function specFilesOf(t) {
  return [...t.entries.keys()].filter((p) => p.startsWith("playwright/") && isSpecPath(p.slice("playwright/".length))).map((p) => p.slice("playwright/".length)).sort();
}

/** The browser the config launches (sandbox-chromium.cjs resolves it from the environment), the runtime that runs the specs. */
export function runtimeFingerprint(repo = REPO, env = process.env) {
  let chromium = "default";
  try { chromium = createRequire(import.meta.url)(path.join(repo, "playwright", "sandbox-chromium.cjs")).sandboxChromium(env) ?? "default"; } catch (e) { chromium = `unresolvable ${e.message}`; }
  return `node ${process.versions.node} ${process.platform} ${process.arch} chromium ${chromium}`;
}

// The build profile the specs see (base-url.ts buildProfile(): the explicit variable wins, else the shape of the site dir).
const profileOf = (env) => {
  const explicit = (env.PW_BUILD_PROFILE ?? "").toLowerCase();
  if (explicit === "debug" || explicit === "release") return explicit;
  const dir = (env.SSG_SITE_DIR ?? "").replace(/\\/g, "/");
  return /\/dx\/[^/]+\/debug\/web\/public\/?$/.test(dir) ? "debug" : /\/dx\/[^/]+\/release\/web\/public\/?$/.test(dir) ? "release" : "marker-or-unknown";
};
// A spec decides on the path of the base URL (a prefixed build is served under a prefix); host and port are per run.
const basePathOf = (v) => { if (!v) return ""; try { return new URL(v).pathname.replace(/\/+$/, ""); } catch { return `raw:${v}`; } };

/** { file -> { key, why } } for every spec file of the tree; `key` is null and `why` says why when the file cannot be keyed. */
export function specKeys(t, { config = "suite.local.config.ts", runtime = runtimeFingerprint(t.repo), env = process.env } = {}) {
  t.prefetch([...t.entries.keys()].filter((p) => p.startsWith("playwright/") && MODULE_RE.test(p) && !NOT_TEST_INPUT(p)));
  const closureCache = new Map();
  const closureOf = (root) => {
    if (!closureCache.has(root)) closureCache.set(root, t.exists(root) ? importClosure(root, { exists: t.exists, read: t.read }) : { files: new Set(), unresolved: [{ from: root, spec: "(missing)" }], opaque: [] });
    return closureCache.get(root);
  };
  const analysis = new Map();
  const analysisOf = (f) => { if (!analysis.has(f)) analysis.set(f, MODULE_RE.test(f) ? analyze(t.read(f), f) : { env: new Set(), bare: 0 }); return analysis.get(f); };

  const specs = specFilesOf(t);
  const roots = new Map(specs.map((file) => [file, [`playwright/${file}`, `playwright/${config}`, "playwright/fixtures.ts"]]));
  const inSomeClosure = new Set();
  for (const rs of roots.values()) for (const r of rs) for (const f of closureOf(r).files) inSomeClosure.add(f);

  const candidates = [...t.entries.keys()].filter((p) => p.startsWith("playwright/") && !NOT_TEST_INPUT(p));
  const isNarrowed = (p) => DATA_READERS.some((r) => r.data.test(p));
  const narrowed = DATA_READERS.map((r) => ({ ...r, files: candidates.filter((p) => r.data.test(p)) }));
  // by subtraction: what no spec loads, nothing proves unread
  const ambient = sha(candidates.filter((p) => !inSomeClosure.has(p) && !isNarrowed(p)).sort().map((p) => `${t.entries.get(p).oid} ${p}`));
  const row = (p) => `${t.entries.get(p).oid} ${p}`;

  const base = [`spec-key ${FORMAT}`, `tool ${toolHash()}`, `parser ${parserName()}`, `runtime ${runtime}`, `ambient ${ambient}`, `timing-pass ${env.TIMING_PASS ?? "<unset>"}`];
  const out = new Map();
  for (const file of specs) {
    const files = new Set(), why = [];
    for (const r of roots.get(file)) {
      const c = closureOf(r);
      for (const f of c.files) files.add(f);
      for (const u of c.unresolved) why.push(`${u.from} imports "${u.spec}", which names no file`);
      for (const o of c.opaque) why.push(`${o.from} loads a computed module (${o.text})`);
    }
    const names = new Set();
    for (const f of files) {
      const a = analysisOf(f);
      for (const n of a.env) if (!ENV_NEUTRAL.has(n)) names.add(n);
      if (a.bare && !BARE_ENV_REVIEWED.has(f)) why.push(`${f} uses process.env as a whole`);
    }
    if (why.length) { out.set(file, { key: null, why: why[0], reasons: why }); continue; }
    const rows = [...base, ...[...files].sort().map(row)];
    for (const n of narrowed) if (n.readers.some((r) => files.has(r))) rows.push(`data ${n.readers.join(",")} ${sha(n.files.sort().map(row))}`);
    for (const n of [...names].sort()) rows.push(n === "PLAYWRIGHT_BASE_URL" ? `env PLAYWRIGHT_BASE_URL path=${basePathOf(env[n])}` : `env ${n}=${env[n] ?? "<unset>"}`);
    rows.push(`profile ${profileOf(env)}`);
    out.set(file, { key: sha(rows), files: files.size });
  }
  return out;
}

// ---- the built site -----------------------------------------------------------------------------------------------
/** A hash of a directory's files: names and bytes, a symlink by its target (not times, not modes). */
export function hashDir(dir) {
  const rows = [];
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
      const p = path.join(d, e.name), rel = path.relative(dir, p);
      if (e.isSymbolicLink()) rows.push(`link ${fs.readlinkSync(p)} ${rel}`);
      else if (e.isDirectory()) walk(p);
      else if (e.isFile()) rows.push(`${crypto.createHash("sha256").update(fs.readFileSync(p)).digest("hex")} ${rel}`);
      else rows.push(`special ${rel}`);
    }
  };
  walk(dir);
  return sha(rows);
}

// ---- the store ------------------------------------------------------------------------------------------------------
const recordFile = (store, site, file, specKey) => path.join(store, site, `${encodeURIComponent(file)}@${specKey.slice(0, 16)}.json`);

/**
 * The record when it is acceptable for (site, file, specKey); otherwise { reason }. A record is acceptable only when it carries
 * its proof: green, every test expected or flaky (none skipped), and the number of tests the independent list named.
 */
export function lookup(store, site, file, specKey, { publicSha = null, timing } = {}) {
  let r;
  try { r = JSON.parse(fs.readFileSync(recordFile(store, site, file, specKey), "utf8")); } catch { return { reason: "no record" }; }
  const c = r?.counts;
  if (r?.format !== FORMAT) return { reason: "record format differs" };
  if (r.siteKey !== site || r.specKey !== specKey || r.file !== file) return { reason: "record names another key" };
  if (r.status !== "green") return { reason: `record is ${r.status}` };
  if (!c || !["expected", "unexpected", "flaky", "skipped"].every((k) => Number.isInteger(c[k]) && c[k] >= 0) || c.unexpected !== 0 || c.skipped !== 0) return { reason: "record counts are not green" };
  if (!r.proof || r.proof.listed !== r.proof.reported || r.proof.reported !== c.expected + c.flaky) return { reason: "record carries no proof that every listed test ran" };
  if (typeof r.publicSha !== "string" || !r.publicSha) return { reason: "record names no built site" };
  if (publicSha && r.publicSha !== publicSha) return { reason: "record is for another build of the site" };
  if (timing !== undefined && r.timingPass !== timing) return { reason: `record ran with the timing pass ${r.timingPass}` };
  return { record: r };
}

function writeRecord(store, rec) {
  const f = recordFile(store, rec.siteKey, rec.file, rec.specKey);
  fs.mkdirSync(path.dirname(f), { recursive: true });
  const tmp = `${f}.${process.pid}.tmp`;
  fs.writeFileSync(tmp, JSON.stringify(rec, null, 1) + "\n");
  fs.renameSync(tmp, f);
}

// ---- one summary, one guard: accept and verdict print and judge the same way ---------------------------------------
/** The whole-suite line and the reasons the totals cannot stand for a green suite (nothing passed). */
export function summarize({ passed, failed, flaky, skipped, chunksMissing = 0, reusedFiles, reusedPassed, runFiles }, env = process.env) {
  const note = env.TIMING_PASS === "on" ? "" : ` timing cases deactivated (row ${env.TIMING_REENABLE_ROW})`;
  const reasons = [];
  if (!(passed > 0)) reasons.push("no test passed");
  if (failed > 0) reasons.push(`${failed} failed`);
  return { reasons, line: `passed=${passed} failed=${failed} flaky=${flaky} skipped=${skipped} chunks_missing=${chunksMissing} reused_files=${reusedFiles} reused_passed=${reusedPassed} run_files=${runFiles}${note}` };
}

/** The reason the worktree is not exactly `tree` (HEAD's tree, no tracked change outside docs/), or null. */
export function worktreeProblem(repo, tree) {
  const git = (...a) => execFileSync("git", ["-C", repo, ...a], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  try {
    const head = git("rev-parse", "HEAD^{tree}");
    if (head !== tree) return `HEAD's tree ${head.slice(0, 12)} is not the tree ${tree.slice(0, 12)} the run was planned on`;
    if (git("status", "--porcelain", "--untracked-files=no", "--", ".", ":(exclude)docs")) return "tracked files changed during the run";
  } catch (e) { return `cannot read the worktree: ${e.message.split("\n")[0]}`; }
  return null;
}

// ---- plan: which spec files still have to run ---------------------------------------------------------------------
export function plan(t, store, opts = {}) {
  const env = opts.env ?? process.env, timing = env.TIMING_PASS ?? "<unset>";
  const site = siteKey(t, { buildEnv: opts.buildEnv ?? "" });
  const keys = specKeys(t, { config: opts.config, runtime: opts.runtime, env });
  const publicSha = opts.publicDir ? hashDir(opts.publicDir) : null;
  const found = [], run = [], unkeyable = [], notes = {};
  for (const [file, k] of keys) {
    if (!k.key) { run.push(file); unkeyable.push({ file, why: k.why }); continue; }
    const r = lookup(store, site.key, file, k.key, { publicSha, timing });
    if (r.record) found.push({ file, specKey: k.key, record: r.record }); else { run.push(file); notes[file] = r.reason; }
  }
  // Without a built site in hand the records must all name one build; records of different builds are not a whole.
  let reusedPublicSha = publicSha;
  if (!publicSha) {
    const shas = new Set(found.map((f) => f.record.publicSha));
    if (shas.size > 1) { for (const f of found) { run.push(f.file); notes[f.file] = "records name different builds of the site"; } found.length = 0; }
    reusedPublicSha = shas.size === 1 ? [...shas][0] : null;
  }
  run.sort();
  const reused = found.sort((a, b) => (a.file < b.file ? -1 : 1)).map((f) => ({ file: f.file, specKey: f.specKey, counts: f.record.counts, flaky: f.record.flaky ?? [] }));
  const totals = { expected: reused.reduce((n, r) => n + r.counts.expected, 0), flaky: reused.reduce((n, r) => n + r.counts.flaky, 0) };
  return {
    format: FORMAT, tree: t.tree, siteKey: site.key, publicSha: reusedPublicSha, timingPass: timing, siteFiles: site.files, pulledBack: site.pulledBack,
    files: keys.size, run, unkeyable, reused, totals,
    keys: Object.fromEntries([...keys].filter(([, k]) => k.key).map(([f, k]) => [f, k.key])), notes, allGreen: run.length === 0,
  };
}

/** What `accept` prints and judges for a plan: the same summary and guard as `verdict`, with nothing run. */
export function acceptance(p, env = process.env) {
  const s = summarize({ passed: p.totals.expected, failed: 0, flaky: p.totals.flaky, skipped: 0, reusedFiles: p.reused.length, reusedPassed: p.totals.expected, runFiles: 0 }, env);
  const reasons = [...s.reasons];
  if (!p.allGreen) reasons.push(`${p.run.length} spec file(s) without a green record`);
  if (!p.publicSha) reasons.push("the records name no single built site");
  return { ok: reasons.length === 0, line: s.line, reasons };
}

// ---- reading a run's reports ---------------------------------------------------------------------------------------
const readJson = (f) => { try { return JSON.parse(fs.readFileSync(f, "utf8")); } catch { return null; } };
const lines = (f) => { try { return fs.readFileSync(f, "utf8").split("\n").map((l) => l.trim()).filter(Boolean); } catch { return []; } };

/** { file -> [{ id, status, results: [status] }] } of one JSON report (a run) or list (a `--list` run). */
export function testsOf(report) {
  const out = new Map();
  const walk = (suite, file, trail) => {
    for (const s of suite.suites ?? []) walk(s, s.file ?? file, [...trail, s.title]);
    for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
      const f = spec.file ?? file;
      if (!out.has(f)) out.set(f, []);
      out.get(f).push({ id: `${f}␟${[...trail, spec.title].join(" > ")}␟${test.projectName ?? ""}`, title: `${f}: ${spec.title}`, status: test.status, results: (test.results ?? []).map((r) => r.status) });
    }
  };
  for (const s of report.suites ?? []) walk(s, s.file, [s.title]);
  return out;
}
const multiset = (ids) => { const m = new Map(); for (const i of ids) m.set(i, (m.get(i) ?? 0) + 1); return m; };
function sameIds(listed, reported) {
  const a = multiset(listed), b = multiset(reported), missing = [], extra = [];
  for (const [k, n] of a) if ((b.get(k) ?? 0) < n) missing.push(k);
  for (const [k, n] of b) if ((a.get(k) ?? 0) < n) extra.push(k);
  return { equal: !missing.length && !extra.length, missing, extra };
}
const emptyState = () => ({ irregular: 0, listed: 0, reported: 0, equal: true, mismatch: [], expected: 0, unexpected: 0, flaky: 0, skipped: 0, failed: [], flakyTitles: [], timingListed: 0 });

/** Folds one report's tests of a file into a state, judged against the ids the independent list named for it. */
function judge(st, listedTests, reportedTests, label) {
  const proof = sameIds(listedTests.map((t) => t.id), reportedTests.map((t) => t.id));
  st.listed += listedTests.length; st.reported += reportedTests.length;
  if (!proof.equal) { st.equal = false; st.mismatch.push(`${label}: ${proof.missing.length} listed test(s) missing from the report, ${proof.extra.length} reported that were not listed`); }
  for (const t of reportedTests) {
    if (t.results.includes("interrupted")) { st.unexpected++; st.irregular++; st.failed.push(`${t.title} (interrupted)`); }
    else if (t.status === "expected") st.expected++;
    else if (t.status === "flaky") { st.flaky++; st.flakyTitles.push(t.title); }
    else if (t.status === "skipped" && t.results.length > 0 && t.results.every((r) => r === "skipped")) st.skipped++; // a test that decided to skip
    else if (t.status === "unexpected") { st.unexpected++; st.failed.push(`${t.title} (failed)`); }
    else { st.unexpected++; st.irregular++; st.failed.push(`${t.title} (${t.status === "skipped" ? "did not run" : `status ${t.status}`})`); } // did not run, or a status this reader does not know
  }
}

/**
 * The state of a run directory. A chunk counts only when its report parses, has no run errors and was started after the run
 * began (`started-at`), and the two lists (list.json, tlist.json) parse; then each listed file gets a state: its ids
 * against the list, and the outcome of every test. A file of an incomplete chunk has no state.
 */
export function readRun(out, chunks, { timingExpected }) {
  const files = new Map(), incomplete = [];
  const startedAt = Date.parse(lines(path.join(out, "started-at"))[0] ?? "");
  if (!Number.isFinite(startedAt)) return { files, incomplete: ["no started-at: the reports cannot be tied to this run"], timingIncomplete: true };
  const fresh = (j) => Number.isFinite(Date.parse(j.stats?.startTime)) && Date.parse(j.stats.startTime) >= startedAt;
  const clean = (j) => j && j.stats && !(j.errors ?? []).length;
  const timingListed = new Map(); // file -> tests the timing list names
  const L = readJson(path.join(out, "list.json")), T = readJson(path.join(out, "tlist.json"));
  for (let i = 1; i <= chunks; i++) {
    const listed = lines(path.join(out, `specs-${i}.txt`));
    if (!listed.length) continue; // no file to run in this chunk: no report either
    const j = readJson(path.join(out, `chunk-${i}.json`));
    const bad = !j ? "no report" : !clean(j) ? `${(j.errors ?? []).length} run errors` : !fresh(j) ? "a report from an earlier run" : !clean(L) ? "no list" : !clean(T) ? "no timing list" : null;
    if (bad) { incomplete.push(`${i} (${bad})`); continue; }
    const rep = testsOf(j), lst = testsOf(L), tl = testsOf(T);
    for (const f of listed) {
      const st = emptyState();
      judge(st, lst.get(f) ?? [], rep.get(f) ?? [], "chunk");
      st.timingListed = (tl.get(f) ?? []).length;
      timingListed.set(f, tl.get(f) ?? []);
      files.set(f, st);
    }
  }
  let timingIncomplete = false;
  if (timingExpected) {
    const j = readJson(path.join(out, "chunk-timing.json"));
    if (!clean(j) || !fresh(j)) { timingIncomplete = true; incomplete.push("timing"); }
    else {
      const rep = testsOf(j);
      for (const [f, st] of files) judge(st, timingListed.get(f) ?? [], rep.get(f) ?? [], "timing pass");
    }
  }
  return { files, incomplete, timingIncomplete };
}

// ---- record ----------------------------------------------------------------------------------------------------------
/** Writes one record per spec file this run executed and proved; returns { green, red, unrecorded }. */
export function record(store, out, chunks, { planFile = path.join(out, "plan.json"), now = new Date(), repo = REPO, worktree = (tree) => worktreeProblem(repo, tree) } = {}) {
  const pl = readJson(planFile);
  if (!pl?.siteKey || !pl.publicSha || !pl.keys) return { green: 0, red: 0, unrecorded: ["(no plan with a built-site hash: nothing is recorded)"] };
  const problem = worktree(pl.tree);
  if (problem) return { green: 0, red: 0, unrecorded: [`(${problem}: nothing is recorded)`] };
  const timingExpected = pl.timingPass === "on" && pl.run.length > 0;
  const run = readRun(out, chunks, { timingExpected });
  const res = { green: 0, red: 0, unrecorded: [] };
  for (const file of pl.run) {
    const specKey = pl.keys[file], st = run.files.get(file);
    if (!specKey) { res.unrecorded.push(`${file}: no key`); continue; }
    if (!st) { res.unrecorded.push(`${file}: no complete report`); continue; }
    if (timingExpected && run.timingIncomplete) { res.unrecorded.push(`${file}: the timing pass did not complete`); continue; }
    if (!st.equal) { res.unrecorded.push(`${file}: the report does not match the list (${st.mismatch.join("; ")})`); continue; }
    if (st.listed + st.timingListed === 0) { res.unrecorded.push(`${file}: no test listed`); continue; }
    if (st.irregular > 0) { res.unrecorded.push(`${file}: ${st.irregular} test(s) interrupted, not run or of unknown status; the file runs every time`); continue; }
    if (st.skipped > 0) { res.unrecorded.push(`${file}: ${st.skipped} test(s) skipped; a skipped test is not proof, the file runs every time`); continue; }
    const green = st.unexpected === 0;
    writeRecord(store, {
      format: FORMAT, file, siteKey: pl.siteKey, specKey, publicSha: pl.publicSha, timingPass: pl.timingPass, tree: pl.tree,
      status: green ? "green" : "red", proof: { listed: st.listed, reported: st.reported },
      counts: { expected: st.expected, unexpected: st.unexpected, flaky: st.flaky, skipped: st.skipped }, flaky: st.flakyTitles,
      failed: st.failed, recordedAt: now.toISOString(), runDir: out,
    });
    if (green) res.green++; else res.red++;
  }
  // site directories not written for 21 days
  for (const d of fs.readdirSync(store, { withFileTypes: true })) {
    if (d.isDirectory() && d.name !== pl.siteKey && Date.now() - fs.statSync(path.join(store, d.name)).mtimeMs > 21 * 86400e3) fs.rmSync(path.join(store, d.name), { recursive: true, force: true });
  }
  return res;
}

// ---- verdict ---------------------------------------------------------------------------------------------------------
/**
 * The whole-suite verdict of one run directory: every spec file of the tree is either a reused green record (checked again
 * now) or a file this run executed in a complete chunk whose reported tests equal its list and none failed (a test that decided
 * to skip is allowed here and counted; it only keeps the file from getting a record). { ok, line, failed, flaky, reasons }.
 */
export function verdict(out, chunks, { store = null, tree = null, env = process.env, repo = REPO, worktree = (t) => worktreeProblem(repo, t) } = {}) {
  const pl = readJson(path.join(out, "plan.json"));
  if (tree) { try { tree = openTree(repo, tree).tree; } catch { /* reported below as a worktree problem */ } }
  const timingExpected = env.TIMING_PASS === "on" && (!pl || pl.run.length > 0);
  const run = readRun(out, chunks, { timingExpected });
  const reasons = [], total = { expected: 0, unexpected: 0, flaky: 0, skipped: 0 };
  const failed = [], flaky = [];
  for (const [f, st] of run.files) {
    for (const k of Object.keys(total)) total[k] += st[k];
    failed.push(...st.failed); flaky.push(...st.flakyTitles);
    if (!st.equal) reasons.push(`${f}: the report does not match the list (${st.mismatch.join("; ")})`);
  }
  const runFiles = pl ? pl.run : [...run.files.keys()];
  for (const f of runFiles) if (!run.files.has(f)) reasons.push(`${f} has no complete report`);
  if (tree) {
    const problem = worktree(tree);
    if (problem) reasons.push(problem);
    if (pl && pl.tree !== tree) reasons.push("the plan was made for another tree");
  }
  const reused = { files: 0, expected: 0, flaky: 0 };
  if (pl) {
    if (!store || !tree) reasons.push("a run with reused files needs --store and --tree to check them again");
    else {
      const t = openTree(repo, tree);
      if (JSON.stringify(specFilesOf(t)) !== JSON.stringify([...pl.run, ...pl.reused.map((r) => r.file)].sort())) reasons.push("the plan does not cover exactly the spec files of the tree");
      for (const r of pl.reused) {
        const again = lookup(store, pl.siteKey, r.file, r.specKey, { publicSha: pl.publicSha, timing: pl.timingPass });
        if (!again.record) { reasons.push(`reused ${r.file}: ${again.reason}`); continue; }
        reused.files++; reused.expected += again.record.counts.expected; reused.flaky += again.record.counts.flaky;
        flaky.push(...(again.record.flaky ?? []).map((x) => `${x} (reused)`));
      }
    }
  }
  const s = summarize({
    passed: total.expected + reused.expected, failed: total.unexpected, flaky: total.flaky + reused.flaky, skipped: total.skipped,
    chunksMissing: run.incomplete.length ? run.incomplete.join(",") : 0, reusedFiles: reused.files, reusedPassed: reused.expected, runFiles: runFiles.length,
  }, env);
  return { ok: reasons.length === 0 && s.reasons.length === 0 && run.incomplete.length === 0, line: s.line, failed, flaky, reasons: [...reasons, ...s.reasons, ...run.incomplete.map((c) => `chunk ${c} did not report`)] };
}

// ---- CLI ---------------------------------------------------------------------------------------------------------------
function main(argv) {
  const [cmd, ...rest] = argv;
  const opt = {};
  const positional = [];
  for (let i = 0; i < rest.length; i++) {
    if (rest[i].startsWith("--")) opt[rest[i].slice(2)] = rest[++i]; else positional.push(rest[i]);
  }
  const usage = (m) => { console.error(`suite-key: ${m}`); process.exit(2); };
  const repo = opt.repo ?? REPO;
  try {
    if (cmd === "hash-dir") { if (!positional[0]) usage("hash-dir needs a DIR"); console.log(hashDir(positional[0])); return 0; }
    if (cmd === "verdict") {
      if (!opt.out || !opt.chunks) usage("verdict needs --out and --chunks");
      const v = verdict(opt.out, Number(opt.chunks), { store: opt.store ?? null, tree: opt.tree ?? null, repo });
      console.log(v.line);
      if (v.flaky.length) console.error(`flaky (passed on retry):\n  ${v.flaky.join("\n  ")}`);
      if (v.failed.length) console.error(`failed:\n  ${v.failed.join("\n  ")}`);
      if (v.reasons.length) console.error(`not accepted:\n  ${v.reasons.join("\n  ")}`);
      return v.ok ? 0 : 1;
    }
    if (cmd === "record") {
      if (!opt.store || !opt.out || !opt.chunks) usage("record needs --store, --out and --chunks");
      const r = record(opt.store, opt.out, Number(opt.chunks), { repo });
      console.log(`suite-key: records written: ${r.green} green, ${r.red} red; not recorded: ${r.unrecorded.length}`);
      for (const u of r.unrecorded.slice(0, 30)) console.log(`  ${u}`);
      return 0;
    }
    const t = openTree(repo, opt.tree ?? "HEAD");
    if (cmd === "site-key") { const s = siteKey(t, { buildEnv: opt["build-env"] ?? "" }); console.log(JSON.stringify({ tree: t.tree, ...s }, null, 1)); return 0; }
    if (cmd === "keys") { console.log(JSON.stringify(Object.fromEntries(specKeys(t, { config: opt.config })), null, 1)); return 0; }
    if (cmd === "plan" || cmd === "accept") {
      if (!opt.store) usage(`${cmd} needs --store`);
      const p = plan(t, opt.store, { config: opt.config, publicDir: opt.public, buildEnv: opt["build-env"] ?? "", runtime: opt.runtime });
      if (opt.out) { fs.mkdirSync(path.dirname(opt.out), { recursive: true }); fs.writeFileSync(opt.out, JSON.stringify(p, null, 1) + "\n"); }
      console.log(`suite-key: site ${p.siteKey.slice(0, 12)}: ${p.reused.length} of ${p.files} spec files have a green record (${p.totals.expected} tests), ${p.run.length} to run` +
        (p.unkeyable.length ? ` (${p.unkeyable.length} without a key: ${p.unkeyable.slice(0, 3).map((u) => `${u.file}: ${u.why}`).join("; ")})` : ""));
      if (cmd === "accept") {
        const a = acceptance(p);
        if (a.ok) console.log(a.line); else for (const r of a.reasons) console.error(`suite-key: not accepted: ${r}`);
        return a.ok ? 0 : 1;
      }
      return 0;
    }
    usage(`unknown command ${cmd ?? ""}`);
  } catch (e) {
    if (e instanceof KeyError) { console.error(`suite-key: no key: ${e.message}`); return 3; }
    throw e;
  }
  return 2;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) process.exit(main(process.argv.slice(2)));
