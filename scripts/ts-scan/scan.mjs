#!/usr/bin/env node
// scan.mjs   (backlog row 444, phase 0.1; rule text: dev-docs/research/sync-js-root-2026-10-26.md section 5.3 item 4)
//
// The TypeScript side of the sink gates, read with the TypeScript compiler API and its checker, not with a
// spelling denylist. `bridge.ts` types receivers as `any`, so `e[name]()`, `self['ev'+'al']('1')`,
// `Reflect.get(e, 'prevent'+'Default')`, `(0, W.eval)('1')` and `(()=>{}).constructor('return 1')()` all pass a
// regex; each of them is a node shape here.
//
// Rules (every file of `primitives/src/ts/**/*.ts`, `.d.ts` excluded):
//   computed-access    an element access whose argument is not a literal, outside `own(TABLE, key)`
//   string-sink        a call or `new` that resolves (or, on an `any` receiver, is named) `eval`, `Function`,
//                      `setTimeout` or `setInterval` with a string (or string-typed / `any`-typed) argument
//   dynamic-key-call   `Reflect.get/set/has/defineProperty/...` or `Object.defineProperty/...` with a non-literal key,
//                      and every `Reflect.apply` / `Reflect.construct`
//   constructor        a `.constructor` member (or `["constructor"]`)
//   verb-row           a row of a closed verb table (METHODS, PROPS, FIELD_GETTERS, ...) that is more than one
//                      member call or member read plus at most one guard `if`
//   any, as-any        an explicit `any` / `x as any` (tree-wide: shrink-only against the baseline)
// Under `ts/behaviours/**` (a path segment `behaviours`), hard rules with no baseline:
//   any, as-any, constructor, banned-member (`__proto__`, `prototype`), computed-access except on a typed record
//   (a non-`any` type with an index signature), free-ident (a free identifier outside the allowlist, by checker scope
//   analysis: `globalThis.document`, `window`, `Reflect`, `Object.assign` are red), behaviour-import (anything but
//   `lib/register`, the generated `behaviours.gen`, or a type-only import), ts-suppress (`@ts-ignore` and friends).
//
// SHRINK-ONLY BASELINE `scripts/ts-scan/bridge.baseline.tsv`: the sites that predate the rules, one row per
// `file, rule, context, hash` with a count. `context` is the enclosing function path and `hash` is a hash of the
// whitespace-normalised expression text, so a line move changes nothing, an edit of a baselined site or a new site is
// red, and a baseline row the tree no longer needs is red too ("stale": remove it). `--update-baseline` only ever
// removes or lowers rows, never adds; a first baseline is written with `--init-baseline` when the file is absent.
//
// TypeScript is not pinned in the repo: `TS_MODULE` below is the ONE place the compiler is resolved (the sandbox
// image's /opt/node-tools; `TS_SCAN_TYPESCRIPT` overrides it). Pinning it later is a change of that one line.
//
// Usage: node scripts/ts-scan/scan.mjs [--self-test | --update-baseline | --init-baseline | --list]
// Exit status: 0 = green, 1 = a rule is red, 2 = bad usage.
import { createRequire } from "node:module";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const TS_MODULE = process.env.TS_SCAN_TYPESCRIPT || "/opt/node-tools/node_modules/typescript";
const ts = createRequire(import.meta.url)(TS_MODULE);

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../..");
const TSCONFIG = path.join(ROOT, "primitives/tsconfig.json");
const BASELINE = path.join(HERE, "bridge.baseline.tsv");
const FIXTURES = path.join(HERE, "fixtures");
const DOC = "dev-docs/js-interop.md";

