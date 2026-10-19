#!/usr/bin/env node
// Changed-files spec selector (backlog row 152a).
//
//   node playwright/tools/select.mjs [--range A..B] [--files f1 f2 ...] [--explain] [--matrix] [--format list|args|json]
//
// Prints, one per line and relative to playwright/, the spec files that cover what a diff touched. No browser, no
// build, no network: it reads the working tree and `git diff --name-only`.
//
// Default diff: everything this branch changed against origin/main (`origin/main...HEAD`) plus uncommitted changes.
//
// The map (a COVER relation, deliberately over-inclusive: a missed spec is a hole, an extra one is only minutes):
//   preview/src/components/<c>/**            -> specs about component <c>
//   primitives/src/<p>.rs | <p>/** | js/ts   -> specs of every component that uses primitive <p>, directly or through
//                                               another primitive (transitive); if that is > BROAD_AT components the
//                                               change is shared plumbing and the BROAD set is added
//   theme / main.css / main.rs / eager_head / index.html / components/mod.rs / scripts that build the site
//                                            -> the BROAD set (below)
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
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

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
const opt = { range: null, files: null, explain: false, matrix: false, format: "list" };
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === "--range") opt.range = args[++i];
  else if (a === "--files") { opt.files = []; while (i + 1 < args.length && !args[i + 1].startsWith("--")) opt.files.push(args[++i]); }
  else if (a === "--explain") opt.explain = true;
  else if (a === "--matrix") opt.matrix = true;
  else if (a === "--format") opt.format = args[++i];
  else { console.error(`select: unknown argument ${a}`); process.exit(2); }
}

const git = (...a) => execFileSync("git", a, { cwd: ROOT, encoding: "utf8" }).split("\n").filter(Boolean);
let changed;
if (opt.files) changed = opt.files;
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
const specsOf = (c) => specs.filter((f) => aboutComps.get(f).has(c) && !matrixSpecs.has(f));

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

for (const f of changed) {
  let m;
  if ((m = f.match(/^preview\/src\/components\/([^/]+)\//)) && comps.includes(m[1])) {
    for (const s of specsOf(m[1])) add(s, f);
    if (opt.matrix) for (const s of matrixSpecs) add(s, `${f} (matrix)`);
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

const out = [...sel.keys()].sort();
if (opt.explain) {
  for (const s of out) console.error(`${s}\t${[...sel.get(s)].slice(0, 3).join(", ")}${sel.get(s).size > 3 ? ", ..." : ""}`);
  if (noSpec.length) console.error(`# no spec maps to: ${noSpec.join(" ")}`);
  console.error(`# ${out.length} of ${specs.length} specs; matrix specs (${matrixSpecs.size}): ${[...matrixSpecs].sort().join(" ")}`);
}
if (opt.format === "json") console.log(JSON.stringify({ specs: out, total: specs.length, noSpec }, null, 2));
else if (opt.format === "args") console.log(out.join(" "));
else for (const s of out) console.log(s);
