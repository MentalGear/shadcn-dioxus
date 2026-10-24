#!/usr/bin/env node
// Bucket a computed-style snapshot diff against a list of INTENDED differences, at ELEMENT level.
//
//   node scripts/snapshot-classify.mjs <after>.diff.json [--rules rules.json] [--elements] [--out report.json]
//                                      [--fail-unused]
//   node scripts/snapshot-classify.mjs --self-test
//
// Input: the machine-readable twin `<B>.diff.json` that `playwright/computed-style-snapshot.spec.ts -g compare`
// writes beside `<B>.diff.txt` (one object per differing value or added/removed record: mode, page, section,
// id, kind, tag, cls, pseudo, state, target, prop, a, b). A legacy `.diff.txt` is parsed best-effort.
//
// Output: every difference is either EXPLAINED by exactly one rule (first match wins) or UNEXPLAINED; the exit
// status is 1 when anything is unexplained, 2 on bad usage or a rejected rule. A rule that matched nothing is
// listed as UNUSED (a stale intention: `--fail-unused` makes it an error).
//
// WHY element-level (backlog row 246, dev-docs/archive/rounds/phase-c-plan-2026-10-13.md §4). Matching on class, page
// and property, with page-level buckets (`route:/`, avatar, attachment, button) and a cascade rule that tolerates
// shifts up to 40 px, lets a real regression of the same kind hide inside them. Here a bucket is one (mode, page, element, property) and a rule is only accepted if it names its target
// narrowly, by construction (`checkRule`):
//
//   * a rule must name an ELEMENT: `cls` (a class of the element) and/or `page`; or else an exact VALUE
//     TRANSITION (`prop` + `from` + `to`, e.g. the palette change `#000 -> #0a0a0a`, whatever element wears it);
//   * a pixel tolerance (`maxPx`: both values are `<n>px` and differ by at most that) additionally requires
//     `cls` AND `prop`, and is capped at MAX_PX_CAP, so "the cascade moves things <= 40 px" cannot be one rule
//     for the whole page; every cascade is written down per element class and property;
//   * `cls`, `page`, `prop`, `from`, `to`, `state`, `tag`, `section`, `mode` are anchored regular expressions
//     (`^(...)$`) over the whole field, never substring matches.
//
// Rule shape (JSON array of objects; unknown keys are rejected):
//   { "name": "menu-item-density", "cls": "dx-menu-item.*", "prop": "padding-(top|bottom)", "maxPx": 4,
//     "why": "row 8x12 -> 6x4", "mode": "light|dark", "page": "dropdown_menu", "section": "open",
//     "tag": "div", "state": "open:.*", "pseudo": "", "kind": "changed|added|removed", "from": "...", "to": "..." }
// `cls` is matched against EACH class of the element; `kind: added|removed` records have no `prop` (use `cls`).

import fs from "node:fs";
import process from "node:process";

const MAX_PX_CAP = 16;
const RULE_KEYS = new Set([
  "name", "why", "mode", "page", "section", "tag", "cls", "state", "pseudo", "kind", "prop", "from", "to", "maxPx",
]);

/** @typedef {{mode:string,page:string,section:string,id:string,kind:string,tag:string,cls:string,pseudo:string,state:string,target:string,prop?:string,a?:string,b?:string}} Diff */

const anchored = (src) => new RegExp(`^(?:${src})$`);