const RULES = {
  "computed-access": "a computed member access with a non-literal key reaches any property, `constructor` and `__proto__` included: read a closed table through `own(TABLE, key)`, or type the receiver",
  "string-sink": "a string reaching eval, Function, setTimeout or setInterval is code: pass a function, or add a verb row",
  "dynamic-key-call": "Reflect and Object property functions with a computed key are `e[name]` by another name: use a closed table row",
  "constructor": "`.constructor` reaches Function and so eval",
  "verb-row": "a verb row is one member call or read plus at most one guard `if`; anything larger is a registered behaviour with a row and vectors",
  "any": "an explicit `any` switches the checker off for everything it touches: type the value (Element, Window, Event, a record)",
  "as-any": "`as any` switches the checker off: narrow with a typed cast or a guard",
  "banned-member": "`__proto__` and `prototype` reach the prototype chain",
  "free-ident": "a behaviour may name only Math, Number, String, Array, Object.keys, Object.freeze and RegExp (plus its own scope), so it cannot reach the DOM through a global",
  "behaviour-import": "a behaviour imports only `lib/register`, the generated `behaviours.gen` and types",
  "ts-suppress": "a `@ts-ignore`/`@ts-expect-error`/`@ts-nocheck` comment turns the checker off",
};
const HARD_ONLY = new Set(["banned-member", "free-ident", "behaviour-import", "ts-suppress"]);

const SINK_NAMES = new Set(["eval", "Function", "setTimeout", "setInterval"]);
const VERB_TABLES = new Set(["METHODS", "PROPS", "FIELD_GETTERS", "SETTERS", "SETS", "REDUCERS"]);
const FREE_OK = new Set(["Math", "Number", "String", "Array", "RegExp", "undefined", "NaN", "Infinity"]);
const OBJECT_OK = new Set(["keys", "freeze"]);
const KEY_ARG1 = {
  Reflect: new Set(["get", "set", "has", "defineProperty", "getOwnPropertyDescriptor", "deleteProperty"]),
  Object: new Set(["defineProperty", "getOwnPropertyDescriptor"]),
};
const REFLECT_ALWAYS = new Set(["apply", "construct"]);

const rel = (file) => path.relative(ROOT, file).split(path.sep).join("/");
const inBehaviours = (file) => rel(file).split("/").includes("behaviours");
const norm = (text) => text.replace(/\s+/g, " ").trim();
const sha = (text) => crypto.createHash("sha1").update(norm(text)).digest("hex").slice(0, 10);

// ------------------------------------------------------------------------------------------ the program

function makeProgram(rootNames) {
  const cfg = ts.readConfigFile(TSCONFIG, ts.sys.readFile);
  const parsed = ts.parseJsonConfigFileContent(cfg.config, ts.sys, path.dirname(TSCONFIG));
  const options = { ...parsed.options, noEmit: true };
  return { program: ts.createProgram(rootNames ?? parsed.fileNames, options), parsed };
}

function treeFiles() {
  const { parsed } = makeProgram([]);
  return parsed.fileNames.filter((f) => !f.endsWith(".d.ts")).sort();
}

// ------------------------------------------------------------------------------------------ the scan

