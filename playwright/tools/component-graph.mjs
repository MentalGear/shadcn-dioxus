// The component use graph of select.mjs (backlog row 510), derived from the sources on every run. Nothing here is a
// hand-kept list: a component added later, or a new use of one component by another, is picked up as it is written.
//
// Three kinds of edge `A -> B` ("A depends on B", so a change to B's code reaches A's rendering), each read from text:
//   code  A's own files (everything in the directory of A under preview/src/components/, outside variants/) name B as `crate::components::B`,
//         `crate::components::{B::.., ..}` or, from A/component.rs, `super::super::B`; or A's component.json lists B in
//         `componentDependencies` (the installer's closure; scripts/check-installed-paths.sh keeps it equal to the imports
//         of component.rs). A's page, A's specs and every consumer of A render B's code.
//   demo  A's demos (A/variants/**) name B. Only A's own page embeds B: a consumer of A does not render A's demos.
//   css   A/style.css spells a class of B (`.dx-B`, the longest component name that prefixes the class). A's rules style B's
//         markup, so a change to B can break them; as with demos, that is seen on A's own page.
//   site  a file of the app outside the components (preview/src/site/, dashboard/, docs_nav.rs, a *_parity.rs, ...) names B:
//         a page of the site renders B (siteUsers). The graph only reports which files; select.mjs maps them to specs.
//
// `consumersOf(B)` returns every component whose rendering contains B's changed code: B itself, the transitive code
// consumers, and the demo/css consumers of any of those (one hop: a demo does not embed a demo).
import fs from "node:fs";
import path from "node:path";

export const COMPONENTS_DIR = "preview/src/components";
// Files of a component directory that do not change what any page renders: its demos, its docs, its manifest.
export const NO_FANOUT = /^(variants\/|docs\.md$|notes\.md$|component\.json$)/;
export const reachesConsumers = (relInsideComponent) => !NO_FANOUT.test(relInsideComponent);

const walk = (dir, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, out); else out.push(p);
  }
  return out;
};
const stripComments = (t) => t.replace(/^\s*\/\/.*$/gm, "");

// every top-level name of a `{a::X, b, c::{d, e}}` group
function groupNames(body) {
  const out = [];
  let depth = 0, cur = "";
  const flush = () => { const m = cur.trim().match(/^([a-z_0-9]+)/); if (m) out.push(m[1]); cur = ""; };
  for (const ch of body) {
    if (ch === "{") depth++;
    if (ch === "}") depth--;
    if (depth === 0 && ch === ",") flush(); else if (depth === 0 || ch !== ",") cur += ch;
  }
  flush();
  return out;
}

// the spellings of one component naming another in Rust. `fromComponentFile`: a file directly in the component's own
// directory, where `super::super` is the `components` module itself.
export function rustRefs(text, { fromComponentFile }) {
  const names = new Set();
  for (const m of text.matchAll(/crate::components::([a-z_0-9]+)\b/g)) names.add(m[1]);
  for (const m of text.matchAll(/crate::components::\{/g)) {
    let i = m.index + m[0].length, depth = 1;
    const start = i;
    while (depth && i < text.length) { if (text[i] === "{") depth++; else if (text[i] === "}") depth--; i++; }
    for (const n of groupNames(text.slice(start, i - 1))) names.add(n);
  }
  if (fromComponentFile) for (const m of text.matchAll(/\bsuper::super::([a-z_0-9]+)\b/g)) names.add(m[1]);
  return names;
}

// A form of `crate::components` that names no component directly (`components::*`, `use crate::components;`, an alias):
// a use the scan above cannot attribute. select-selftest.mjs fails when one exists.
export function unreadableUses(text) {
  const out = [];
  for (const m of text.matchAll(/crate::components(?!::(?:[a-z_0-9]+\b|\{))/g)) out.push(text.slice(m.index, m.index + 40).split("\n")[0]);
  for (const m of text.matchAll(/crate::components::\{[^}]*\*[^}]*\}|\buse\s+crate::components\s+as\b/g)) out.push(m[0].slice(0, 40));
  return out;
}

// `.dx-button-group-x` is button_group's, `.dx-button-icon` is button's: the longest component name that prefixes the class
function cssRefs(text, comps) {
  const dashed = comps.map((c) => [c, c.replace(/_/g, "-")]).sort((a, b) => b[1].length - a[1].length);
  const names = new Set();
  for (const m of text.matchAll(/\.dx-([a-z0-9-]+)/g)) {
    const hit = dashed.find(([, d]) => m[1] === d || m[1].startsWith(d + "-"));
    if (hit) names.add(hit[0]);
  }
  return names;
}