/** Throws with the reason when a rule is too broad or malformed. Returns the compiled rule. */
export function checkRule(r, i = 0) {
  const where = `rule #${i}${r && r.name ? ` "${r.name}"` : ""}`;
  if (!r || typeof r !== "object") throw new Error(`${where}: not an object`);
  if (!r.name || typeof r.name !== "string") throw new Error(`${where}: needs a "name"`);
  for (const k of Object.keys(r)) if (!RULE_KEYS.has(k)) throw new Error(`${where}: unknown key "${k}"`);
  const hasElement = !!(r.cls || r.page);
  const hasTransition = !!(r.prop && r.from && r.to);
  if (!hasElement && !hasTransition) {
    throw new Error(`${where}: too broad -- name an element ("cls" and/or "page") or an exact value transition ("prop" + "from" + "to")`);
  }
  if (r.maxPx !== undefined) {
    if (typeof r.maxPx !== "number" || !(r.maxPx >= 0)) throw new Error(`${where}: "maxPx" must be a number >= 0`);
    if (!r.cls || !r.prop) throw new Error(`${where}: "maxPx" needs "cls" AND "prop" (a tolerance is per element class and property)`);
    if (r.maxPx > MAX_PX_CAP) throw new Error(`${where}: "maxPx" ${r.maxPx} exceeds the cap of ${MAX_PX_CAP}px`);
    if (r.from || r.to) throw new Error(`${where}: "maxPx" and "from"/"to" are alternatives`);
  }
  const re = {};
  for (const k of ["mode", "page", "section", "tag", "cls", "state", "pseudo", "prop", "from", "to"]) {
    if (r[k] === undefined) continue;
    try {
      re[k] = anchored(r[k]);
    } catch (e) {
      throw new Error(`${where}: bad regular expression in "${k}": ${e.message}`);
    }
  }
  return { ...r, re, used: 0 };
}

const px = (v) => {
  const m = /^(-?\d+(?:\.\d+)?)px$/.exec(String(v ?? "").trim());
  return m ? Number(m[1]) : null;
};

/** Does compiled rule `r` explain diff `d`? Returns null or { pxDelta }. */
function matches(r, d) {
  const t = (k, v) => !r.re[k] || r.re[k].test(String(v ?? ""));
  if (r.kind && r.kind !== d.kind) return null;
  if (!t("mode", d.mode) || !t("page", d.page) || !t("section", d.section) || !t("tag", d.tag)) return null;
  if (!t("state", d.state) || !t("pseudo", d.pseudo)) return null;
  if (r.re.cls && !String(d.cls).split(/\s+/).filter(Boolean).some((c) => r.re.cls.test(c))) return null;
  if (d.kind !== "changed") {
    // an added/removed record carries no property: only an element rule (never a value transition) explains it
    return r.prop || r.from || r.to || r.maxPx !== undefined ? null : { pxDelta: 0 };
  }
  if (!t("prop", d.prop)) return null;
  if (r.re.from && !r.re.from.test(String(d.a))) return null;
  if (r.re.to && !r.re.to.test(String(d.b))) return null;
  let pxDelta = 0;
  if (r.maxPx !== undefined) {
    const x = px(d.a);
    const y = px(d.b);
    if (x === null || y === null) return null;
    pxDelta = Math.abs(y - x);
    if (pxDelta > r.maxPx + 1e-6) return null;
  }
  return { pxDelta };
}

/** Classify `diffs` against compiled `rules`. Pure; used by the CLI and the self-test. */
export function classify(diffs, rules) {
  const explained = [];
  const unexplained = [];
  for (const d of diffs) {
    let hit = null;
    for (const r of rules) {
      const m = matches(r, d);
      if (m) {
        hit = { rule: r, ...m };
        break;
      }
    }
    if (hit) {
      hit.rule.used++;
      explained.push({ d, rule: hit.rule.name, pxDelta: hit.pxDelta });
    } else {
      unexplained.push(d);
    }
  }
  return { explained, unexplained };
}

const elementKey = (d) => `${d.mode} ${d.page} ${d.section} ${d.id}`;