function scanFile(program, checker, sf) {
  const out = [];
  const beh = inBehaviours(sf.fileName);
  const text = (n) => n.getText(sf);

  const nameOf = (n) => {
    if (n.name && (ts.isIdentifier(n.name) || ts.isStringLiteral(n.name))) return n.name.text;
    return undefined;
  };
  const contextOf = (node) => {
    const parts = [];
    for (let n = node.parent; n; n = n.parent) {
      if (ts.isFunctionDeclaration(n) || ts.isClassDeclaration(n) || ts.isMethodDeclaration(n) || ts.isPropertyAssignment(n)) {
        const name = nameOf(n);
        if (name) parts.unshift(name);
      } else if (ts.isVariableDeclaration(n) && ts.isIdentifier(n.name)) {
        parts.unshift(n.name.text);
      }
    }
    return parts.length ? parts.join(".") : "<module>";
  };
  const report = (node, rule, detail, ctxNode = node, exprNode = node) => {
    const { line, character } = sf.getLineAndCharacterOfPosition(node.getStart(sf));
    out.push({
      file: rel(sf.fileName), line: line + 1, col: character + 1, rule, context: contextOf(ctxNode),
      hash: sha(text(exprNode)), detail,
    });
  };

  const strip = (e) => {
    for (;;) {
      if (ts.isParenthesizedExpression(e) || ts.isNonNullExpression(e) || ts.isAsExpression(e) ||
          ts.isTypeAssertionExpression(e) || ts.isSatisfiesExpression(e)) e = e.expression;
      else if (ts.isBinaryExpression(e) && e.operatorToken.kind === ts.SyntaxKind.CommaToken) e = e.right;
      else return e;
    }
  };
  const isLiteralKey = (e) => {
    e = strip(e);
    return ts.isStringLiteral(e) || ts.isNoSubstitutionTemplateLiteral(e) || ts.isNumericLiteral(e);
  };
  const literalValue = (e) => {
    e = strip(e);
    return ts.isStringLiteral(e) || ts.isNoSubstitutionTemplateLiteral(e) ? e.text : undefined;
  };
  const isLibDecl = (d) => {
    const f = d.getSourceFile();
    return program.isSourceFileDefaultLibrary(f) || /[\\/]typescript[\\/]lib[\\/]/.test(f.fileName);
  };
  // true when the symbol of `node` is declared in the scanned code (not the language's own libraries)
  const userDefined = (node) => {
    const sym = checker.getSymbolAtLocation(node);
    return !!sym && !!sym.declarations && sym.declarations.length > 0 && sym.declarations.every((d) => !isLibDecl(d));
  };
  const calleeName = (callee) => {
    const e = strip(callee);
    if (ts.isIdentifier(e)) return { name: e.text, node: e };
    if (ts.isPropertyAccessExpression(e)) return { name: e.name.text, node: e.name };
    if (ts.isElementAccessExpression(e)) {
      const v = literalValue(e.argumentExpression);
      return v === undefined ? undefined : { name: v, node: e.argumentExpression };
    }
    return undefined;
  };
  const stringish = (arg) => {
    const e = strip(arg);
    if (ts.isStringLiteral(e) || ts.isNoSubstitutionTemplateLiteral(e) || ts.isTemplateExpression(e)) return true;
    if (ts.isBinaryExpression(e) && e.operatorToken.kind === ts.SyntaxKind.PlusToken) return stringish(e.left) || stringish(e.right);
    const t = checker.getTypeAtLocation(e);
    const flags = ts.TypeFlags.StringLike | ts.TypeFlags.Any | ts.TypeFlags.Unknown;
    return t.flags & flags ? true : !!t.isUnion() && t.types.some((x) => x.flags & flags);
  };

  const typedRecord = (n) => {
    const t = checker.getNonNullableType(checker.getTypeAtLocation(n.expression));
    if (t.flags & ts.TypeFlags.Any) return false;
    const key = checker.getTypeAtLocation(n.argumentExpression);
    const hasIndex = (kind) => checker.getIndexInfosOfType(t).some((i) => i.keyType.flags & kind);
    if (key.flags & ts.TypeFlags.NumberLike) return hasIndex(ts.TypeFlags.Number);
    return hasIndex(ts.TypeFlags.String);
  };

  // ---- verb rows
  const rowCost = (n) => {
    if (!n || ts.isTypeNode(n)) return 0;
    if (ts.isConditionalExpression(n)) return Math.max(rowCost(n.whenTrue), rowCost(n.whenFalse));
    if (ts.isPropertyAccessExpression(n)) {
      let base = n.expression;
      while (ts.isPropertyAccessExpression(base)) base = base.expression;
      return 1 + rowCost(base);
    }
    let sum = 0;
    ts.forEachChild(n, (c) => { sum += rowCost(c); });
    return sum;
  };
  const stmtCost = (s) => {
    if (ts.isIfStatement(s)) return Math.max(stmtCost(s.thenStatement), s.elseStatement ? stmtCost(s.elseStatement) : 0);
    if (ts.isBlock(s)) return s.statements.reduce((a, x) => a + stmtCost(x), 0);
    if (ts.isExpressionStatement(s) || ts.isReturnStatement(s)) return rowCost(s.expression);
    if (ts.isVariableStatement(s)) return rowCost(s.declarationList);
    return Infinity;
  };
  const checkRow = (fn, label) => {
    const body = fn.body;
    let why = null;
    if (!ts.isBlock(body)) {
      if (rowCost(body) > 1) why = "more than one member call or read";
    } else {
      let ifs = 0;
      const countIfs = (n) => { if (ts.isIfStatement(n)) ifs++; ts.forEachChild(n, countIfs); };
      countIfs(body);
      const c = stmtCost(body);
      if (ifs > 1) why = "more than one guard `if`";
      else if (c === Infinity) why = "a statement that is neither a call, a read nor a guard";
      else if (c > 1) why = "more than one member call or read";
    }
    if (why) report(fn, "verb-row", `row ${label}: ${why}`, fn.parent, fn);
  };
  const isFn = (n) => n && (ts.isArrowFunction(n) || ts.isFunctionExpression(n) || ts.isMethodDeclaration(n));
  const checkTable = (decl) => {
    let init = decl.initializer && strip(decl.initializer);
    if (!init || !ts.isObjectLiteralExpression(init)) return;
    const table = decl.name.text;
    for (const row of init.properties) {
      const key = nameOf(row) ?? "?";
      if (ts.isMethodDeclaration(row)) { checkRow(row, `${table}.${key}`); continue; }
      if (!ts.isPropertyAssignment(row)) continue;
      const v = strip(row.initializer);
      if (isFn(v)) checkRow(v, `${table}.${key}`);
      else if (ts.isObjectLiteralExpression(v)) {
        for (const m of v.properties) {
          if (ts.isMethodDeclaration(m)) checkRow(m, `${table}.${key}.${nameOf(m)}`);
          else if (ts.isPropertyAssignment(m) && isFn(strip(m.initializer))) checkRow(strip(m.initializer), `${table}.${key}.${nameOf(m)}`);
        }
      }
    }
  };

  // ---- free identifiers
  const isReference = (id) => {
    const p = id.parent;
    if (ts.isPropertyAccessExpression(p)) return p.expression === id;
    if (ts.isQualifiedName(p)) return p.left === id;
    if (ts.isPropertyAssignment(p)) return p.initializer === id;
    if (ts.isShorthandPropertyAssignment(p)) return true;
    if (ts.isBindingElement(p)) return p.initializer === id;
    if (ts.isImportSpecifier(p) || ts.isExportSpecifier(p) || ts.isImportClause(p) || ts.isNamespaceImport(p)) return false;
    if (ts.isLabeledStatement(p) || ts.isBreakOrContinueStatement(p)) return false;
    if ("name" in p && p.name === id) return false; // a declaration's own name or a member name
    return true;
  };
  const inType = (n) => {
    for (let a = n.parent; a && !ts.isStatement(a); a = a.parent) if (ts.isTypeNode(a)) return true;
    return false;
  };
  const checkIdentifier = (id) => {
    if (!isReference(id) || inType(id)) return;
    const sym = ts.isShorthandPropertyAssignment(id.parent)
      ? checker.getShorthandAssignmentValueSymbol(id.parent)
      : checker.getSymbolAtLocation(id);
    const local = sym && sym.declarations && sym.declarations.length > 0 && sym.declarations.every((d) => !isLibDecl(d) && !d.getSourceFile().isDeclarationFile);
    if (local) return;
    if (id.text === "Object") {
      const p = id.parent;
      if (ts.isPropertyAccessExpression(p) && p.expression === id && OBJECT_OK.has(p.name.text)) return;
      report(id, "free-ident", "free identifier `Object` (only Object.keys and Object.freeze are allowed)", id, id.parent);
      return;
    }
    if (FREE_OK.has(id.text)) return;
    report(id, "free-ident", `free identifier \`${id.text}\``, id, id.parent);
  };

  const visit = (n) => {
    // computed access, constructor, banned members
    if (ts.isElementAccessExpression(n)) {
      const lit = literalValue(n.argumentExpression);
      if (lit === "constructor") report(n, "constructor", "`[\"constructor\"]`");
      else if (beh && (lit === "__proto__" || lit === "prototype")) report(n, "banned-member", `\`["${lit}"]\``);
      else if (!isLiteralKey(n.argumentExpression)) {
        let fn = n.parent;
        while (fn && !ts.isFunctionDeclaration(fn)) fn = fn.parent;
        const inOwn = !beh && fn && fn.name && fn.name.text === "own";
        if (!inOwn && !(beh && typedRecord(n))) report(n, "computed-access", "non-literal key");
      }
    } else if (ts.isPropertyAccessExpression(n)) {
      const name = n.name.text;
      if (name === "constructor") report(n, "constructor", "`.constructor`");
      else if (beh && (name === "__proto__" || name === "prototype")) report(n, "banned-member", `\`.${name}\``);
    } else if (ts.isComputedPropertyName(n)) {
      const lit = literalValue(n.expression);
      if (lit === "constructor") report(n, "constructor", "computed `constructor` key");
      else if (beh && (lit === "__proto__" || lit === "prototype")) report(n, "banned-member", `computed \`${lit}\` key`);
    } else if (beh && (ts.isPropertyAssignment(n) || ts.isMethodDeclaration(n)) && nameOf(n) === "__proto__") {
      report(n, "banned-member", "an `__proto__` property");
    }

    // sinks
    if (ts.isCallExpression(n) || ts.isNewExpression(n)) {
      const args = n.arguments ?? [];
      const c = calleeName(n.expression);
      if (c && SINK_NAMES.has(c.name) && !userDefined(c.node)) {
        const bad = c.name === "Function" ? args.some(stringish) : args.length > 0 && stringish(args[0]);
        if (bad) report(n, "string-sink", `\`${c.name}\` with a string argument`);
      }
      const callee = strip(n.expression);
      if (ts.isCallExpression(n) && ts.isPropertyAccessExpression(callee) && ts.isIdentifier(callee.expression)) {
        const obj = callee.expression.text;
        const fnName = callee.name.text;
        if ((obj === "Reflect" || obj === "Object") && !userDefined(callee.expression)) {
          if (obj === "Reflect" && REFLECT_ALWAYS.has(fnName)) report(n, "dynamic-key-call", `Reflect.${fnName}`);
          else if (KEY_ARG1[obj].has(fnName) && args[1] && !isLiteralKey(args[1])) report(n, "dynamic-key-call", `${obj}.${fnName} with a computed key`);
        }
      }
    }

    // `any`
    if (n.kind === ts.SyntaxKind.AnyKeyword) {
      const p = n.parent;
      if (!((ts.isAsExpression(p) || ts.isTypeAssertionExpression(p)) && p.type === n)) report(n, "any", "explicit `any`");
    } else if ((ts.isAsExpression(n) || ts.isTypeAssertionExpression(n)) && n.type.kind === ts.SyntaxKind.AnyKeyword) {
      report(n, "as-any", "`as any`");
    }

    if (beh) {
      if (ts.isIdentifier(n)) checkIdentifier(n);
      if (ts.isImportDeclaration(n)) {
        const spec = n.moduleSpecifier.text;
        const typeOnly = n.importClause && n.importClause.isTypeOnly;
        if (!typeOnly && !/(^|\/)lib\/register$/.test(spec) && !/(^|\/)behaviours\.gen$/.test(spec)) {
          report(n, "behaviour-import", `import of "${spec}"`);
        }
      }
      if (ts.isCallExpression(n) && n.expression.kind === ts.SyntaxKind.ImportKeyword) report(n, "behaviour-import", "dynamic import()");
    }

    if (ts.isVariableDeclaration(n) && ts.isIdentifier(n.name) && VERB_TABLES.has(n.name.text)) checkTable(n);
    ts.forEachChild(n, visit);
  };
  visit(sf);

  if (beh) {
    const re = /@ts-(ignore|expect-error|nocheck)/g;
    for (let m; (m = re.exec(sf.text));) {
      const { line, character } = sf.getLineAndCharacterOfPosition(m.index);
      out.push({ file: rel(sf.fileName), line: line + 1, col: character + 1, rule: "ts-suppress", context: "<module>", hash: sha(m[0] + ":" + (line + 1)), detail: m[0] });
    }
  }
  return out;
}

