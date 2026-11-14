// The one import walker of the Playwright tools: which files a TypeScript / JavaScript file loads, and which environment
// variables it reads. select.mjs asks it for the importers of a changed helper, suite-key.mjs for the files and variables a
// spec's result depends on.
//
// Source text is read by the TypeScript parser (the pinned `typescript` of scripts/ts-tools, or one installed in playwright/):
// a specifier is a string literal of an import / export declaration, `import x = require()`, `import()`, `require()` or an
// `import("x")` type; a `process.env.NAME` / `process.env["NAME"]` / `env.NAME` access is a variable; a computed `import()`,
// `require()` or `process.env[expr]`, and a `process.env` used as a whole, are reported as opaque. Without a parser the same
// forms are read by regular expressions over the raw text with NO comment stripping: a regular expression cannot tell a
// comment from a string, so it reads every mention and errs toward more edges (more re-runs), never fewer.
// A relative specifier resolves to one file by the Node and TypeScript extension rules; a bare one (a package, `node:fs`) is the
// lock file's business and is not an edge. A relative specifier that resolves to no file is `unresolved`: a caller that needs
// soundness treats the file as unknown.
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXTS = [".ts", ".tsx", ".mts", ".cts", ".js", ".mjs", ".cjs", ".json"];
export const SOURCE_RE = /\.(?:ts|tsx|mts|cts|js|mjs|cjs)$/;

let ts = null;
for (const where of [path.join(HERE, "..", "..", "scripts", "ts-tools", "package.json"), path.join(HERE, "..", "package.json")]) {
  try { ts = createRequire(where)("typescript"); break; } catch { /* the next place */ }
}
/** Which reader is in use; both keys of suite-key.mjs hash it, so a run with the other reader never reuses this one's records. */
export const parserName = () => (ts ? `typescript ${ts.version}` : "regex");

const SPEC_RES = [
  /\bfrom\s*["']([^"'\n]+)["']/g,
  /\bimport\s*["']([^"'\n]+)["']/g,
  /\bimport\s*\(\s*["']([^"'\n]+)["']\s*\)/g,
  /\brequire\s*\(\s*["']([^"'\n]+)["']\s*\)/g,
];

function analyzeRegex(text) {
  const specifiers = new Set(), env = new Set(), opaque = [];
  for (const re of SPEC_RES) for (const m of text.matchAll(re)) specifiers.add(m[1]);
  for (const m of text.matchAll(/\b(?:import|require)\s*\(\s*([^"'\s)][^)]*)\)/g)) opaque.push(m[0].slice(0, 60));
  for (const m of text.matchAll(/\bprocess\.env\.([A-Za-z_][A-Za-z0-9_]*)/g)) env.add(m[1]);
  for (const m of text.matchAll(/\bprocess\.env\[\s*["']([A-Za-z_][A-Za-z0-9_]*)["']\s*\]/g)) env.add(m[1]);
  for (const m of text.matchAll(/(?<![\w.])env\.([A-Z][A-Z0-9_]{2,})\b/g)) env.add(m[1]);
  const bare = [...text.matchAll(/\bprocess\.env\b(?!\.|\s*\[\s*["'])/g)].length;
  return { specifiers: [...specifiers], env, opaque, bare };
}

function analyzeTs(text, file) {
  const kind = /\.(?:cjs|mjs|js)$/.test(file) ? ts.ScriptKind.JS : ts.ScriptKind.TS;
  const sf = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, kind);
  const specifiers = new Set(), env = new Set(), opaque = [];
  let bare = 0;
  const lit = (n) => n && (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n));
  const isProcessEnv = (n) => ts.isPropertyAccessExpression(n) && n.name.text === "env" && ts.isIdentifier(n.expression) && n.expression.text === "process";
  const visit = (n) => {
    if ((ts.isImportDeclaration(n) || ts.isExportDeclaration(n)) && lit(n.moduleSpecifier)) specifiers.add(n.moduleSpecifier.text);
    else if (ts.isImportEqualsDeclaration(n) && ts.isExternalModuleReference(n.moduleReference) && lit(n.moduleReference.expression)) specifiers.add(n.moduleReference.expression.text);
    else if (ts.isCallExpression(n) && (n.expression.kind === ts.SyntaxKind.ImportKeyword || (ts.isIdentifier(n.expression) && n.expression.text === "require"))) {
      if (lit(n.arguments[0])) specifiers.add(n.arguments[0].text); else opaque.push(n.getText(sf).slice(0, 60));
    } else if (ts.isImportTypeNode(n) && ts.isLiteralTypeNode(n.argument) && lit(n.argument.literal)) specifiers.add(n.argument.literal.text);
    else if (ts.isPropertyAccessExpression(n) && isProcessEnv(n.expression)) env.add(n.name.text);
    else if (ts.isPropertyAccessExpression(n) && ts.isIdentifier(n.expression) && n.expression.text === "env" && /^[A-Z][A-Z0-9_]{2,}$/.test(n.name.text)) env.add(n.name.text);
    else if (ts.isElementAccessExpression(n) && isProcessEnv(n.expression)) { if (lit(n.argumentExpression)) env.add(n.argumentExpression.text); else bare++; }
    else if (isProcessEnv(n) && !(n.parent && (ts.isPropertyAccessExpression(n.parent) || ts.isElementAccessExpression(n.parent)) && n.parent.expression === n)) bare++;
    ts.forEachChild(n, visit);
  };
  visit(sf);
  return { specifiers: [...specifiers], env, opaque, bare };
}

/** { specifiers: [string], env: Set(name), opaque: [text], bare: count } of one source file. */
export function analyze(text, file = "x.ts") { return ts ? analyzeTs(text, file) : analyzeRegex(text); }

/** Every static module specifier a source text names. */
export const specifiersOf = (text, file = "x.ts") => analyze(text, file).specifiers;

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
    if (!SOURCE_RE.test(f)) continue; // a data file loads nothing
    const a = analyze(read(f), f);
    for (const o of a.opaque) opaque.push({ from: f, text: o });
    for (const spec of a.specifiers) {
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
    if (!SOURCE_RE.test(f)) continue;
    const deps = new Set();
    for (const spec of specifiersOf(text, f)) { const r = resolveSpecifier(f, spec, exists); if (r.file) deps.add(r.file); }
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
