// The one import walker of the Playwright tools: which files a TypeScript / JavaScript file loads. select.mjs asks it for the
// importers of a changed helper, suite-key.mjs for the files a spec's result depends on.
//
// A specifier is read from every static form (`from "x"`, `import "x"`, `import("x")`, `require("x")`, `export ... from "x"`).
// A relative or absolute-from-root specifier resolves to one file by the Node and TypeScript extension rules; a bare specifier
// (a package, `node:fs`) is the lock file's business and is not an edge. A relative specifier that resolves to no file is
// returned as `unresolved`: a caller that needs soundness treats the file as unknown, it never skips the edge.
import path from "node:path";

const EXTS = [".ts", ".tsx", ".mts", ".cts", ".js", ".mjs", ".cjs", ".json"];

// Comments out of the way (a specifier named in a comment is not an edge): whole-line `//` and `/* */` blocks, as select.mjs
// reads other sources. A `//` inside a string on a code line stays.
export const stripComments = (t) => t.replace(/^\s*\/\/.*$/gm, "").replace(/\/\*[\s\S]*?\*\//g, "");

const SPEC_RES = [
  /\bfrom\s*["']([^"'\n]+)["']/g,
  /\bimport\s*["']([^"'\n]+)["']/g,
  /\bimport\s*\(\s*["']([^"'\n]+)["']\s*\)/g,
  /\brequire\s*\(\s*["']([^"'\n]+)["']\s*\)/g,
];

/** Every static module specifier a source text names, in order of appearance, without duplicates. */
export function specifiersOf(text) {
  const t = stripComments(text);
  const out = new Set();
  for (const re of SPEC_RES) for (const m of t.matchAll(re)) out.add(m[1]);
  return [...out];
}

/** Dynamic loads this walker cannot read: `import(x)` / `require(x)` with a computed argument. */
export function opaqueLoads(text) {
  const t = stripComments(text);
  const out = [];
  for (const m of t.matchAll(/\b(?:import|require)\s*\(\s*([^"'\s)][^)]*)\)/g)) out.push(m[0].slice(0, 60));
  return out;
}

const join = (dir, spec) => path.posix.normalize(path.posix.join(dir, spec));

/**
 * The file a specifier of `fromFile` names, among the files `exists` knows. Returns { file } for a hit, { external: true }
 * for a bare specifier, { unresolved: spec } for a relative one that names no file. Paths are posix and relative to one root.
 */
export function resolveSpecifier(fromFile, spec, exists) {
  if (!spec.startsWith(".") && !spec.startsWith("/")) return { external: true };
  const base = spec.startsWith("/") ? spec.slice(1) : join(path.posix.dirname(fromFile), spec);
  const candidates = [base, ...EXTS.map((e) => base + e), ...EXTS.map((e) => `${base}/index${e}`)];
  const hit = candidates.find((c) => exists(c));
  return hit ? { file: hit } : { unresolved: spec };
}

/** Every file a file loads, transitively (the file itself included): `{ files: Set, unresolved: [{from, spec}], opaque: [{from, text}] }`. */
export function importClosure(entry, { exists, read }) {
  const files = new Set([entry]), unresolved = [], opaque = [], queue = [entry];
  while (queue.length) {
    const f = queue.pop();
    if (!/\.(?:ts|tsx|mts|cts|js|mjs|cjs)$/.test(f)) continue; // a data file loads nothing
    const text = read(f);
    for (const o of opaqueLoads(text)) opaque.push({ from: f, text: o });
    for (const spec of specifiersOf(text)) {
      const r = resolveSpecifier(f, spec, exists);
      if (r.unresolved) unresolved.push({ from: f, spec });
      else if (r.file && !files.has(r.file)) { files.add(r.file); queue.push(r.file); }
    }
  }
  return { files, unresolved, opaque };
}

/** The forward graph of `files` (a Map path -> text): path -> Set of the files it names directly. */
export function importGraph(files) {
  const exists = (p) => files.has(p);
  const graph = new Map();
  for (const [f, text] of files) {
    if (!/\.(?:ts|tsx|mts|cts|js|mjs|cjs)$/.test(f)) continue;
    const deps = new Set();
    for (const spec of specifiersOf(text)) { const r = resolveSpecifier(f, spec, exists); if (r.file) deps.add(r.file); }
    graph.set(f, deps);
  }
  return graph;
}

/** Every file of `files` that loads `target`, transitively. */
export function importersOf(target, files) {
  const graph = importGraph(files), hit = new Set(), queue = [target];
  while (queue.length) {
    const t = queue.pop();
    for (const [f, deps] of graph) if (!hit.has(f) && f !== target && deps.has(t)) { hit.add(f); queue.push(f); }
  }
  return hit;
}
