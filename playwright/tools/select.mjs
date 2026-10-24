#!/usr/bin/env node
// Changed-files spec selector (backlog row 152a).
//
//   node playwright/tools/select.mjs [--range A..B] [--files f1 f2 ...] [--explain] [--matrix] [--format list|args|json]
//                                    [--ssg-dir DIR] [--no-css-narrow] [--no-titles] [--pages c:button,site:home]
//
// Prints, one per line and relative to playwright/, the spec files that cover what a diff touched. No browser, no
// build, no network: it reads the working tree and `git diff --name-only`.
//
// Default diff: everything this branch changed against origin/main (`origin/main...HEAD`) plus uncommitted changes.
//
// The map (a COVER relation, deliberately over-inclusive: a missed spec is a hole, an extra one is only minutes):
//   preview/src/components/<c>/**            -> specs about component <c>, plus all-routes' test of <c>'s page (the
//                                               page's at-rest axe scan lives only there, row 152)
//   primitives/src/<p>.rs | <p>/** | js/ts   -> specs of every component that uses primitive <p>, directly or through
//                                               another primitive (transitive), plus those components' all-routes
//                                               tests; if that is > BROAD_AT components the change is shared plumbing
//                                               and the BROAD set is added
//   theme / main.css / main.rs / eager_head / index.html / components/mod.rs / scripts that build the site
//                                            -> the BROAD set (below); a changed preview/assets/*.css is read rule by rule
//                                               first and often needs only part of it (see "CSS finer", below)
//   playwright/<spec>.spec.ts                -> itself; playwright/<helper>.ts -> the specs that import it (transitively);
//                                               fixtures.ts / base-url.ts / *.config.ts -> every spec
//   docs, dev-docs, scripts/check-*, cli, templates, component.json -> no spec (their own gates cover them)
// "About component C" = the file is named for C (longest component-name prefix of the stem: carousel-drag-free ->
// carousel, chart_tooltip_follow -> chart_tooltip) OR its source names C as a page (`name=C`, `component/C`) or as a
// quoted subject. A spec that names more than MATRIX_AT components is a MATRIX spec (a route walk or a subject
// matrix: hydration-parity, main-thread, dx-class-migration, ...): it covers every component, so it is part of the
// BROAD set and is added for a single-component change only with --matrix (run it with `--grep <component>` instead).
//
// The BROAD set = every matrix spec + BROAD_EXTRA (the cheap cross-cutting guards). main-thread is release-only and
// is listed but skipped by the configs on a debug build (base-url.ts).
//
// CSS finer than "a theme file changed" (backlog row 323). A changed `preview/assets/*.css` is diffed RULE by RULE
// against the base of the range (css-rules.mjs: an edited rule is one change, a moved rule is a change, a comment is
// none), each changed rule's constraining classes / ids are placed on the pages of the SSG build under test
// (site-index.mjs: --ssg-dir, SELECT_SSG_DIR / SSG_SITE_DIR, or a crawl of PLAYWRIGHT_BASE_URL -- re-read every run, so
// it is never stale against the build being tested), and custom properties / keyframes are traced to the rules that
// read them (css-impact.mjs; the soundness argument is written at its top). Instead of the whole BROAD set that selects:
//   - every BROAD spec that cannot tell pages apart, whole -- but only when some page is reached at all;
//   - for the matrix specs whose test titles name the page they visit (TITLE_SCOPED in select-titles.mjs), the tests
//     whose page is reached plus every test whose title names no page; the others are dropped with a `--grep-invert`
//     pattern (`grepInvert` in the json output, passed to Playwright by tools/test-changed.sh);
//   - the specs about the component that OWNS a changed class (the theme sheet carries hoisted component parts), the
//     specs of a site page for a class that lives only on site pages, and every spec that spells a changed class or a
//     rarely spelled custom property by name.
// Anything the argument does not cover (a selector with no class or id, @font-face, a token read by script, a class
// that cannot be placed, no SSG build to read, --no-css-narrow) keeps the BROAD set; `--explain` says which and why.
// --pages c:calendar,site:home is a diagnostic: pretend the changed rules reach exactly those pages (the git diff is ignored
// unless --files / --range name one).
// --self-test runs the node-only test of all this (scripts/check-select-css.sh).
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { loadIndex } from "./site-index.mjs";
import { buildContext, impactOfChanges } from "./css-impact.mjs";
import { makeTitleScopes } from "./select-titles.mjs";