/** Best-effort parser of the legacy text report. */
function parseTxt(text) {
  const out = [];
  const val = '("(?:[^"\\\\]|\\\\.)*")';
  const rowRe = new RegExp(`^(light|dark) (\\S+) (base|pseudo|focus|focusK|hover|open) (.+?)#(\\d+) ([-a-z]+): ${val} -> ${val}$`);
  const onlyRe = /^(light|dark) (\S+) (base|pseudo|focus|focusK|hover|open) (.+?)#(\d+): only in ([AB])$/;
  const split = (id) => {
    let state = "";
    let rest = id;
    const sm = /:((?:open|hover)[:\d/a-z]*)$/.exec(rest);
    if (sm) {
      state = sm[1];
      rest = rest.slice(0, sm.index);
    }
    let pseudo = "";
    const pm = /(::[a-z-]+)$/.exec(rest);
    if (pm) {
      pseudo = pm[1];
      rest = rest.slice(0, pm.index);
    }
    const dot = rest.indexOf(".");
    return { tag: dot < 0 ? rest : rest.slice(0, dot), cls: dot < 0 ? "" : rest.slice(dot + 1), pseudo, state };
  };
  for (const line of text.split("\n")) {
    let m = rowRe.exec(line);
    if (m) {
      out.push({ mode: m[1], page: m[2], section: m[3], id: `${m[4]}#${m[5]}`, kind: "changed", ...split(m[4]), target: "", prop: m[6], a: JSON.parse(m[7]), b: JSON.parse(m[8]) });
      continue;
    }
    m = onlyRe.exec(line);
    if (m) out.push({ mode: m[1], page: m[2], section: m[3], id: `${m[4]}#${m[5]}`, kind: m[6] === "A" ? "removed" : "added", ...split(m[4]), target: "" });
  }
  return out;
}

function report(diffs, rules, opts) {
  const { explained, unexplained } = classify(diffs, rules);
  const lines = [];
  lines.push(`${diffs.length} difference(s): ${explained.length} explained, ${unexplained.length} UNEXPLAINED`);
  if (rules.length) {
    lines.push("", "rule buckets (differences / distinct elements / max px delta):");
    for (const r of rules) {
      const hits = explained.filter((e) => e.rule === r.name);
      const els = new Set(hits.map((e) => elementKey(e.d)));
      const maxPx = hits.reduce((m, e) => Math.max(m, e.pxDelta), 0);
      lines.push(`  ${r.used ? "  " : "!!"} ${r.name}: ${hits.length} / ${els.size}${r.maxPx !== undefined ? ` / ${maxPx}px (limit ${r.maxPx})` : ""}${r.used ? "" : "  UNUSED"}`);
    }
  }
  if (opts.elements) {
    const byEl = new Map();
    for (const e of explained) {
      const k = `${elementKey(e.d)} [${e.rule}]`;
      byEl.set(k, (byEl.get(k) ?? 0) + 1);
    }
    lines.push("", "explained elements:");
    for (const [k, n] of [...byEl].sort()) lines.push(`  ${k}: ${n}`);
  }
  if (unexplained.length) {
    const byEl = new Map();
    for (const d of unexplained) {
      const k = elementKey(d);
      if (!byEl.has(k)) byEl.set(k, []);
      byEl.get(k).push(d);
    }
    lines.push("", `UNEXPLAINED, by element (${byEl.size} element(s)):`);
    for (const [k, ds] of [...byEl].sort(([x], [y]) => (x < y ? -1 : 1))) {
      const d0 = ds[0];
      lines.push(`  ${k}${d0.target ? `  (${d0.target})` : ""}`);
      for (const d of ds) lines.push(d.kind === "changed" ? `      ${d.prop}: ${JSON.stringify(d.a)} -> ${JSON.stringify(d.b)}` : `      record ${d.kind}`);
    }
  }
  return { text: lines.join("\n"), explained, unexplained };
}

