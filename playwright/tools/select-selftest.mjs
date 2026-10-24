// Self-test of the spec selector's CSS half (backlog row 323). Node only: no browser, no build, no network (the
// title check runs `playwright test --list`, which loads the spec files and never starts a browser).
//
//   node playwright/tools/select.mjs --self-test          (scripts/check-select-css.sh runs exactly this)
//
// 1. css-rules.mjs: the parser and the rule-level differ (comments, strings, at-rule contexts, in-place edit vs move);
// 2. analyzeSelector: which classes / ids a selector REQUIRES, and what it leaves unconstrained;
// 3. css-impact.mjs on a synthetic site: page placement, shell-wide classes, closed-popup siblings, dead classes,
//    custom properties traced to their readers, keyframes, and every fallback to the broad set;
// 4. the title shapes of the matrix specs (select-titles.mjs) against the real spec files: a spec whose titles do not
//    follow the shape the table expects fails here, instead of silently keeping every test;
// 5. the `--grep-invert` pattern select.mjs builds really drops exactly the dropped tests (escaping, anchoring);
// 6. the real tree: `--pages` narrows the biggest matrix spec and the broad behaviour comes back with --no-css-narrow.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { parseCss, diffRules, analyzeSelector } from "./css-rules.mjs";
import { fromDir } from "./site-index.mjs";
import { buildContext, impactOfChanges, routeKey } from "./css-impact.mjs";
import { makeTitleScopes } from "./select-titles.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const PW = path.resolve(HERE, "..");
const ROOT = path.resolve(PW, "..");
let failed = 0, passed = 0;
const check = (name, ok, detail = "") => {
  if (ok) passed++;
  else { failed++; console.error(`FAIL ${name}${detail ? `\n     ${detail}` : ""}`); }
};
const eq = (name, got, want) => check(name, JSON.stringify(got) === JSON.stringify(want), `got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
const same = (a, b) => JSON.stringify([...a].sort()) === JSON.stringify([...b].sort());

// ---- 1. parser and differ ----------------------------------------------------------------------------------------
{
  const css = `/* c */ .a, .b > .c { color: red; /* x */ margin: 0 } @media (min-width: 10px) { .a { color: blue } }
    @keyframes k { from { opacity: 0 } to { opacity: 1 } } @font-face { font-family: "x;{y"; src: url(a.woff2) }
    @import "z.css"; :root { --x: calc(1px + 2px); --y: "}" }`;
  const r = parseCss(css);
  eq("parse: kinds in source order", r.map((x) => x.kind), ["rule", "rule", "at", "at", "stmt", "rule"]);
  eq("parse: media context", r[1].ctx, ["@media (min-width: 10px)"]);
  eq("parse: selector list normalised", r[0].sel, ".a, .b>.c");
  eq("parse: declarations", r[0].decls, [["color", "red"], ["margin", "0"]]);
  eq("parse: a string holding } does not end the rule", r[5].decls.map((d) => d[0]), ["--x", "--y"]);

  const a = `.a { color: red; margin: 0 } .b { top: 0 } .c { left: 0 }`;
  check("diff: whitespace and comments are not a change", diffRules(a, `/* hi */ .a{color:red;margin:0}\n.b {\n top: 0 }\n.c{left:0}`).same);
  const edit = diffRules(a, `.a { color: blue; margin: 0 } .b { top: 0 } .c { left: 0 }`);
  eq("diff: an edited rule is one removed + one added", [edit.removed.length, edit.added.length], [1, 1]);
  const moved = diffRules(a, `.b { top: 0 } .a { color: red; margin: 0 } .c { left: 0 }`);
  check("diff: a MOVED rule is a change (the cascade order changed)", !moved.same);
  const ins = diffRules(a, `.a { color: red; margin: 0 } .n { x: y } .b { top: 0 } .c { left: 0 }`);
  eq("diff: an insertion is one added rule", [ins.removed.length, ins.added.length], [0, 1]);
  const ctxChange = diffRules(`.a { x: y }`, `@media (min-width: 1px) { .a { x: y } }`);
  check("diff: the same rule moved into @media is a change", !ctxChange.same);
}

// ---- 2. selectors ------------------------------------------------------------------------------------------------
{
  const terms = (sel) => analyzeSelector(sel).map((a) => (a.unconstrained ? "*" : [...a.terms.map((t) => (t.kind === "id" ? "#" : ".") + t.name), ...a.groups.map((g) => "(" + g.map((x) => x.terms.map((t) => t.name).join("&")).join("|") + ")")].join(" ")));
  eq("sel: class", terms(".dx-a"), [".dx-a"]);
  eq("sel: compound needs every class", terms(".dx-a.dx-b:hover::before"), [".dx-a .dx-b"]);
  eq("sel: descendant: the subject constrains", terms(".dx-a .dx-b"), [".dx-b"]);
  eq("sel: element subject falls to the ancestor class", terms(".dx-a > button"), [".dx-a"]);
  eq("sel: list", terms(".dx-a, #hero p"), [".dx-a", "#hero"]);
  eq("sel: :not() requires nothing", terms("button:not(.dx-x)"), ["*"]);
  eq("sel: :is() is a union", terms(":is(.dx-a, .dx-b) > span"), ["(dx-a|dx-b)"]);
  eq("sel: :where() with an unconstrained alternative requires nothing", terms(":where(.dx-a, button) > span"), ["*"]);
  eq("sel: attribute only", terms('dialog[class^="dx-"]'), ["*"]);
  eq("sel: :root", terms(":root[data-theme=dark]"), ["*"]);
  eq("sel: escaped class", terms(".sm\\:flex"), [".sm:flex"]);
  eq("sel: class inside an attribute value is not a class", terms('[data-x=".fake"]'), ["*"]);
}

// ---- 3. impact on a synthetic site -------------------------------------------------------------------------------
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "select-selftest-"));
try {
  const write = (rel, text) => { const f = path.join(tmp, rel); fs.mkdirSync(path.dirname(f), { recursive: true }); fs.writeFileSync(f, text); };
  const page = (classes, ids = []) => `<html><body>${classes.map((c) => `<div class="${c}"></div>`).join("")}${ids.map((i) => `<p id="${i}"></p>`).join("")}</body></html>`;
  const shell = ["shell", "navbar"];
  write("site/index.html", page([...shell, "hero", "dx-alpha"], ["intro"]));
  write("site/docs/index.html", page([...shell, "docs-title"]));
  write("site/component/alpha/index.html", page([...shell, "dx-alpha", "dx-alpha-trigger", "dx-beta"]));
  write("site/component/beta/index.html", page([...shell, "dx-beta"]));
  write("site/component/gamma/index.html", page([...shell, "dx-gamma"]));
  for (const c of ["alpha", "beta", "gamma"]) write(`preview/src/components/${c}/style.css`, `.dx-${c} { color: red }`);
  // alpha's library code spells a class that only exists inside a closed popup (not in the SSG HTML), next to a trigger that is
  write("preview/src/components/alpha/component.rs", `rsx! { button { class: "dx-alpha-trigger" } div { class: "dx-alpha" } div { class: "dx-alpha-popup" } div { class: format!("dx-alpha-kind-{kind}") } }`);
  write("preview/src/components/beta/component.rs", `rsx! { div { class: "dx-beta" } }`);
  write("preview/src/components/gamma/component.rs", `rsx! { div { class: "dx-gamma", style: "--gamma-gap: 4px" } }`);
  write("preview/src/main.rs", `rsx! { div { class: "shell navbar hero docs-title", id: "intro", style: "--site-inline: 1" } }`);
  write("preview/assets/main.css", `.shell { gap: var(--space) }`);
  write("primitives/src/lib.rs", ``);
  const comps = ["alpha", "beta", "gamma"];
  const index = fromDir(path.join(tmp, "site"));
  const ctx = buildContext({ root: tmp, index, comps });
  const run = (oldCss, newCss, file = "preview/assets/main.css") => impactOfChanges(ctx, [{ file, oldText: oldCss, newText: newCss }]);
  const keys = (r) => [...r.keys.keys()].sort();

  eq("route keys", ["/", "/docs/", "/component/alpha/", "/component/block/alpha/main/", "/charts/bar/", "/component/nope/"].map((r) => routeKey(r, comps)),
    ["site:home", "site:docs", "c:alpha", "c:alpha", "site:charts", "site:other"]);

  let r = run(".hero { color: red }", ".hero { color: blue }");
  eq("impact: a class on one page", [r.broad, keys(r)], [[], ["site:home"]]);
  r = run(".dx-beta { top: 0 }", ".dx-beta { top: 1px }");
  eq("impact: a class on two pages", [r.broad, keys(r)], [[], ["c:alpha", "c:beta"]]);
  r = run(".dx-alpha .dx-beta { top: 0 }", ".dx-alpha .dx-beta { top: 1px }");
  eq("impact: the subject decides (a descendant rule reaches the subject's pages)", keys(r), ["c:alpha", "c:beta"]);
  r = run(".dx-alpha.dx-beta { top: 0 }", ".dx-alpha.dx-beta { top: 1px }");
  eq("impact: a compound needs every class on the page", keys(r), ["c:alpha"]);
  r = run(".shell { top: 0 }", ".shell { top: 1px }");
  eq("impact: a shell class reaches every page", keys(r), ["c:alpha", "c:beta", "c:gamma", "site:docs", "site:home"]);
  r = run("#intro { top: 0 }", "#intro { top: 1px }");
  eq("impact: an id", keys(r), ["site:home"]);
  r = run(".dx-alpha-popup { top: 0 }", ".dx-alpha-popup { top: 1px }");
  eq("impact: a closed-popup class takes its source siblings' pages", [r.broad, keys(r)], [[], ["c:alpha", "site:home"]]);
  eq("impact: ... and its component is the owner", [...r.ownerComps.keys()], ["alpha"]);
  r = run(".dx-alpha-kind-wide { top: 0 }", ".dx-alpha-kind-wide { top: 1px }");
  eq("impact: a format!-built class is owned by its literal prefix", [r.broad, keys(r)], [[], ["c:alpha", "site:home"]]);
  r = run(".nobody-spells-me { top: 0 }", ".nobody-spells-me { top: 1px }");
  eq("impact: a dead class reaches no page", [r.broad, keys(r)], [[], []]);
  r = run("", ".dx-alpha:hover { top: 1px }");
  eq("impact: an added rule", keys(r), ["c:alpha", "site:home"]);
  r = run(".hero { color: red }", ".hero { color: red } /* only a comment changed */");
  eq("impact: a comment-only edit changes nothing", [r.broad, keys(r), r.notes.some((n) => /comments/.test(n))], [[], [], true]);

  // fallbacks to the broad set
  const broad = (o, n) => run(o, n).broad.length > 0;
  check("broad: element selector", broad("body { margin: 0 }", "body { margin: 1px }"));
  check("broad: attribute-only selector", broad('dialog[class^="dx-"] { top: 0 }', 'dialog[class^="dx-"] { top: 1px }'));
  check("broad: :root with a real property", broad(":root { color-scheme: light }", ":root { color-scheme: dark }"));
  check("broad: @font-face", broad("@font-face { font-family: a; src: url(a) }", "@font-face { font-family: a; src: url(b) }"));
  check("broad: @import", broad("", '@import "a.css";'));
  check("broad: a nested rule", broad("", ".a { .b { top: 0 } }"));
  check("broad: an unrecognised at-rule", broad("", "@page { margin: 0 }"));

  // custom properties
  write("preview/assets/readers.css", `.hero { gap: var(--tok) } .docs-title { margin: var(--tok2, 0) } :root { --tok2: var(--tok3) } .dx-gamma { padding: var(--tok3) }`);
  ctx.sheets.push({ rel: "preview/assets/readers.css", abs: path.join(tmp, "preview/assets/readers.css"), rules: null });
  r = run(":root { --tok: 1px }", ":root { --tok: 2px }");
  eq("token: traced to the class that reads it", [r.broad, keys(r)], [[], ["site:home"]]);
  r = run(":root { --tok3: 1px }", ":root { --tok3: 2px }");
  eq("token: followed through a variable that re-reads it", [r.broad, keys(r)], [[], ["c:gamma", "site:docs"]]);
  r = run(":root { --a: 1; --tok: 1px; --b: 2 }", ":root { --a: 1; --tok: 2px; --b: 2 }");
  eq("token: only the declaration that changed is traced (in-place edit)", keys(r), ["site:home"]);
  eq("token: the changed name is recorded for the spec-name rule", [...r.mentions.keys()].includes("--tok") && !r.mentions.has("--a"), true);
  r = run(":root { --gamma-gap: 1px }", ":root { --gamma-gap: 2px }");
  check("token: spelled in script or inline style -> broad", r.broad.length === 1 && /gamma/.test(r.broad[0]));
  r = run(":root { --site-inline: 1 }", ":root { --site-inline: 2 }");
  check("token: spelled in main.rs -> broad", r.broad.length === 1);
  r = run(":root { --nobody: 1 }", ":root { --nobody: 2 }");
  eq("token: read by nobody -> no page", [r.broad, keys(r)], [[], []]);
  write("preview/assets/anim.css", `.dx-beta { animation: dx-spin 1s } .docs-title { --enter: dx-spin }`);
  ctx.sheets.push({ rel: "preview/assets/anim.css", abs: path.join(tmp, "preview/assets/anim.css"), rules: null });
  r = run("@keyframes dx-spin { to { opacity: 0 } }", "@keyframes dx-spin { to { opacity: 1 } }");
  eq("keyframes: traced to the rules that name it", [r.broad, keys(r)], [[], ["c:alpha", "c:beta", "site:docs"]]);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

let LISTED = null; // spec file -> every test title path, from `playwright test --list` (section 4)
const QUICK = process.env.SELECT_SELFTEST_QUICK === "1"; // sections 1-3 only (the mutation proof of the CSS half)
if (!QUICK) {
// ---- 4. title shapes against the real specs ---------------------------------------------------------------------
const comps = fs.readdirSync(path.join(ROOT, "preview/src/components"), { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name);
const SCOPES = makeTitleScopes(comps);
eq("titles: first-paint client navigation names both pages", SCOPES["oracle/tier2-html/first-paint-styles.spec.ts"]("3. client-side navigation: no route is painted with late styles /docs -> /component/bar_chart/"), ["site:docs", "c:bar_chart"]);
eq("titles: layout-stability", SCOPES["layout-stability.spec.ts"]("layout stability /component/avatar/ dark (5 widths)"), ["c:avatar"]);
eq("titles: layout-stability home", SCOPES["layout-stability.spec.ts"]("layout stability / light (64 widths)"), ["site:home"]);
eq("titles: theme-picker-phone", SCOPES["theme-picker-phone.spec.ts"]("phone 390x844 demos (/demos): unchanged, the entry is the navigation sheet's \"Theme\""), ["site:demos"]);
eq("titles: touch", SCOPES["oracle/tier2-html/touch-double-tap.spec.ts"]("component \"radio_group\": every interactive element suppresses double-tap zoom"), ["c:radio_group"]);
eq("titles: popup-style-isolation", SCOPES["popup-style-isolation.spec.ts"]("popup style isolation: a popup nested in a component looks like the same popup alone dropdown_menu"), ["c:dropdown_menu"]);
eq("titles: listener-inventory route hop is unscoped", SCOPES["oracle/tier2-html/listener-inventory.spec.ts"]("listener inventory: x route: entering and leaving the sidebar provider (window resize + keydown) leaves no listener behind"), null);
eq("titles: alias matches a squashed name", SCOPES["oracle/tier1-apg/keyboard-matrix.spec.ts"]("APG Menu Button pattern — DropdownMenu trigger — Enter")?.includes("c:dropdown_menu"), true);

{
  const env = { ...process.env, PLAYWRIGHT_BASE_URL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:1" };
  const files = Object.keys(SCOPES);
  const missing = files.filter((f) => !fs.existsSync(path.join(PW, f)));
  eq("titles: every scoped spec exists", missing, []);
  // `playwright test --list` LOADS each file; a spec that builds something while it loads (hydration-parity runs
  // `cargo build` for attr-synth at load time) must never be listed here: the build lock is for cargo
  const loadsCargo = files.filter((f) => fs.existsSync(path.join(PW, f)) && /(?:spawnSync|execSync|execFileSync|spawn)\(\s*["'`]cargo|cargo build/.test(fs.readFileSync(path.join(PW, f), "utf8")));
  eq("titles: no scoped spec runs cargo while it loads", loadsCargo, []);
  // Row 338, as a class: NO spec file may run cargo at all (module load runs for `--list`, outside the build lock).
  // A generator binary is built by a tools/ pre-step through scripts/lane-lock.sh and only RUN by the spec.
  const allSpecs = [];
  const walkDir = (d) => { for (const e of fs.readdirSync(d, { withFileTypes: true })) { if (e.name === "node_modules" || e.name === "target") continue; const q = path.join(d, e.name); if (e.isDirectory()) walkDir(q); else if (/\.spec\.ts$/.test(e.name)) allSpecs.push(q); } };
  walkDir(PW);
  eq("specs: none runs cargo (row 338)", allSpecs.filter((f) => /(?:spawnSync|execSync|execFileSync|spawn|exec)\(\s*["'`]cargo|\bcargo build\b/.test(fs.readFileSync(f, "utf8"))).map((f) => path.relative(PW, f)), []);
  const r = spawnSync("pnpm", ["exec", "playwright", "test", "-c", "ssg.local.config.ts", "--list", "--reporter=json", ...files], { cwd: PW, env, encoding: "utf8", maxBuffer: 1 << 28 });
  if (r.status !== 0 || !r.stdout) check("titles: playwright --list works", false, (r.stderr || "").slice(0, 300));
  else {
    const j = JSON.parse(r.stdout), perFile = new Map();
    const walk = (s, trail, file) => {
      for (const sp of s.specs ?? []) { const f = sp.file ?? file; if (!perFile.has(f)) perFile.set(f, []); perFile.get(f).push([...trail, sp.title].join(" ")); }
      for (const c of s.suites ?? []) walk(c, c.file && c.title === c.file ? trail : [...trail, c.title], c.file ?? file);
    };
    for (const t of j.suites ?? []) walk(t, [], t.file);
    LISTED = perFile;
    for (const f of files) {
      const titles = perFile.get(f) ?? [];
      const scoped = titles.filter((t) => SCOPES[f](t));
      if (f === "all-routes.spec.ts") {
        // The page-naming shape is read off the spec's own listed titles, never a phrase typed here: the first test whose title
        // starts "<component>: " / "site-<page>: " must scope to exactly that page, and a test that names no route stays unscoped.
        const compTitle = titles.find((t) => comps.some((c) => t.startsWith(`${c}: `)));
        const siteTitle = titles.find((t) => /^site-(home|docs|demos|charts): /.test(t));
        const pageless = titles.find((t) => !/^[\w-]+: /.test(t));
        check("titles: all-routes has a listed component-route test", !!compTitle);
        if (compTitle) eq("titles: all-routes component", SCOPES[f](compTitle), [`c:${compTitle.split(": ")[0]}`]);
        if (siteTitle) eq("titles: all-routes site page", SCOPES[f](siteTitle), [`site:${siteTitle.match(/^site-(\w+)/)[1]}`]);
        if (pageless) eq("titles: all-routes page-less test is unscoped", SCOPES[f](pageless), null);
      }
      // a table entry that names no page at all in its own spec is stale: the spec's titles changed shape
      check(`titles: ${f} still names pages in its titles (${scoped.length}/${titles.length})`, scoped.length > 0, "the title pattern matches no test of this spec");
      // a title that visibly names a page ("<component>: ...", "/component/<name>/", "component \"<name>\"") must be recognised: if a
      // spec renames its tests the table has to follow, and this is the line that goes red instead of the tests silently staying
      const names = titles.filter((t) => comps.some((c) => t.startsWith(`${c}: `) || t.includes(`/component/${c}/`) || t.includes(`component "${c}"`)));
      const unseen = names.filter((t) => !SCOPES[f](t));
      check(`titles: ${f} recognises every title that names a component page (${names.length - unseen.length}/${names.length})`, unseen.length === 0, unseen.slice(0, 2).map((t) => t.slice(0, 100)).join(" | "));
      // every key an entry returns is a real page key
      for (const t of scoped) for (const k of SCOPES[f](t)) if (!/^(c:[\w]+|site:(home|docs|demos|charts|dashboard))$/.test(k) || (k.startsWith("c:") && !comps.includes(k.slice(2)))) check(`titles: ${f} names a real page`, false, `${JSON.stringify(t)} -> ${k}`);
    }
  }
}

// ---- 5 + 6. the real tree: --pages, the drop pattern, --no-css-narrow -------------------------------------------
{
  const node = process.execPath;
  const env = { ...process.env };
  for (const k of ["PLAYWRIGHT_BASE_URL", "SELECT_SSG_DIR", "SSG_SITE_DIR"]) delete env[k]; // the "no build to read" case must not find one
  const run = (...a) => JSON.parse(execFileSync(node, [path.join(HERE, "select.mjs"), "--format", "json", ...a], { cwd: ROOT, env, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], maxBuffer: 1 << 28 }));
  // `--files` with no names is an empty diff: the checks below must not read this branch's own git diff (a batch branch that
  // changes components adds their at-rest all-routes tests to the kept set, row 152), only the pages they pretend.
  const narrowed = run("--files", "--pages", "c:calendar");
  const all = narrowed.narrowed.find((n) => n.file === "all-routes.spec.ts");
  check("pages: all-routes is narrowed to the calendar page", !!all && all.kept < 30 && all.kept >= 2, JSON.stringify(all && [all.kept, all.total]));
  check("pages: a drop pattern is produced", typeof narrowed.grepInvert === "string" && narrowed.grepInvert.length > 0);
  if (all && narrowed.grepInvert) {
    const re = new RegExp(narrowed.grepInvert, "i"); // Playwright builds the same case-insensitive RegExp from --grep-invert
    const grepText = (file, title) => `chromium ${file} ${title}`;
    // By construction, from the specs' own current titles: the pattern must match EXACTLY the titles select.mjs says it dropped,
    // in every narrowed file, and nothing in any other file (a hard-coded sample title goes stale when a spec renames its tests).
    check("pattern: the real titles of the specs are listed", !!LISTED && LISTED.has("all-routes.spec.ts"));
    if (LISTED) {
      let wrong = 0, seen = 0;
      const detail = [];
      for (const n of narrowed.narrowed) {
        const titles = LISTED.get(n.file);
        if (!titles) continue; // not listed here (a spec that runs cargo at load is kept whole and never narrowed)
        for (const t of titles) {
          seen++;
          // the expectation is computed HERE from the title table, not read back from select.mjs: a test that names pages is
          // dropped iff none of them is reached (--pages c:calendar), a test that names none is always kept
          const ks = SCOPES[n.file]?.(t);
          const want = !!ks && ks.length > 0 && !ks.includes("c:calendar");
          if (re.test(grepText(n.file, t)) !== want) { wrong++; if (detail.length < 3) detail.push(`${n.file}: ${t.slice(0, 90)} (want ${want ? "dropped" : "kept"})`); }
        }
        check(`pattern: ${n.file} keeps ${n.kept} of ${n.total}`, titles.length === n.total && titles.filter((t) => !re.test(grepText(n.file, t))).length === n.kept, `listed ${titles.length}`);
      }
      check(`pattern: drops exactly the dropped tests (${seen} titles checked)`, wrong === 0 && seen > 0, detail.join("; "));
      for (const [f, titles] of LISTED) if (!narrowed.narrowed.some((n) => n.file === f)) check(`pattern: leaves ${f} alone`, titles.every((t) => !re.test(grepText(f, t))));
    }
    check("pattern: does not touch another spec file", !re.test(grepText("button.spec.ts", "calendar: axe (light)")) && !re.test(grepText("button.spec.ts", "accordion: axe (light)")));
    const sample = LISTED?.get("all-routes.spec.ts")?.find((t) => t.startsWith("accordion: "));
    if (sample) check("pattern: anchored, so a longer title is not dropped", !re.test(grepText("all-routes.spec.ts", sample + " extra")));
  }
  const noCss = run("--files", "preview/assets/main.css", "--no-css-narrow");
  check("--no-css-narrow: the whole BROAD set (matrix specs and the guards) comes back", noCss.specs.length >= 25 && noCss.specs.includes("all-routes.spec.ts") && !noCss.grepInvert);
  const emptySite = fs.mkdtempSync(path.join(os.tmpdir(), "select-empty-site-"));
  fs.writeFileSync(path.join(emptySite, "index.html"), "<html><body></body></html>");
  const devLike = run("--files", "preview/assets/main.css", "--range", "HEAD~5..HEAD", "--ssg-dir", emptySite);
  fs.rmSync(emptySite, { recursive: true, force: true });
  check("a build that is not a prerendered SSG (an empty shell, as a dev server serves) is not believed", devLike.specs.length >= 25 && devLike.specs.includes("all-routes.spec.ts") && !devLike.grepInvert);
  const noIndex = run("--files", "preview/assets/main.css", "--range", "HEAD~5..HEAD");
  check("no SSG build to read: the BROAD set stays", noIndex.specs.length >= 25 && noIndex.specs.includes("all-routes.spec.ts") && !noIndex.grepInvert);
  // A file inside a primitive directory maps to the components that reach what mod.rs re-exports from it:
  // virtual/find.rs is VirtualList's and VirtualGrid's find-in-page, so it is their specs, not Carousel's and not the
  // broad set; virtual/mod.rs keeps every user of the engine.
  const find = run("--files", "primitives/src/virtual/find.rs");
  check("primitive submodule: virtual/find.rs selects the virtual list and grid specs", ["virtual_list.spec.ts", "virtual-grid.spec.ts", "virtual-list-find.spec.ts"].every((f) => find.specs.includes(f)), find.specs.join(" "));
  check("primitive submodule: virtual/find.rs selects neither Carousel nor the broad set", !find.specs.some((f) => /^carousel|^message_scroller|dx-class-migration|hydration-parity|preview\.spec/.test(f)), find.specs.join(" "));
  const vmod = run("--files", "primitives/src/virtual/mod.rs");
  check("primitive submodule: virtual/mod.rs keeps every engine user (Carousel, Message Scroller, the tables)", ["carousel.spec.ts", "message_scroller.spec.ts", "table.spec.ts", "virtual_list.spec.ts"].every((f) => vmod.specs.includes(f)) && !vmod.specs.includes("dx-class-migration.spec.ts"), vmod.specs.join(" "));
  // A component's at-rest axe scan lives only in all-routes (row 152): a change to the component runs that page's test.
  const comp = run("--files", "preview/src/components/kbd/component.rs");
  const ar = comp.narrowed.find((n) => n.file === "all-routes.spec.ts");
  check("component change: all-routes runs, narrowed to the component's page", comp.specs.includes("all-routes.spec.ts") && !!ar && ar.kept >= 2 && ar.kept < 10, JSON.stringify(ar && [ar.kept, ar.total]));
  const arTitles = LISTED?.get("all-routes.spec.ts");
  if (arTitles && comp.grepInvert) {
    const re = new RegExp(comp.grepInvert, "i");
    const kept = arTitles.filter((t) => !re.test(`chromium all-routes.spec.ts ${t}`));
    check("component change: the kept all-routes tests are the component's page and the page-less ones", kept.some((t) => t.startsWith("kbd: ")) && !kept.some((t) => /^(accordion|button|site-home): /.test(t)), kept.join(" | ").slice(0, 300));
  }
}

}

console.log(failed ? `select-selftest: FAIL -- ${failed} of ${passed + failed} checks` : `select-selftest: OK -- ${passed} checks`);
process.exit(failed ? 1 : 0);
