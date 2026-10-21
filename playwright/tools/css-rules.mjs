// A small, dependency-free CSS rule parser and rule-level differ for the spec selector (backlog row 323).
//
//   parseCss(text)            -> [{ kind, ctx, sel, decls, key }]   (flat, in source order, comments dropped)
//   diffRules(oldText, newText) -> { removed: [rule], added: [rule], same: bool }
//   analyzeSelector(sel)      -> [{ terms, groups, unconstrained }]  one entry per comma alternative
//
// A "rule" is one qualified rule (`a, b { x: y }`) with its at-rule context (`@media (...)`, `@layer x`, ...), or one
// opaque at-rule (`@keyframes`, `@font-face`, `@property`, ...), or one statement at-rule (`@import`, `@layer a, b;`).
// Two rules are the same rule when their `key` is equal: the context, the selector list and the declaration list
// after comments and whitespace are normalised. The differ is a sequence diff (common prefix and suffix trimmed, LCS
// on the middle), so a rule that merely MOVED counts as removed + added (moving a rule changes the cascade).
//
// No browser, no build, no network, no CSS library: the repo's sheets use no native nesting, and a nested rule is
// reported as `nested: true` so the caller falls back to the broad set instead of guessing.

const GROUP_AT = new Set(["media", "supports", "layer", "container", "scope", "document", "starting-style"]);

const ws = (s) => s.replace(/\s+/g, " ").trim();

// Strip comments, keeping strings intact.
function stripComments(t) {
  let out = "";
  for (let i = 0; i < t.length; i++) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "*") { const e = t.indexOf("*/", i + 2); i = e < 0 ? t.length : e + 1; out += " "; continue; }
    if (c === '"' || c === "'") {
      let j = i + 1;
      while (j < t.length && t[j] !== c) { if (t[j] === "\\") j++; j++; }
      out += t.slice(i, j + 1); i = j; continue;
    }
    out += c;
  }
  return out;
}

// Index of the first top-level occurrence of any char in `stops` at or after `i` (skips strings, (), []), or -1.
function scanTo(t, i, stops) {
  let paren = 0, brack = 0;
  for (; i < t.length; i++) {
    const c = t[i];
    if (c === '"' || c === "'") { i++; while (i < t.length && t[i] !== c) { if (t[i] === "\\") i++; i++; } continue; }
    if (c === "\\") { i++; continue; }
    if (c === "(") paren++; else if (c === ")") paren = Math.max(0, paren - 1);
    else if (c === "[") brack++; else if (c === "]") brack = Math.max(0, brack - 1);
    else if (paren === 0 && brack === 0 && stops.includes(c)) return i;
  }
  return -1;
}

// Index of the `}` matching the `{` at `open`.
function matchBrace(t, open) {
  let depth = 0;
  for (let i = open; i < t.length; i++) {
    const c = t[i];
    if (c === '"' || c === "'") { i++; while (i < t.length && t[i] !== c) { if (t[i] === "\\") i++; i++; } continue; }
    if (c === "\\") { i++; continue; }
    if (c === "{") depth++; else if (c === "}") { depth--; if (depth === 0) return i; }
  }
  return t.length;
}

export function normSelector(s) {
  return ws(s).replace(/\s*([>+~,])\s*/g, "$1").replace(/,/g, ", ");
}

function parseDecls(body) {
  const out = [];
  let i = 0;
  while (i < body.length) {
    let e = scanTo(body, i, ";");
    if (e < 0) e = body.length;
    const d = body.slice(i, e).trim();
    i = e + 1;
    if (!d) continue;
    const c = d.indexOf(":");
    if (c < 0) { out.push([ws(d), ""]); continue; }
    out.push([ws(d.slice(0, c)).toLowerCase().startsWith("--") ? ws(d.slice(0, c)) : ws(d.slice(0, c)).toLowerCase(), ws(d.slice(c + 1))]);
  }
  return out;
}