function scan(files) {
  const { program } = makeProgram(files);
  const checker = program.getTypeChecker();
  const found = [];
  for (const f of files) {
    const sf = program.getSourceFile(f);
    if (!sf) throw new Error(`ts-scan: ${f} is not in the program`);
    found.push(...scanFile(program, checker, sf));
  }
  return found;
}

// ------------------------------------------------------------------------------------------ the baseline

const keyOf = (f) => [f.file, f.rule, f.context, f.hash].join("\t");

function parseBaseline(textIn) {
  const map = new Map();
  for (const line of textIn.split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const cols = line.split("\t");
    if (cols.length !== 5 || !/^\d+$/.test(cols[4])) throw new Error(`ts-scan: malformed baseline row: ${line}`);
    map.set(cols.slice(0, 4).join("\t"), Number(cols[4]));
  }
  return map;
}

const baselineHeader = () =>
  "# ts-scan baseline (row 444): the sites that predate the rules, shrink-only. `scripts/ts-scan/scan.mjs --update-baseline`\n" +
  "# lowers or removes rows, never adds one; a new site or an edited baselined site is red. context = enclosing function\n" +
  "# path, hash = sha1 of the whitespace-normalised expression text (a line move changes nothing).\n" +
  "#! columns: file\trule\tcontext\thash\tcount\n";

