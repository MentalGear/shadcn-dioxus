// What a CSS diff can change, as a set of pages (backlog row 323).
//
//   const ctx = buildContext({ root, index, comps });
//   const r = impactOfChanges(ctx, [{ file, oldText, newText }]);
//   r.broad    string[]               reasons the diff cannot be narrowed (empty = narrowed)
//   r.keys     Map<pageKey, string[]> pages (`c:<component>` or `site:<page>`) whose DOM can contain an element the
//                                     changed rules match, with the rule that reaches them
//   r.mentions Map<name, string[]>    changed class / custom-property / keyframes names (specs that spell one of them
//                                     are selected by the caller whatever the pages say)
//   r.notes    string[]               human-readable trace for --explain
//
// The argument, in full (a rule is the unit, never a file):
//   1. Only elements matching a changed rule's selector can render differently, and the elements a selector can match
//      are among those carrying the classes / ids of its constraining compound (`.a button` only styles buttons inside
//      `.a`, so it needs `.a`). Inheritance and custom properties flow DOWN to descendants, which live on the same
//      pages as their ancestor. So the pages are the intersection of the pages of the required classes.
//   2. A page's classes are what the SSG rendered for it (site-index.mjs). A class the SSG does not show but a source
//      file spells (a closed popup, script-added) takes the pages of its source siblings: the classes of the same
//      component directory / site file / primitive that ARE in the HTML. A class nobody spells is dead (no pages).
//   3. Anything this argument does not cover falls back to the broad set: a selector with no class or id and a
//      non-variable declaration (`html`, `body`, `*`, `dialog[class^="dx-"]`), `@font-face`, `@import`, `@layer` order,
//      an unparseable or nested rule, a class that cannot be placed, and a custom property or keyframes name that is
//      read by script or inline style. A custom property defined by an unconstrained rule (`:root { --x }`) is traced
//      to the rules that read it (`var(--x`), transitively; each reader is resolved by 1 and 2.
import fs from "node:fs";
import path from "node:path";
import { parseCss, analyzeSelector, diffRules } from "./css-rules.mjs";

const SKIP_DIRS = new Set(["node_modules", "target", "dist", ".git", "public"]);
const walk = (dir, out = []) => {
  if (!fs.existsSync(dir)) return out;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP_DIRS.has(e.name)) continue;
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, out); else out.push(p);
  }
  return out;
};
const esc = (s) => s.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&");
const wordRe = (name) => new RegExp(`(?<![\\w-])${esc(name)}(?![\\w-])`);