// Files of preview/src outside components/ that name a component, as `components::<B>` or in a `components::{..}` group,
// per component. A module root that only imports (`site/mod.rs`; its children say `use super::*`) renders what its children
// spell: the users of a component imported there are the children that mention one of the types imported from it.
function siteUses(root, comps) {
  const src = path.join(root, "preview/src"), out = new Map(comps.map((c) => [c, new Set()]));
  const textOf = (f) => stripComments(fs.readFileSync(f, "utf8")).split(/^#\[cfg\(test\)\]/m)[0]; // a test module closes the file: fixtures, not renderings
  for (const f of walk(src)) {
    const rel = path.relative(root, f);
    if (!f.endsWith(".rs") || rel.startsWith(COMPONENTS_DIR + "/")) continue;
    const text = textOf(f);
    const names = new Set([...text.matchAll(/\bcomponents::([a-z_0-9]+)\b/g)].map((m) => m[1]));
    for (const m of text.matchAll(/\bcomponents::\{/g)) {
      let i = m.index + m[0].length, depth = 1;
      const start = i;
      while (depth && i < text.length) { if (text[i] === "{") depth++; else if (text[i] === "}") depth--; i++; }
      for (const n of groupNames(text.slice(start, i - 1))) names.add(n);
    }
    const kids = path.basename(f) === "mod.rs"
      ? fs.readdirSync(path.dirname(f)).filter((k) => k.endsWith(".rs") && k !== "mod.rs").map((k) => [path.join(path.dirname(rel), k), textOf(path.join(path.dirname(f), k))]).filter(([, t]) => /\buse super::\*/.test(t))
      : [];
    for (const n of names) {
      if (!out.has(n)) continue;
      const idents = [...text.matchAll(new RegExp(`\\b${n}::(\\{[^}]*\\}|\\w+)`, "g"))].flatMap((m) => m[1].match(/\b[A-Z]\w*/g) ?? []);
      const using = idents.length ? kids.filter(([, t]) => idents.some((i) => new RegExp(`\\b${i}\\b`).test(t))).map(([k]) => k) : [];
      for (const u of using.length ? using : [rel]) out.get(n).add(u);
    }
  }
  return new Map([...out].map(([c, set]) => [c, [...set].sort()]));
}

export function buildComponentGraph({ root, comps }) {
  const dir = path.join(root, COMPONENTS_DIR);
  const code = new Map(), demo = new Map(), css = new Map(), manifest = new Map(), rustCode = new Map(), unreadable = [];
  for (const c of comps) {
    const base = path.join(dir, c);
    let codeText = "", demoText = "";
    for (const f of walk(base)) {
      if (!f.endsWith(".rs")) continue;
      const rel = path.relative(base, f);
      if (rel.startsWith("variants/")) demoText += stripComments(fs.readFileSync(f, "utf8")) + "\n";
      else codeText += stripComments(fs.readFileSync(f, "utf8")) + "\n";
    }
    const own = (set) => new Set([...set].filter((x) => comps.includes(x) && x !== c));
    const rust = own(rustRefs(codeText, { fromComponentFile: true }));
    let man = new Set();
    try {
      const j = JSON.parse(fs.readFileSync(path.join(base, "component.json"), "utf8"));
      man = own((j.componentDependencies ?? []).map((d) => (typeof d === "string" ? d : d.name)));
    } catch { /* a component without a manifest has no declared dependency */ }
    for (const u of [...unreadableUses(codeText), ...unreadableUses(demoText)]) unreadable.push(`${c}: ${u}`);
    rustCode.set(c, rust);
    manifest.set(c, man);
    code.set(c, new Set([...rust, ...man]));
    demo.set(c, own(rustRefs(demoText, { fromComponentFile: false })));
    const cssFile = path.join(base, "style.css");
    css.set(c, fs.existsSync(cssFile) ? own(cssRefs(fs.readFileSync(cssFile, "utf8"), comps)) : new Set());
  }
  const codeUsers = new Map(comps.map((c) => [c, []])), pageUsers = new Map(comps.map((c) => [c, []]));
  for (const c of comps) {
    for (const d of code.get(c)) codeUsers.get(d).push(c);
    for (const d of demo.get(c)) pageUsers.get(d).push([c, "demo"]);
    for (const d of css.get(c)) pageUsers.get(d).push([c, "css"]);
  }

  // every component whose rendering contains `changed`'s code -> a chain explaining why, e.g. "card -> data_table (code)"
  function consumersOf(changed) {
    const why = new Map([[changed, changed]]), queue = [changed];
    while (queue.length) {
      const x = queue.shift();
      for (const u of codeUsers.get(x).sort()) if (!why.has(u)) { why.set(u, `${why.get(x)} <- ${u} (code)`); queue.push(u); }
    }
    const codeClosure = new Set(why.keys());
    for (const x of [...why.keys()]) {
      for (const [u, kind] of pageUsers.get(x).sort()) if (!why.has(u)) why.set(u, `${why.get(x)} <- ${u} (${kind})`);
    }
    why.delete(changed);
    return Object.assign(why, { codeClosure });
  }
  const siteUsers = siteUses(root, comps);
  // what a changed file `relInside` (relative to the component's directory) reaches: the consumers, and the site files that
  // render the component or one of them. A demo, a doc or a manifest reaches nothing beyond its own component's page.
  function reachOf(comp, relInside) {
    if (!reachesConsumers(relInside)) return { consumers: new Map(), sites: [] };
    const consumers = consumersOf(comp), sites = [];
    // a site file renders the changed code only through the component itself and its code consumers: a component whose demo
    // or stylesheet names it shows it on its own page, and a site file that renders that component does not show its demos
    for (const c of consumers.codeClosure) for (const file of siteUsers.get(c)) sites.push({ comp: c, file });
    return { consumers, sites };
  }
  return { code, demo, css, manifest, rustCode, unreadable, siteUsers, consumersOf, reachOf };
}