function renderBaseline(map) {
  const rows = [...map.entries()].filter(([, n]) => n > 0).sort(([a], [b]) => (a < b ? -1 : 1));
  return baselineHeader() + rows.map(([k, n]) => `${k}\t${n}\n`).join("");
}

/** Compare the findings with a baseline map; returns { red: [finding|string], stale: [string] }. */
function compare(findings, base) {
  const red = [];
  const groups = new Map();
  for (const f of findings) {
    if (inBehaviours(f.file) || HARD_ONLY.has(f.rule)) { red.push(f); continue; }
    const k = keyOf(f);
    if (!groups.has(k)) groups.set(k, []);
    groups.get(k).push(f);
  }
  for (const [k, list] of groups) {
    const allowed = base.get(k) ?? 0;
    if (list.length > allowed) red.push(...list.slice(allowed));
  }
  const stale = [];
  for (const [k, n] of base) {
    const have = groups.get(k)?.length ?? 0;
    const file = k.split("\t")[0];
    if (inBehaviours(file)) stale.push(`${k.replaceAll("\t", " | ")}: a behaviours file has no baseline rows`);
    else if (have < n) stale.push(`${k.replaceAll("\t", " | ")}: baseline ${n}, tree ${have} (lower or remove the row: --update-baseline)`);
  }
  return { red, stale };
}