export function routeKey(route, comps) {
  const r = route.endsWith("/") ? route : route + "/";
  if (r === "/") return "site:home";
  if (r === "/docs/") return "site:docs";
  if (r === "/demos/") return "site:demos";
  if (r.startsWith("/charts/")) return "site:charts";
  if (r.startsWith("/dashboard/")) return "site:dashboard";
  if (r === "/component/" || r === "/component/block/") return "site:component-index";
  let m = r.match(/^\/component\/block\/([^/]+)\//);
  if (m && comps.includes(m[1])) return `c:${m[1]}`;
  if ((m = r.match(/^\/component\/([^/]+)\//)) && comps.includes(m[1])) return `c:${m[1]}`;
  return "site:other";
}

export function buildContext({ root, index, comps }) {
  const code = walk(path.join(root, "preview/src")).concat(walk(path.join(root, "primitives/src")), [path.join(root, "preview/index.html")])
    .filter((f) => /\.(rs|js|ts|html)$/.test(f) && fs.existsSync(f))
    .map((f) => {
      const rel = path.relative(root, f).split(path.sep).join("/");
      const text = fs.readFileSync(f, "utf8");
      let m, unit = rel;
      if ((m = rel.match(/^preview\/src\/components\/([^/]+)\//))) unit = `comp:${m[1]}`;
      else if ((m = rel.match(/^primitives\/src\/(?:ts\/|js\/)?([^/.]+)/))) unit = `prim:${m[1]}`;
      // `format!("dx-calendar-{kind}")` / `` `dx-${x}` ``: the literal prefix owns every class that starts with it
      const prefixes = new Set([...text.matchAll(/([A-Za-z][\w-]*-)\$?\{/g)].map((x) => x[1]).filter((p) => p.length >= 3));
      return { rel, text, unit, prefixes };
    });
  const sheets = walk(path.join(root, "preview/assets")).concat(walk(path.join(root, "preview/src")), walk(path.join(root, "primitives/src")))
    .filter((f) => f.endsWith(".css"))
    .map((f) => ({ rel: path.relative(root, f).split(path.sep).join("/"), abs: f, rules: null }));
  const ctx = { root, index, comps, code, sheets, memo: new Map(), unitClasses: new Map() };
  return ctx;
}

const sheetRules = (s) => (s.rules ??= parseCss(fs.readFileSync(s.abs, "utf8")));

function owners(ctx, name) {
  const re = wordRe(name);
  return ctx.code.filter((f) => re.test(f.text) || [...f.prefixes].some((p) => name.startsWith(p)));
}

function siblingPages(ctx, unit) {
  let cls = ctx.unitClasses.get(unit);
  if (!cls) {
    cls = new Set();
    for (const f of ctx.code) if (f.unit === unit) for (const w of f.text.match(/[A-Za-z_][\w-]*/g) ?? []) if (ctx.index.classes.has(w)) cls.add(w);
    ctx.unitClasses.set(unit, cls);
  }
  const pages = new Set();
  for (const c of cls) for (const r of ctx.index.classes.get(c)) pages.add(r);
  return pages;
}

// Set<route> (maybe empty: dead) or null (cannot be placed)
export function pagesOfTerm(ctx, term) {
  const map = term.kind === "id" ? ctx.index.ids : ctx.index.classes;
  const hit = map.get(term.name);
  if (hit) return { pages: hit, how: "html" };
  const own = owners(ctx, term.name);
  if (own.length === 0) return { pages: new Set(), how: "dead" };
  const pages = new Set();
  for (const u of new Set(own.map((f) => f.unit))) {
    const p = siblingPages(ctx, u);
    if (p.size === 0) return { pages: null, how: `spelled in ${u} but none of its classes is in the SSG HTML` };
    for (const r of p) pages.add(r);
  }
  return { pages, how: `siblings of ${[...new Set(own.map((f) => f.unit))].join(", ")}` };
}

// One comma alternative -> { pages:Set<route>, names:[...] } or { broad: reason }
function resolveAlt(ctx, alt, names) {
  if (alt.unconstrained) return { broad: `selector "${alt.sel ?? ""}" names no class or id` };
  const sets = [];
  for (const t of alt.terms) {
    names.set(t.name, t.kind);
    const r = pagesOfTerm(ctx, t);
    if (r.pages) sets.push(r.pages);
  }
  for (const g of alt.groups) {
    const u = new Set();
    let ok = true;
    for (const a of g) {
      const r = resolveAlt(ctx, a, names);
      if (r.broad) { ok = false; break; }
      for (const p of r.pages) u.add(p);
    }
    if (ok) sets.push(u);
  }
  if (sets.length === 0) return { broad: `class ${alt.terms.map((t) => "." + t.name).join(" ") || alt.sel} cannot be placed on any page` };
  let acc = sets[0];
  for (const s of sets.slice(1)) acc = new Set([...acc].filter((x) => s.has(x)));
  return { pages: acc };
}

const isCustom = (p) => p.startsWith("--");

// property names whose value list differs between two declaration lists (a property declared twice is a fallback chain)
function changedProps(a, b) {
  const group = (ds) => { const m = new Map(); for (const [p, v] of ds) { if (!m.has(p)) m.set(p, []); m.get(p).push(v); } return m; };
  const A = group(a), B = group(b), out = new Set();
  for (const p of new Set([...A.keys(), ...B.keys()])) if (JSON.stringify(A.get(p) ?? null) !== JSON.stringify(B.get(p) ?? null)) out.add(p);
  // the same declarations in a different order can change the winner of a longhand/shorthand pair
  if (out.size === 0) for (const [p] of a) out.add(p);
  return out;
}

// pages (Set<route>) of every reader of a custom property / keyframes name, or { broad }
function traceName(ctx, name, kind, seen, names) {
  if (seen.has(name)) return { pages: new Set() };
  seen.add(name);
  names.set(name, kind);
  const re = kind === "keyframes" ? wordRe(name) : new RegExp(`var\\(\\s*${esc(name)}(?![\\w-])`);
  const direct = kind === "keyframes" ? wordRe(name) : wordRe(name);
  const hit = ctx.code.find((f) => direct.test(f.text));
  if (hit) return { broad: `${name} is spelled in ${hit.rel} (script or inline style): not traceable` };
  const pages = new Set();
  for (const s of ctx.sheets) {
    for (const r of sheetRules(s)) {
      if (r.kind !== "rule") continue;
      const reads = r.decls.some(([p, v]) => (kind === "keyframes" ? re.test(v) : re.test(v)) && !(kind === "var" && p === name));
      if (!reads) continue;
      const customOnly = r.decls.length > 0 && r.decls.every(([p]) => isCustom(p));
      const readsDefinedHere = r.decls.filter(([p, v]) => re.test(v) && isCustom(p)).map(([p]) => p);
      for (const a of analyzeSelector(r.sel)) {
        const res = resolveAlt(ctx, a, names);
        if (res.broad) {
          // an unconstrained reader that only re-defines variables is followed through its variables
          if (a.unconstrained && customOnly) continue;
          return { broad: `${name} is read by ${r.sel} in ${s.rel}: ${res.broad}` };
        }
        for (const p of res.pages) pages.add(p);
      }
      for (const v of readsDefinedHere) {
        const sub = traceName(ctx, v, "var", seen, names);
        if (sub.broad) return sub;
        for (const p of sub.pages) pages.add(p);
      }
    }
  }
  return { pages };
}

export function impactOfChanges(ctx, changes) {
  const broad = [], notes = [], keys = new Map(), mentions = new Map();
  const touch = (route, why) => {
    const k = routeKey(route, ctx.comps);
    if (!keys.has(k)) keys.set(k, new Set());
    keys.get(k).add(why);
  };
  const kinds = new Map();
  const addMention = (name, file, kind) => { if (!mentions.has(name)) mentions.set(name, new Set()); mentions.get(name).add(file); kinds.set(name, kind); };
  const seenTokens = new Set();
  for (const { file, oldText, newText } of changes) {
    const d = diffRules(oldText ?? "", newText ?? "");
    if (d.same) { notes.push(`${file}: no rule changed (comments / whitespace only)`); continue; }
    // A rule edited in place (same context and selector, different declarations) is ONE change that touches only the
    // declarations that differ: adding a token to a 50-line `:root` block must not re-trace the other 49.
    const headOf = (r) => `${r.ctx.join(" | ")} :: ${r.kind} ${r.kind === "rule" ? r.sel : r.prelude}`;
    const pool = new Map();
    for (const r of d.removed) { const h = headOf(r); if (!pool.has(h)) pool.set(h, []); pool.get(h).push(r); }
    const rules = [];
    for (const r of d.added) {
      const m = r.kind === "rule" && !r.nested ? pool.get(headOf(r))?.shift() : null;
      if (m && !m.nested) rules.push({ r, side: "~", changed: changedProps(m.decls, r.decls) });
      else rules.push({ r, side: "+" });
    }
    for (const list of pool.values()) for (const r of list) rules.push({ r, side: "-" });
    notes.push(`${file}: ${d.removed.length} rule(s) removed, ${d.added.length} added`);
    for (const { r, side, changed } of rules) {
      const label = `${side} ${r.kind === "rule" ? r.sel : r.prelude}`.slice(0, 110);
      const names = new Map();
      if (r.kind === "stmt" || r.kind === "junk") { broad.push(`${file}: statement ${r.prelude.slice(0, 60)}`); continue; }
      if (r.kind === "at") {
        const m = r.prelude.match(/^@(?:-webkit-)?keyframes\s+([\w-]+)/i);
        const p = r.prelude.match(/^@property\s+(--[\w-]+)/i);
        if (m) {
          const t = traceName(ctx, m[1], "keyframes", seenTokens, names);
          names.forEach((_k, n) => addMention(n, file, names.get(n)));
          if (t.broad) { broad.push(`${file}: ${t.broad}`); continue; }
          for (const route of t.pages) touch(route, `${file} ${label}`);
          notes.push(`${label}: ${t.pages.size} page(s) via readers`);
        } else if (p) {
          const t = traceName(ctx, p[1], "var", seenTokens, names);
          names.forEach((_k, n) => addMention(n, file, names.get(n)));
          if (t.broad) { broad.push(`${file}: ${t.broad}`); continue; }
          for (const route of t.pages) touch(route, `${file} ${label}`);
          notes.push(`${label}: ${t.pages.size} page(s) via readers`);
        } else broad.push(`${file}: ${r.prelude.slice(0, 60)} (at-rule the selector does not model)`);
        continue;
      }
      if (r.nested) { broad.push(`${file}: nested rule ${r.sel}`); continue; }
      const alts = analyzeSelector(r.sel);
      const props = changed ? [...changed] : r.decls.map(([p]) => p);
      const customOnly = props.length > 0 && props.every((p) => isCustom(p));
      // every custom property this rule changes is a name specs may probe, whether or not it can be traced
      if (alts.some((a) => a.unconstrained)) for (const p of props) if (isCustom(p)) names.set(p, "var");
      let pages = new Set(), bad = null;
      for (const a of alts) {
        if (a.unconstrained && customOnly) {
          // `:root { --x: ... }`: follow each variable that changed to its readers
          for (const p of new Set(props)) {
            const t = traceName(ctx, p, "var", seenTokens, names);
            if (t.broad) { bad = t.broad; break; }
            for (const route of t.pages) pages.add(route);
          }
          if (bad) break;
          continue;
        }
        const res = resolveAlt(ctx, { ...a, sel: a.sel }, names);
        if (res.broad) { bad = res.broad; break; }
        for (const route of res.pages) pages.add(route);
      }
      names.forEach((_k, n) => addMention(n, file, names.get(n)));
      if (bad) { broad.push(`${file}: ${label} -- ${bad}`); continue; }
      for (const route of pages) touch(route, `${file} ${label}`);
      notes.push(`${label}: ${pages.size} page(s)`);
    }
  }
  // who OWNS the changed classes: a component directory (its specs observe the rule even when the theme sheet carries
  // it) or a site file (its pages' specs do)
  const ownerComps = new Map(), siteOwned = new Set();
  for (const [name, kind] of kinds) {
    if (kind !== "class") continue;
    for (const f of owners(ctx, name)) {
      const m = f.unit.match(/^comp:(.+)$/);
      // a component OWNS a class only when its library code (component.rs / mod.rs) spells it: a demo that mentions
      // `dx-component-preview-frame` does not own the docs frame
      if (m && !/^preview\/src\/components\/[^/]+\/(component|mod)\.rs$/.test(f.rel)) continue;
      if (m) { if (!ownerComps.has(m[1])) ownerComps.set(m[1], new Set()); ownerComps.get(m[1]).add(name); }
      else if (!f.unit.startsWith("prim:")) siteOwned.add(name);
    }
  }
  return {
    broad, notes,
    keys: new Map([...keys].map(([k, v]) => [k, [...v]])),
    mentions: new Map([...mentions].map(([k, v]) => [k, [...v]])),
    mentionKinds: kinds,
    ownerComps: new Map([...ownerComps].map(([k, v]) => [k, [...v]])),
    siteOwned: [...siteOwned],
  };
}
