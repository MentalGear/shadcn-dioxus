// The Playwright tools' reader of TypeScript / JavaScript source. select.mjs asks it which files import a changed helper;
// suite-key.mjs asks it which environment variables a file reads and whether it does anything the suite key cannot account for.
//
// The TypeScript parser comes from scripts/ts-tools only (its pinned `typescript`); there is no second place it is looked for.
// Without it `parserName()` is null: select.mjs reads import specifiers by regular expressions (over-inclusive: a mention in a
// comment is an edge), and suite-key.mjs derives no key at all (the whole suite runs).
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const SOURCE_RE = /\.(?:ts|tsx|mts|cts|js|mjs|cjs)$/;
const EXTS = [".ts", ".tsx", ".mts", ".cts", ".js", ".mjs", ".cjs", ".json"];

let ts = null;
try { ts = createRequire(path.join(HERE, "..", "..", "scripts", "ts-tools", "package.json"))("typescript"); } catch { /* no parser */ }
/** The parser in use, or null. */
export const parserName = () => (ts ? `typescript ${ts.version}` : null);

// `process.<member>` forms that cannot read the environment.
const PROCESS_MEMBERS = new Set(["argv", "exit", "exitCode", "stdout", "stderr", "stdin", "cwd", "platform", "arch", "versions", "version", "pid",
  "ppid", "on", "once", "off", "emit", "kill", "hrtime", "memoryUsage", "nextTick", "uptime", "execPath", "title", "release", "removeListener", "chdir", "umask"]);
const ENUMERATORS = /^(?:readdir|readdirSync|opendir|opendirSync|glob|globSync|scandir)$/;

const REGEX_SPEC_RES = [/\bfrom\s*["']([^"'\n]+)["']/g, /\bimport\s*["']([^"'\n]+)["']/g, /\bimport\s*\(\s*["']([^"'\n]+)["']\s*\)/g, /\brequire\s*\(\s*["']([^"'\n]+)["']\s*\)/g];

/** Import specifiers by regular expressions over the raw text (select.mjs without a parser). */
function specifiersByRegex(text) {
  const out = new Set();
  for (const re of REGEX_SPEC_RES) for (const m of text.matchAll(re)) out.add(m[1]);
  return [...out];
}

/**
 * What one source file does that a suite key has to account for (needs the parser):
 *   specifiers   every static module specifier (import / export from / require / import() / import("x") type)
 *   env          the names of `process.env.NAME`, `process.env["NAME"]` and `env.NAME` accesses
 *   processSites uses of `process` outside `process.env.NAME` and the members that cannot read the environment: a bare
 *                `process.env`, `process.env[expr]`, `const p = process`, `globalThis.process`, an import of `process` ...
 *   specNaming   string literals that contain ".spec." and imports of a `.spec` module (a test that names another spec file)
 *   enumerates   readdir / opendir / glob calls (a test that could enumerate spec files)
 *   toolsImports imports of a module under tools/
 *   computed     import() / require() with a computed argument
 */
export function analyze(text, file = "x.ts") {
  if (!ts) throw new Error("no TypeScript parser (scripts/ts-tools)");
  const kind = /\.(?:cjs|mjs|js)$/.test(file) ? ts.ScriptKind.JS : ts.ScriptKind.TS;
  const sf = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, kind);
  const r = { specifiers: new Set(), env: new Set(), processSites: 0, specNaming: 0, enumerates: 0, toolsImports: 0, computed: 0 };
  const lit = (n) => n && (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n));
  const spec = (s) => {
    r.specifiers.add(s);
    if (/\.spec(\.|$)/.test(s)) r.specNaming++;
    if (/(^|\/)tools(\/|$)/.test(s)) r.toolsImports++;
    if (/^(?:node:)?process$/.test(s)) r.processSites++;
  };
  const isProcess = (n) => ts.isIdentifier(n) && n.text === "process";
  const visit = (n) => {
    if ((ts.isImportDeclaration(n) || ts.isExportDeclaration(n)) && lit(n.moduleSpecifier)) spec(n.moduleSpecifier.text);
    else if (ts.isImportEqualsDeclaration(n) && ts.isExternalModuleReference(n.moduleReference) && lit(n.moduleReference.expression)) spec(n.moduleReference.expression.text);
    else if (ts.isCallExpression(n) && (n.expression.kind === ts.SyntaxKind.ImportKeyword || (ts.isIdentifier(n.expression) && n.expression.text === "require"))) {
      if (lit(n.arguments[0])) spec(n.arguments[0].text); else r.computed++;
    } else if (ts.isImportTypeNode(n) && ts.isLiteralTypeNode(n.argument) && lit(n.argument.literal)) spec(n.argument.literal.text);
    if (ts.isCallExpression(n)) {
      const e = n.expression, name = ts.isIdentifier(e) ? e.text : ts.isPropertyAccessExpression(e) ? e.name.text : "";
      if (ENUMERATORS.test(name)) r.enumerates++;
    }
    if (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n) || ts.isTemplateHead(n) || ts.isTemplateMiddle(n) || ts.isTemplateTail(n)) {
      if (n.text.includes(".spec.")) r.specNaming++;
      if (n.text === "process" && n.parent && ts.isElementAccessExpression(n.parent)) r.processSites++;
    }
    if (ts.isPropertyAccessExpression(n) && ts.isIdentifier(n.expression) && n.expression.text === "env" && /^[A-Z][A-Z0-9_]{2,}$/.test(n.name.text)) r.env.add(n.name.text);
    if (isProcess(n)) {
      const p = n.parent;
      const named = p && ((ts.isPropertyAccessExpression(p) && p.name === n) || (ts.isPropertyAssignment(p) && p.name === n) || (ts.isMethodDeclaration(p) && p.name === n));
      if (named) {
        if (ts.isPropertyAccessExpression(p) && ts.isIdentifier(p.expression) && /^(?:globalThis|global|self|window)$/.test(p.expression.text)) r.processSites++;
      } else if (p && ts.isPropertyAccessExpression(p) && p.expression === n) {
        if (p.name.text === "env") {
          const g = p.parent;
          if (ts.isPropertyAccessExpression(g) && g.expression === p) r.env.add(g.name.text);
          else if (ts.isElementAccessExpression(g) && g.expression === p && lit(g.argumentExpression)) r.env.add(g.argumentExpression.text);
          else r.processSites++;
        } else if (!PROCESS_MEMBERS.has(p.name.text)) r.processSites++;
      } else r.processSites++;
    }
    ts.forEachChild(n, visit);
  };
  visit(sf);
  return r;
}

/** Every static module specifier of a source text (the parser when there is one, else regular expressions). */
export const specifiersOf = (text, file = "x.ts") => (ts ? [...analyze(text, file).specifiers] : specifiersByRegex(text));

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

/** Every file of `files` (a Map path -> text) that loads `target`, transitively. */
export function importersOf(target, files) {
  const exists = (p) => files.has(p), graph = new Map();
  for (const [f, text] of files) {
    if (!SOURCE_RE.test(f)) continue;
    const deps = new Set();
    for (const spec of specifiersOf(text, f)) { const r = resolveSpecifier(f, spec, exists); if (r.file) deps.add(r.file); }
    graph.set(f, deps);
  }
  const hit = new Set(), queue = [target];
  while (queue.length) {
    const t = queue.pop();
    for (const [f, deps] of graph) if (!hit.has(f) && f !== target && deps.has(t)) { hit.add(f); queue.push(f); }
  }
  return hit;
}