function fmt(f) {
  return `${f.file}:${f.line}:${f.col}: ${f.rule}: ${f.detail}\n    ${RULES[f.rule]} (${DOC})`;
}

// ------------------------------------------------------------------------------------------ self-test

function selfTest() {
  const errors = [];
  const files = [];
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith(".ts")) files.push(p);
    }
  };
  walk(FIXTURES);
  const found = scan(files);
  const covered = new Set();
  for (const f of files) {
    const m = /^\/\/ expect: (.+)$/m.exec(fs.readFileSync(f, "utf8"));
    if (!m) { errors.push(`${rel(f)}: no \`// expect:\` line`); continue; }
    const want = new Set(m[1].trim() === "none" ? [] : m[1].split(",").map((s) => s.trim()));
    const got = new Set(found.filter((x) => x.file === rel(f)).map((x) => x.rule));
    want.forEach((r) => covered.add(r));
    if ([...want].sort().join() !== [...got].sort().join()) {
      errors.push(`${rel(f)}: expected {${[...want].sort()}}, got {${[...got].sort()}}`);
    }
  }
  for (const r of Object.keys(RULES)) if (!covered.has(r)) errors.push(`rule ${r} has no red fixture`);

  // baseline mechanics: the same sites are green, a new site is red, a removed site leaves a stale row
  const sample = found.filter((x) => x.rule === "computed-access" && !inBehaviours(x.file));
  if (sample.length < 2) errors.push("baseline test needs two computed-access fixtures");
  else {
    const all = new Map();
    for (const f of sample) all.set(keyOf(f), (all.get(keyOf(f)) ?? 0) + 1);
    let r = compare(sample, all);
    if (r.red.length || r.stale.length) errors.push("baseline: the findings it was built from are not green");
    r = compare(sample, new Map([...all].slice(1)));
    if (r.red.length !== 1) errors.push("baseline: one new site must be exactly one red");
    r = compare(sample.slice(1), all);
    if (r.stale.length !== 1) errors.push("baseline: a fixed site must leave one stale row");
    const moved = sample.map((f) => ({ ...f, line: f.line + 40 }));
    r = compare(moved, all);
    if (r.red.length || r.stale.length) errors.push("baseline: a line move must change nothing");
    const beh = found.find((x) => inBehaviours(x.file));
    r = compare([beh], new Map([[keyOf(beh), 1]]));
    if (r.red.length !== 1 || r.stale.length !== 1) errors.push("baseline: a behaviours site must never be absorbed by a baseline row");
  }
  return errors;
}