const PW = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const ROOT = path.resolve(PW, "..");
const MATRIX_AT = 12; // a spec naming more components than this walks the site, it is not "about" any of them
const BROAD_AT = 30; // a primitive used by more components than this is shared plumbing
const BROAD_EXTRA = [
  "preview.spec.ts", "all-routes.spec.ts", "layout-stability.spec.ts", "focus-ring.spec.ts", "user-select.spec.ts",
  "popup-style-isolation.spec.ts", "computed-style-snapshot.spec.ts", "dark-surface-contrast.spec.ts",
  "site-brand.spec.ts", "docs-structure.spec.ts", "docs-nav-sort.spec.ts", "docs-button-parity.spec.ts",
  "theme-all-reset.spec.ts", "theme-dark-class.spec.ts", "theme-preset.spec.ts", "theme-preset-contrast.spec.ts",
  "theme-picker-phone.spec.ts", "nav-scroll-instant.spec.ts", "press-feedback.spec.ts", "hover-mix-space.spec.ts",
  "component-catalog.spec.ts",
];

const args = process.argv.slice(2);
if (args.includes("--self-test")) await import("./select-selftest.mjs"); // node only; exits with its own status
const opt = { range: null, files: null, explain: false, matrix: false, format: "list", ssgDir: null, pages: null, cssNarrow: true, titles: true };
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === "--range") opt.range = args[++i];
  else if (a === "--files") { opt.files = []; while (i + 1 < args.length && !args[i + 1].startsWith("--")) opt.files.push(args[++i]); }
  else if (a === "--explain") opt.explain = true;
  else if (a === "--matrix") opt.matrix = true;
  else if (a === "--format") opt.format = args[++i];
  else if (a === "--ssg-dir") opt.ssgDir = args[++i];
  else if (a === "--pages") opt.pages = args[++i].split(",").filter(Boolean);
  else if (a === "--no-css-narrow") opt.cssNarrow = false;
  else if (a === "--no-titles") opt.titles = false;
  else { console.error(`select: unknown argument ${a}`); process.exit(2); }
}

const git = (...a) => execFileSync("git", a, { cwd: ROOT, encoding: "utf8" }).split("\n").filter(Boolean);
const gitText = (...a) => { try { return execFileSync("git", a, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], maxBuffer: 1 << 28 }); } catch { return null; } };
// the two sides of a rule-level diff: the base of the range, and the end of it (the working tree when the range ends at HEAD)
const rangeSpec = opt.range ?? "origin/main...HEAD";
const threeDot = rangeSpec.includes("...");
const [rangeA, rangeB] = rangeSpec.split(threeDot ? "..." : "..").map((x) => x || "HEAD");
const oldRef = threeDot ? (gitText("merge-base", rangeA, rangeB) ?? rangeA).trim() : rangeA;
const endIsWorktree = (gitText("rev-parse", rangeB) ?? "").trim() === (gitText("rev-parse", "HEAD") ?? "?").trim();
const oldTextOf = (f) => gitText("show", `${oldRef}:${f}`) ?? "";
const newTextOf = (f) => endIsWorktree ? (fs.existsSync(path.join(ROOT, f)) ? fs.readFileSync(path.join(ROOT, f), "utf8") : "") : (gitText("show", `${rangeB}:${f}`) ?? "");
let changed;
if (opt.files) changed = opt.files;
else if (opt.pages && !opt.range) changed = []; // --pages pretends the changed rules: the branch's own diff (and its at-rest component tests) would leak in
else {
  const range = opt.range ?? "origin/main...HEAD";
  changed = [...new Set([...git("diff", "--name-only", range), ...git("diff", "--name-only", "HEAD"), ...git("ls-files", "--others", "--exclude-standard")])];
}