// ---------------------------------------------------------------------------------------------------------------
function selfTest() {
  let failed = 0;
  const ok = (cond, what) => {
    console.log(`${cond ? "ok  " : "FAIL"} ${what}`);
    if (!cond) failed++;
  };
  const diff = (o) => ({ mode: "light", page: "card", section: "base", id: "div.dx-card#1", kind: "changed", tag: "div", cls: "dx-card", pseudo: "", state: "", target: "", prop: "height", a: "100px", b: "101px", ...o });
  const rejects = (r) => {
    try {
      checkRule(r);
      return false;
    } catch {
      return true;
    }
  };
  // construction: broad rules do not compile
  ok(rejects({ name: "all" }), "a rule naming nothing is rejected");
  ok(rejects({ name: "prop-only", prop: "height" }), "a property alone is rejected");
  ok(rejects({ name: "px-no-cls", page: "card", prop: "height", maxPx: 2 }), "a pixel tolerance without cls is rejected");
  ok(rejects({ name: "px-no-prop", cls: "dx-card", maxPx: 2 }), "a pixel tolerance without prop is rejected");
  ok(rejects({ name: "px-40", cls: "dx-card", prop: "height", maxPx: 40 }), "a 40px tolerance is over the cap");
  ok(rejects({ name: "typo", cls: "dx-card", propp: "x" }), "an unknown key is rejected");
  ok(!rejects({ name: "palette", prop: "background-color", from: "rgb\\(0, 0, 0\\)", to: "rgb\\(10, 10, 10\\)" }), "an exact value transition is accepted");
  // behaviour: a 1px shift is caught unless a rule names that element class AND property
  const cascade = [checkRule({ name: "card-grows", cls: "dx-card", prop: "height", maxPx: 12 })];
  let r = classify([diff({})], cascade);
  ok(r.explained.length === 1 && r.unexplained.length === 0, "a 1px height change on the named element class is explained");
  r = classify([diff({ id: "button.dx-button#1", tag: "button", cls: "dx-button" })], cascade);
  ok(r.unexplained.length === 1, "the SAME 1px change on another element class is NOT hidden by that rule");
  r = classify([diff({ prop: "top" })], cascade);
  ok(r.unexplained.length === 1, "a 1px change of another property on the same element is NOT hidden");
  r = classify([diff({ b: "130px" })], cascade);
  ok(r.unexplained.length === 1, "a 30px change beyond the rule's own limit is NOT hidden");
  r = classify([diff({ a: "0px", b: "1px", prop: "height" }), diff({ kind: "removed", prop: undefined, a: undefined, b: undefined })], cascade);
  ok(r.unexplained.length === 1 && r.unexplained[0].kind === "removed", "a removed record is not explained by a value rule");
  const tolerant0 = [checkRule({ name: "exact", cls: "dx-card", prop: "height", maxPx: 0 })];
  ok(classify([diff({})], tolerant0).unexplained.length === 1, "maxPx 0 explains no shift");
  const t = parseTxt('light card base div.dx-card#1 height: "100px" -> "101px"\ndark card hover x.dx-y:hover:3#1: only in A\nlight card open div.dx-a.dx-b::before:open:0#2 color: "red" -> "blue"');
  ok(t.length === 3 && t[0].prop === "height" && t[1].kind === "removed" && t[2].pseudo === "::before", "legacy text parser reads rows");
  if (failed) {
    console.error(`${failed} self-test failure(s)`);
    process.exit(1);
  }
  console.log("snapshot-classify self-test: OK");
}

function main() {
  const argv = process.argv.slice(2);
  if (argv.includes("--self-test")) return selfTest();
  const opt = { rules: null, elements: false, out: null, failUnused: false };
  const files = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--rules") opt.rules = argv[++i];
    else if (a === "--elements") opt.elements = true;
    else if (a === "--out") opt.out = argv[++i];
    else if (a === "--fail-unused") opt.failUnused = true;
    else if (a.startsWith("--")) {
      console.error(`unknown option ${a}`);
      process.exit(2);
    } else files.push(a);
  }
  if (files.length !== 1) {
    console.error("usage: snapshot-classify.mjs <diff.json|diff.txt> [--rules rules.json] [--elements] [--out report.json] [--fail-unused] | --self-test");
    process.exit(2);
  }
  const raw = fs.readFileSync(files[0], "utf8");
  const diffs = files[0].endsWith(".txt") ? parseTxt(raw) : JSON.parse(raw);
  let rules = [];
  if (opt.rules) {
    try {
      rules = JSON.parse(fs.readFileSync(opt.rules, "utf8")).map((r, i) => checkRule(r, i));
    } catch (e) {
      console.error(`rules rejected: ${e.message}`);
      process.exit(2);
    }
  }
  const rep = report(diffs, rules, opt);
  console.log(rep.text);
  if (opt.out) fs.writeFileSync(opt.out, JSON.stringify({ explained: rep.explained, unexplained: rep.unexplained }, null, 1));
  const unused = rules.filter((r) => !r.used);
  if (rep.unexplained.length || (opt.failUnused && unused.length)) process.exit(1);
}

export { parseTxt };
if (import.meta.url === `file://${process.argv[1]}`) main();