// ------------------------------------------------------------------------------------------ main

function main(argv) {
  const known = new Set(["--self-test", "--update-baseline", "--init-baseline", "--list"]);
  const bad = argv.filter((a) => !known.has(a));
  if (bad.length || argv.length > 1) {
    console.error("usage: node scripts/ts-scan/scan.mjs [--self-test | --update-baseline | --init-baseline | --list]");
    return 2;
  }
  console.log(`ts-scan: typescript ${ts.version} (${TS_MODULE})`);
  const mode = argv[0];

  if (mode === "--self-test") {
    const errors = selfTest();
    if (errors.length) { console.error("ts-scan --self-test: RED\n  " + errors.join("\n  ")); return 1; }
    console.log("ts-scan --self-test: OK (every rule red on its fixture, clean fixtures green, baseline mechanics hold)");
    return 0;
  }

  const files = treeFiles();
  const findings = scan(files);
  if (mode === "--list") { findings.forEach((f) => console.log(fmt(f))); return 0; }

  if (mode === "--init-baseline") {
    if (fs.existsSync(BASELINE)) { console.error("ts-scan: the baseline exists; it only shrinks (--update-baseline)"); return 2; }
    const map = new Map();
    for (const f of findings) if (!inBehaviours(f.file) && !HARD_ONLY.has(f.rule)) map.set(keyOf(f), (map.get(keyOf(f)) ?? 0) + 1);
    fs.writeFileSync(BASELINE, renderBaseline(map));
    console.log(`ts-scan: wrote ${rel(BASELINE)} (${map.size} rows)`);
    return 0;
  }

  const base = parseBaseline(fs.readFileSync(BASELINE, "utf8"));
  const { red, stale } = compare(findings, base);

  if (mode === "--update-baseline") {
    if (red.length) { console.error("ts-scan: refusing to update: new sites (a baseline never grows)\n" + red.map(fmt).join("\n")); return 1; }
    const next = new Map();
    for (const [k, n] of base) {
      const have = findings.filter((f) => keyOf(f) === k).length;
      if (have > 0) next.set(k, Math.min(n, have));
    }
    fs.writeFileSync(BASELINE, renderBaseline(next));
    console.log(`ts-scan: baseline ${base.size} -> ${next.size} rows`);
    return 0;
  }

  if (red.length || stale.length) {
    for (const f of red) console.error(fmt(f));
    for (const s of stale) console.error(`stale baseline row: ${s}`);
    console.error(`ts-scan: RED (${red.length} new site(s), ${stale.length} stale baseline row(s))`);
    return 1;
  }
  const total = [...base.values()].reduce((a, b) => a + b, 0);
  console.log(`ts-scan: OK (${files.length} files, ${total} baselined site(s), shrink-only)`);
  return 0;
}

process.exit(main(process.argv.slice(2)));