// ---- inventory -----------------------------------------------------------------------------------------------
const walk = (dir, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (e.name === "node_modules" || e.name === "fixtures" || e.name === "reference") continue;
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, out); else out.push(p);
  }
  return out;
};
const pwFiles = walk(PW).map((p) => path.relative(PW, p));
const specs = pwFiles.filter((f) => f.endsWith(".spec.ts"));
const comps = fs.readdirSync(path.join(ROOT, "preview/src/components"), { withFileTypes: true })
  .filter((e) => e.isDirectory()).map((e) => e.name).sort((a, b) => b.length - a.length);
const read = (f) => fs.readFileSync(path.join(PW, f), "utf8");
const noComments = (t) => t.replace(/^\s*\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");
const src = Object.fromEntries(pwFiles.filter((f) => f.endsWith(".ts")).map((f) => [f, read(f)]));

// component(s) a spec is about
const norm = (s) => s.replace(/-/g, "_");
const stemOf = (f) => norm(path.basename(f).replace(/\.spec\.ts$/, ""));
const aboutComps = new Map();
for (const f of specs) {
  const set = new Set();
  const stem = stemOf(f);
  const byName = comps.find((c) => stem === c || stem.startsWith(c + "_"));
  if (byName) set.add(byName);
  const text = src[f];
  for (const c of comps) {
    const dash = c.replace(/_/g, "-");
    // a quoted token right after getByRole( / role= / role: / haspopup is an ARIA role, not a component subject
    const re = new RegExp(`(?:name=|component/|(?<!getByRole\\(|role=|role:\\s|haspopup=|haspopup:\\s|aria-\\w+=)["'\`])(?:${c}|${dash})(?=[&/"'\`?])`);
    if (re.test(text)) set.add(c);
  }
  aboutComps.set(f, set);
}
const matrixSpecs = new Set(specs.filter((f) => aboutComps.get(f).size > MATRIX_AT));
// all-routes.spec.ts spells a few component names in its own route-list check; it is selected per page below (AT_REST)
const specsOf = (c) => specs.filter((f) => aboutComps.get(f).has(c) && !matrixSpecs.has(f) && f !== "all-routes.spec.ts");

// primitive -> consuming components (transitive through primitives)
const primDir = path.join(ROOT, "primitives/src");
const primMods = [...new Set(fs.readdirSync(primDir).map((n) => n.replace(/\.(rs|js)$/, "")))].filter((n) => !["lib", "ts", "js"].includes(n));
const primText = (m) => {
  const f = path.join(primDir, m + ".rs"), d = path.join(primDir, m);
  let t = fs.existsSync(f) ? noComments(fs.readFileSync(f, "utf8")) : "";
  if (fs.existsSync(d) && fs.statSync(d).isDirectory()) for (const x of walk(d)) if (x.endsWith(".rs")) t += noComments(fs.readFileSync(x, "utf8"));
  return t;
};
const compText = (c) => walk(path.join(ROOT, "preview/src/components", c)).filter((x) => /\.(rs|toml|json)$/.test(x)).map((x) => noComments(fs.readFileSync(x, "utf8"))).join("\n");
const compTexts = Object.fromEntries(comps.map((c) => [c, compText(c)]));
const usesPrim = (text, m, ns) => new RegExp(`${ns}::(?:${m}\\b|\\{[^}]*\\b${m}\\b[^}]*\\})`, "s").test(text);
const primTexts = Object.fromEntries(primMods.map((m) => [m, primText(m)]));
const primUsers = new Map(); // module -> primitive modules that use it
for (const m of primMods) primUsers.set(m, primMods.filter((o) => o !== m && usesPrim(primTexts[o], m, "crate")));
function compsUsingPrim(m) {
  const seen = new Set([m]), queue = [m], out = new Set();
  while (queue.length) {
    const p = queue.pop();
    for (const c of comps) if (usesPrim(compTexts[c], p, "dioxus_primitives")) out.add(c);
    for (const u of primUsers.get(p) ?? []) if (!seen.has(u)) { seen.add(u); queue.push(u); }
  }
  return out;
}

// helper -> importing specs (transitive)
function importersOf(helper) {
  const hit = new Set(), queue = [helper];
  const base = (f) => path.basename(f).replace(/\.ts$/, "");
  while (queue.length) {
    const h = queue.pop();
    for (const f of Object.keys(src)) {
      if (f === h || hit.has(f)) continue;
      if (new RegExp(`from\\s+["'][./]*(?:[\\w./-]*/)?${base(h)}["']`).test(src[f])) { hit.add(f); if (!f.endsWith(".spec.ts")) queue.push(f); }
    }
  }
  return [...hit].filter((f) => f.endsWith(".spec.ts"));
}

// ---- map the diff --------------------------------------------------------------------------------------------
const sel = new Map(); // spec -> reasons
const add = (spec, why) => { if (!sel.has(spec)) sel.set(spec, new Set()); sel.get(spec).add(why); };
const addBroad = (why) => { for (const f of matrixSpecs) add(f, why); for (const f of BROAD_EXTRA) if (specs.includes(f)) add(f, why); };
const addAll = (why) => { for (const f of specs) add(f, why); };
const noSpec = [];
// The at-rest axe scan of a component page lives only in all-routes.spec.ts (backlog row 152), so a component's own specs
// do not cover it: every changed component (directly, or through a primitive it uses) adds its all-routes test(s).
const AT_REST = "all-routes.spec.ts";
const atRestKeys = new Set();
const cssChanged = []; // theme / site sheets: resolved rule by rule after the loop (row 323)
const isSheet = (f) => /^preview\/assets\/[^/]+\.css$/.test(f);

for (const f of changed) {
  let m;
  if ((m = f.match(/^preview\/src\/components\/([^/]+)\//)) && comps.includes(m[1])) {
    for (const s of specsOf(m[1])) add(s, f);
    atRestKeys.add(`c:${m[1]}`);
    if (opt.matrix) for (const s of matrixSpecs) add(s, `${f} (matrix)`);
  } else if (isSheet(f) && opt.cssNarrow) {
    cssChanged.push(f);
  } else if (f === "preview/src/components/mod.rs" || /^preview\/(assets\/|index\.html|build\.rs|Cargo\.toml)/.test(f) ||
             /^preview\/src\/(main|theme|eager_head|docs_nav|nav_link|motion_docs|installed_source|wasm_libc_shim)\.rs$/.test(f) ||
             /^preview\/src\/(i18n|dashboard)\//.test(f) || /^scripts\/(build-ssg|ssg-verify|ssg-css-bundle|emit-404)/.test(f)) {
    addBroad(f);
  } else if ((m = f.match(/^preview\/src\/(\w+?)_parity\.rs$/)) || (m = f.match(/^preview\/src\/(charts_gallery)\.rs$/))) {
    const c = comps.find((x) => m[1] === x || m[1].startsWith(x)) ?? "chart";
    for (const s of specsOf(c)) add(s, f);
    for (const s of specs.filter((s) => /^(chart|charts)/.test(path.basename(s)))) add(s, f);
  } else if ((m = f.match(/^primitives\/src\/(?:ts\/|js\/)?([^/.]+)(?:\.[a-z]+|\/)/)) && primMods.includes(m[1])) {
    const users = compsUsingPrim(m[1]);
    for (const c of users) for (const s of specsOf(c)) add(s, `${f} -> ${c}`);
    if (users.size <= BROAD_AT) for (const c of users) atRestKeys.add(`c:${c}`);
    if (users.size > BROAD_AT) addBroad(`${f} (shared plumbing: ${users.size} components)`);
    else if (opt.matrix) for (const s of matrixSpecs) add(s, `${f} (matrix)`);
    if (users.size === 0) addBroad(`${f} (no component uses it: assume shared)`);
  } else if (/^primitives\//.test(f) || /^dioxus-attributes\//.test(f) || /^Cargo\.(toml|lock)$/.test(f)) {
    addBroad(f);
  } else if (f.startsWith("playwright/")) {
    const rel = f.slice("playwright/".length);
    if (rel.endsWith(".spec.ts")) { if (specs.includes(rel)) add(rel, f); }
    else if (/(^|\/)(fixtures\.ts|base-url\.ts|[\w.]*config\.ts)$/.test(rel)) addAll(f);
    else if (rel.endsWith(".ts") && !rel.startsWith("tools/")) { for (const s of importersOf(rel)) add(s, f); }
    else noSpec.push(f);
  } else noSpec.push(f);
}

// ---- CSS: rule-level impact (row 323) ------------------------------------------------------------------------
const TITLE_SCOPED = makeTitleScopes(comps);
// A site page's own specs: the page's route is spelled in the spec (not the page's word: prose says "docs" everywhere).
const SITE_SPEC_RE = {
  "site:home": /(?:BASE(?:_URL)?|HOST|origin|baseURL)\}\/[`"'?#]|goto\w*\([^)]*["'`]\/["'`?]/,
  "site:docs": /["'`/}]\/docs[/"'`?#]/,
  "site:demos": /["'`/}]\/demos[/"'`?#]/,
  "site:charts": /["'`/}]\/charts[/"'`?#]/,
  "site:dashboard": /\/dashboard\/email-client/,
  "site:component-index": /\/component\/(?:block\/)?[`"'?]/,
};

function listTitles(files) {
  // `playwright test --list` LOADS each file: a spec that builds while it loads (hydration-parity runs `cargo build`
  // for attr-synth at load time, outside the build lock) is never listed; it is kept whole instead
  files = files.filter((f) => !/(?:spawnSync|execSync|execFileSync|spawn)\(\s*["'`]cargo|cargo build/.test(src[f] ?? ""));
  if (!files.length) return new Map();
  const env = { ...process.env, PLAYWRIGHT_BASE_URL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:1" };
  const r = spawnSync("pnpm", ["exec", "playwright", "test", "-c", "ssg.local.config.ts", "--list", "--reporter=json", ...files], { cwd: PW, env, encoding: "utf8", maxBuffer: 1 << 28 });
  if (r.status !== 0 || !r.stdout) return null;
  let j;
  try { j = JSON.parse(r.stdout); } catch { return null; }
  const out = new Map();
  const walkSuite = (suite, trail, file) => {
    for (const sp of suite.specs ?? []) {
      const f = sp.file ?? file;
      if (!out.has(f)) out.set(f, []);
      out.get(f).push({ path: [...trail, sp.title] });
    }
    for (const c of suite.suites ?? []) walkSuite(c, c.file && c.title === c.file ? trail : [...trail, c.title], c.file ?? file);
  };
  for (const top of j.suites ?? []) walkSuite(top, [], top.file);
  return out;
}

const MENTION_MAX = 12; // a custom property spelled by more specs than this is plumbing, not a probe
const reEsc = (t) => t.replace(/[.*+?^${}()|[\]\\/-]/g, "\\$&");
let grepInvert = null;
const cssReport = { notes: [], broad: [], pages: [], mentions: [], owners: [], droppedTests: 0, keptTests: 0, dropped: [] };

if (cssChanged.length || opt.pages) {
  // --pages is a diagnostic: pretend the changed rules reach exactly these page keys (no SSG build needed)
  let { index, why } = opt.pages ? { index: { routes: [], classes: new Map(), ids: new Map(), origin: "--pages" } } : await loadIndex({ ssgDir: opt.ssgDir ?? process.env.SELECT_SSG_DIR ?? process.env.SSG_SITE_DIR });
  // A client-rendered dev server answers every path with the same empty shell: an index read from it would call every
  // class "dead" and select nothing. Only a prerendered site (a page per component, hundreds of classes) is believed.
  if (index && !opt.pages && (index.routes.length < comps.length * 0.8 || index.classes.size < 100)) {
    why = `${index.origin} is not a prerendered SSG build (${index.routes.length} pages, ${index.classes.size} classes)`;
    index = null;
  }
  if (!index) {
    for (const f of cssChanged) addBroad(`${f} (${why})`);
    cssReport.notes.push(`css: ${why}; the whole BROAD set stays`);
    console.error(`select: ${why}; a changed stylesheet keeps the whole BROAD set`);
  } else {
    cssReport.notes.push(`css: class index from ${index.origin}: ${index.routes.length} pages, ${index.classes.size} classes`);
    const ctx = buildContext({ root: ROOT, index, comps });
    const r = opt.pages
      ? { broad: [], notes: [], keys: new Map(opt.pages.map((k) => [k, ["--pages"]])), mentions: new Map(), mentionKinds: new Map(), ownerComps: new Map(), siteOwned: [] }
      : impactOfChanges(ctx, cssChanged.map((f) => ({ file: f, oldText: oldTextOf(f), newText: newTextOf(f) })));
    cssReport.notes.push(...r.notes);
    cssReport.broad = r.broad;
    const PK = new Set(r.keys.keys());
    cssReport.pages = [...PK];
    if (r.broad.length || PK.has("site:other")) {
      const why = r.broad[0] ?? "a page outside the known routes";
      addBroad(`${cssChanged.join(", ") || "--pages"}: ${why}${r.broad.length > 1 ? ` (+${r.broad.length - 1} more)` : ""}`);
    } else if (PK.size) {
      const G = new Set([...matrixSpecs, ...BROAD_EXTRA.filter((x) => specs.includes(x))]);
      const scoped = [...G].filter((f) => TITLE_SCOPED[f]);
      const titles = opt.titles && scoped.length ? listTitles(scoped) : null;
      const why = `${cssChanged.join(", ") || "--pages"}: rules reach ${PK.size} page key(s)`;
      for (const f of G) {
        const tests = TITLE_SCOPED[f] ? titles?.get(f) : null;
        if (!tests) { add(f, `${why} (spec cannot tell pages apart)`); continue; }
        const reach = f === AT_REST ? new Set([...PK, ...atRestKeys]) : PK;
        const kept = tests.filter((t) => { const ks = TITLE_SCOPED[f](t.path.join(" ")); return !ks || ks.length === 0 || ks.some((k) => reach.has(k)); });
        cssReport.keptTests += kept.length; cssReport.droppedTests += tests.length - kept.length;
        if (kept.length === 0) continue;
        add(f, `${why}: ${kept.length}/${tests.length} tests`);
        if (kept.length < tests.length) {
          const keptSet = new Set(kept);
          cssReport.dropped.push({ file: f, kept: kept.length, total: tests.length, titles: tests.filter((t) => !keptSet.has(t)).map((t) => t.path.join(" ")) });
        }
      }
    } else cssReport.notes.push("css: the changed rules reach no page (dead or unrendered CSS): no spec selected by page");
    // owners and names: independent of the page set, so they apply to a broad result too
    for (const [c, names] of r.ownerComps) if (comps.includes(c)) for (const sp of specsOf(c)) add(sp, `${cssChanged.join(", ")}: owner component ${c} (${names.slice(0, 2).join(", ")})`);
    if (!r.broad.length && PK.size && ![...PK].some((k) => k.startsWith("c:"))) {
      // a class that lives only on site pages (the home hero, the dashboard): those pages' own specs observe it
      for (const k of PK) { const re = SITE_SPEC_RE[k]; if (re) for (const sp of specs) if (!matrixSpecs.has(sp) && !BROAD_EXTRA.includes(sp) && re.test(src[sp])) add(sp, `${cssChanged.join(", ")}: site-owned class on ${k}`); }
    }
    for (const [name] of r.mentions) {
      const re = new RegExp(`(?<![\\w-])${name.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&")}(?![\\w-])`);
      const hits = new Map(); // spec -> via
      for (const [f, text] of Object.entries(src)) {
        if (!re.test(text) || f.startsWith("tools/")) continue;
        if (f.endsWith(".spec.ts")) hits.set(f, `names ${name}`);
        else for (const sp of importersOf(f)) hits.set(sp, `${f} names ${name}`);
      }
      // A custom property that a dozen specs spell is background plumbing (`--dx-background`): those specs read it, they
      // do not probe it, and the broad set is its cover. A class or a rarely spelled property is something a spec probes.
      const isToken = r.mentionKinds.get(name) === "var" || r.mentionKinds.get(name) === "keyframes";
      if (isToken && hits.size > MENTION_MAX) { cssReport.notes.push(`css: ${name} is spelled in ${hits.size} specs (more than ${MENTION_MAX}): not used to select`); continue; }
      for (const [sp, why] of hits) add(sp, why);
      cssReport.mentions.push(name);
    }
  }
}

// ---- the at-rest axe scan of the changed components (row 152) ------------------------------------------------
if (atRestKeys.size && !sel.has(AT_REST) && specs.includes(AT_REST)) {
  const why = `at-rest axe of ${[...atRestKeys].map((k) => k.slice(2)).sort().join(", ")}`;
  const tests = opt.titles && TITLE_SCOPED[AT_REST] ? listTitles([AT_REST])?.get(AT_REST) : null;
  if (!tests) add(AT_REST, `${why} (titles not listed: whole file)`);
  else {
    const kept = tests.filter((t) => { const ks = TITLE_SCOPED[AT_REST](t.path.join(" ")); return !ks || ks.length === 0 || ks.some((k) => atRestKeys.has(k)); });
    add(AT_REST, `${why}: ${kept.length}/${tests.length} tests`);
    if (kept.length < tests.length) {
      const keptSet = new Set(kept);
      cssReport.dropped.push({ file: AT_REST, kept: kept.length, total: tests.length, titles: tests.filter((t) => !keptSet.has(t)).map((t) => t.path.join(" ")) });
    }
  }
}

// the drop pattern: one alternative per file, exact title paths, anchored at the end
if (cssReport.dropped.length) {
  const parts = cssReport.dropped.filter((d) => sel.has(d.file)).map((d) => `(?<![\\w/.-])${reEsc(d.file)} (?:${d.titles.map((t) => reEsc(t) + "$").join("|")})`);
  let pat = parts.join("|");
  if (pat.length > 100000) { cssReport.notes.push(`css: drop pattern is ${pat.length} chars: not applied`); pat = ""; }
  grepInvert = pat || null;
}


const out = [...sel.keys()].sort();
if (opt.explain) {
  for (const s of out) console.error(`${s}\t${[...sel.get(s)].slice(0, 3).join(", ")}${sel.get(s).size > 3 ? ", ..." : ""}`);
  if (noSpec.length) console.error(`# no spec maps to: ${noSpec.join(" ")}`);
  for (const n of cssReport.notes) console.error(`# ${n}`);
  if (cssReport.broad.length) console.error(`# css falls back to the broad set: ${cssReport.broad.slice(0, 5).join(" | ")}${cssReport.broad.length > 5 ? ` | (+${cssReport.broad.length - 5})` : ""}`);
  if (cssReport.pages.length) console.error(`# css reaches: ${cssReport.pages.sort().join(" ")}`);
  if (cssReport.mentions.length) console.error(`# css names (specs that spell them are selected): ${cssReport.mentions.slice(0, 12).join(" ")}${cssReport.mentions.length > 12 ? " ..." : ""}`);
  for (const d of cssReport.dropped) console.error(`# narrowed ${d.file}: ${d.kept} of ${d.total} tests`);
  console.error(`# ${out.length} of ${specs.length} specs; matrix specs (${matrixSpecs.size}): ${[...matrixSpecs].sort().join(" ")}`);
}
if (opt.format === "json") console.log(JSON.stringify({ specs: out, total: specs.length, noSpec, grepInvert, narrowed: cssReport.dropped.filter((d) => sel.has(d.file)).map(({ file, kept, total, titles }) => ({ file, kept, total, dropped: titles })), reasons: Object.fromEntries(out.map((f) => [f, [...sel.get(f)]])), css: { broad: cssReport.broad, pages: cssReport.pages } }, null, 2));
else if (opt.format === "args") console.log(out.join(" "));
else for (const s of out) console.log(s);