export function parseCss(text) {
  const t = stripComments(text);
  const rules = [];
  const walk = (from, to, ctx) => {
    let i = from;
    while (i < to) {
      while (i < to && /\s/.test(t[i])) i++;
      if (i >= to) break;
      const stop = scanTo(t, i, "{;}");
      if (stop < 0 || stop >= to) { const rest = ws(t.slice(i, to)); if (rest) rules.push(mk("junk", ctx, rest, "", [])); break; }
      const prelude = ws(t.slice(i, stop));
      if (t[stop] === "}") { i = stop + 1; continue; } // stray brace
      if (t[stop] === ";") { if (prelude) rules.push(mk("stmt", ctx, prelude, "", [])); i = stop + 1; continue; }
      const close = matchBrace(t, stop);
      const inner = t.slice(stop + 1, close);
      if (prelude.startsWith("@")) {
        const name = (prelude.slice(1).match(/^[\w-]+/) ?? [""])[0].toLowerCase();
        if (GROUP_AT.has(name)) walk(stop + 1, close, [...ctx, prelude]);
        else rules.push(mk("at", ctx, prelude, "", [], inner)); // opaque: keyframes, font-face, property, anything unknown
      } else {
        const nested = scanTo(inner, 0, "{") >= 0;
        const r = mk("rule", ctx, "", normSelector(prelude), nested ? [] : parseDecls(inner), nested ? inner : undefined);
        if (nested) r.nested = true;
        rules.push(r);
      }
      i = close + 1;
    }
  };
  const mk = (kind, ctx, prelude, sel, decls, raw) => {
    const ctxS = ctx.map(ws).join(" | ");
    const body = raw !== undefined ? ws(raw) : decls.map(([p, v]) => `${p}:${v}`).join(";");
    const head = kind === "rule" ? sel : prelude;
    return { kind, ctx: ctx.map(ws), sel: kind === "rule" ? sel : "", prelude, decls, key: `${ctxS} :: ${kind} ${head} { ${body} }` };
  };
  walk(0, t.length, []);
  return rules;
}

// Longest-common-subsequence diff on rule keys. Rules are few thousand; the middle after trimming is small.
export function diffRules(oldText, newText) {
  const a = parseCss(oldText), b = parseCss(newText);
  let lo = 0;
  while (lo < a.length && lo < b.length && a[lo].key === b[lo].key) lo++;
  let ea = a.length, eb = b.length;
  while (ea > lo && eb > lo && a[ea - 1].key === b[eb - 1].key) { ea--; eb--; }
  const A = a.slice(lo, ea), B = b.slice(lo, eb);
  const keep = lcsKeep(A.map((r) => r.key), B.map((r) => r.key));
  const removed = A.filter((_, i) => !keep.a.has(i)), added = B.filter((_, i) => !keep.b.has(i));
  return { removed, added, same: removed.length === 0 && added.length === 0 };
}

function lcsKeep(x, y) {
  const n = x.length, m = y.length, keepA = new Set(), keepB = new Set();
  if (n === 0 || m === 0) return { a: keepA, b: keepB };
  if (n * m > 25e6) return { a: keepA, b: keepB }; // pathological rewrite: everything counts as changed
  const w = m + 1, dp = new Uint16Array((n + 1) * w);
  for (let i = n - 1; i >= 0; i--) for (let j = m - 1; j >= 0; j--)
    dp[i * w + j] = x[i] === y[j] ? dp[(i + 1) * w + j + 1] + 1 : Math.max(dp[(i + 1) * w + j], dp[i * w + j + 1]);
  for (let i = 0, j = 0; i < n && j < m;) {
    if (x[i] === y[j]) { keepA.add(i); keepB.add(j); i++; j++; }
    else if (dp[(i + 1) * w + j] >= dp[i * w + j + 1]) i++; else j++;
  }
  return { a: keepA, b: keepB };
}

// ---- selectors -------------------------------------------------------------------------------------------------
// Split `s` at top-level occurrences of the chars in `seps` (skipping (), [], strings, escapes).
function splitTop(s, seps) {
  const parts = [];
  let last = 0;
  for (let i = 0; i < s.length;) {
    const e = scanTo(s, i, seps);
    if (e < 0) break;
    parts.push(s.slice(last, e)); last = e + 1; i = e + 1;
  }
  parts.push(s.slice(last));
  return parts;
}

// Compounds of one complex selector, left to right (descendant space and > + ~ all just separate compounds).
function compounds(sel) {
  const out = [];
  let cur = "", paren = 0, brack = 0;
  for (let i = 0; i < sel.length; i++) {
    const c = sel[i];
    if (c === "\\") { cur += c + (sel[i + 1] ?? ""); i++; continue; }
    if (c === '"' || c === "'") { let j = i + 1; while (j < sel.length && sel[j] !== c) { if (sel[j] === "\\") j++; j++; } cur += sel.slice(i, j + 1); i = j; continue; }
    if (c === "(") paren++; else if (c === ")") paren--; else if (c === "[") brack++; else if (c === "]") brack--;
    if (paren === 0 && brack === 0 && (/\s/.test(c) || c === ">" || c === "+" || c === "~")) { if (cur) out.push(cur); cur = ""; continue; }
    cur += c;
  }
  if (cur) out.push(cur);
  return out;
}

const IDENT = String.raw`(?:[\w-]|[^\x00-\x7f]|\\.)+`;

// What one compound REQUIRES of an element: classes and ids that must be present, and unions (`:is(.a, .b)`) of
// which one must be. `:not()`, `:has()` and the rest constrain nothing we can rely on and are ignored.
function compoundTerms(c) {
  const terms = [], groups = [];
  let rest = c;
  // functional pseudo-classes first (they may contain classes that are NOT required of the element)
  rest = rest.replace(/:(is|where|matches|-webkit-any|-moz-any)\(((?:[^()]|\((?:[^()]|\([^()]*\))*\))*)\)/g, (_, _n, arg) => {
    const alts = splitTop(arg, ",").map((x) => x.trim()).filter(Boolean).map((x) => analyzeComplex(x));
    // a group helps only if every alternative is constrained
    if (alts.length && alts.every((a) => !a.unconstrained)) groups.push(alts);
    return "";
  });
  rest = rest.replace(/:(not|has|nth-child|nth-last-child|nth-of-type|nth-last-of-type|host|host-context|slotted|part|lang|dir|state)\(((?:[^()]|\((?:[^()]|\([^()]*\))*\))*)\)/g, "");
  rest = rest.replace(/\[[^\]]*\]/g, "");
  for (const m of rest.matchAll(new RegExp(String.raw`\.(${IDENT})`, "g"))) terms.push({ kind: "class", name: unescapeIdent(m[1]) });
  for (const m of rest.matchAll(new RegExp(String.raw`#(${IDENT})`, "g"))) terms.push({ kind: "id", name: unescapeIdent(m[1]) });
  return { terms, groups };
}
const unescapeIdent = (s) => s.replace(/\\(.)/g, "$1");

// One complex selector: the first compound from the RIGHT that carries a constraint decides (it is a necessary
// condition: `.a button` styles only buttons inside `.a`, so its pages are among the pages of `.a`).
function analyzeComplex(sel) {
  const cs = compounds(sel);
  for (let i = cs.length - 1; i >= 0; i--) {
    const { terms, groups } = compoundTerms(cs[i]);
    if (terms.length || groups.length) return { terms, groups, unconstrained: false, subject: i === cs.length - 1 };
  }
  return { terms: [], groups: [], unconstrained: true, subject: false };
}

export function analyzeSelector(sel) {
  return splitTop(sel, ",").map((x) => x.trim()).filter(Boolean).map((x) => ({ sel: x, ...analyzeComplex(x) }));
}
