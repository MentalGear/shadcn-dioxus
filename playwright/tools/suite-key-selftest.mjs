// Self-test of the per-file suite reuse (backlog row 513, tools/suite-key.mjs). Node and git only: no browser, no build, no
// suite run. Planted fixture repos stand in for trees; fake run directories (the JSON reports run-suite.sh writes) stand in for
// runs, and go through the real record and verdict code.
//
//   node playwright/tools/suite-key-selftest.mjs          (scripts/check-suite-reuse.sh runs exactly this)
//   SUITE_KEY_SELFTEST_FIXTURES_ONLY=1 skips part 5 (the real tree).
//
// 1. What a change re-runs: a spec-only change runs that file; a shared helper runs its importers; a site change, a config,
//    fixture, lockfile or imported-by-config change runs everything; a file outside the build (dev-docs, docs, row files,
//    tools) runs nothing; an include of dev-docs/motion.md is a site input.
// 2. Run-time reads: a registered reader's data re-runs the specs that load the reader, and only them; an environment variable
//    a spec reads re-runs it; the timing pass switch re-runs everything.
// 3. What is accepted: a red, partial, other-build or hand-edited record is not; a chunk with run errors writes no record; a spec
//    file added without a record blocks the tree; an unkeyable spec always runs.
// 4. The verdict: whole-suite totals (reused plus run) with the reused count stated; red when a file failed, a chunk is missing,
//    or a reused record is gone.
// 5. The real tree: every spec has a key, the registered readers are the only files naming their data, and the build inputs name
//    no excluded path except the reviewed ones.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { openTree, siteKey, specKeys, plan, record, verdict, hashDir, lookup, rustIncludes, specFilesOf, DATA_READERS, KeyError } from "./suite-key.mjs";
import { specFilesOnDisk } from "./spec-files.mjs";
import { stripComments } from "./import-graph.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "..", "..");
let failed = 0, passed = 0;
const check = (name, ok, detail = "") => { if (ok) passed++; else { failed++; console.error(`FAIL ${name}${detail ? `\n     ${detail}` : ""}`); } };
const eq = (name, got, want) => check(name, JSON.stringify(got) === JSON.stringify(want), `got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "suite-key-"));
process.on("exit", () => fs.rmSync(tmp, { recursive: true, force: true }));
const git = (cwd, ...a) => execFileSync("git", ["-C", cwd, ...a], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();

// ---- a fixture repo with the shapes that matter -----------------------------------------------------------------
const repo = path.join(tmp, "repo");
const base = {
  "Cargo.toml": "[workspace]\n", "Cargo.lock": "# lock v1\n",
  "preview/src/main.rs": "pub const A: &str = \"a\";\n",
  "preview/src/motion_docs.rs": "// the table\nconst MOTION_MD: &str = include_str!(\"../../dev-docs/motion.md\");\n",
  "preview/src/generated.rs": "include!(concat!(env!(\"OUT_DIR\"), \"/x.rs\"));\n",
  "scripts/build-ssg.sh": "#!/bin/sh\necho build\n",
  "dev-docs/motion.md": "# motion\n", "dev-docs/STATUS.md": "# status\n", "dev-docs/backlog/001-a.md": "# row\n",
  "docs/index.html": "<html></html>\n",
  "primitives/ts-test/witness.ts": "export const w = 1;\n",
  "playwright/package.json": "{}\n", "playwright/pnpm-lock.yaml": "lock: 1\n", "playwright/durations.json": "{}\n",
  "playwright/sandbox-chromium.cjs": "function sandboxChromium(env = process.env) { return undefined; }\nexports.sandboxChromium = sandboxChromium;\n",
  "playwright/base-url.ts": "export const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? \"x\";\nexport const P = process.env.PW_BUILD_PROFILE;\n",
  "playwright/suite.local.config.ts": "import { P } from \"./base-url\";\nimport { sandboxChromium } from \"./sandbox-chromium.cjs\";\nexport default { P, sandboxChromium };\n",
  "playwright/fixtures.ts": "export const test = 1;\n",
  "playwright/helper.ts": "export const h = 1;\n",
  "playwright/deep.ts": "import { h } from \"./helper\";\nexport const d = h;\n",
  "playwright/a.spec.ts": "import { test } from \"./fixtures\";\nimport { h } from \"./helper\";\nimport { BASE_URL } from \"./base-url\";\n",
  "playwright/b.spec.ts": "import { test } from \"./fixtures\";\nimport { d } from \"./deep\";\n",
  "playwright/c.spec.ts": "import { test } from \"./fixtures\";\nconst r = process.env.FIRST_PAINT_ROUTES;\n",
  "playwright/d.spec.ts": "import { test } from \"./fixtures\";\nimport { w } from \"../primitives/ts-test/witness\";\n",
  "playwright/witness/binding.ts": "import { readFileSync } from \"node:fs\";\nexport const binding = (s: string) => readFileSync(`${__dirname}/bindings/${s}.json`);\n",
  "playwright/witness/bindings/x.json": "{\"files\": []}\n",
  "playwright/witness/w.spec.ts": "import { test } from \"./../fixtures\";\nimport { binding } from \"./binding\";\n",
  "playwright/oracle/hydration-parity.spec.ts": "import { test } from \"../fixtures\";\n// runs the attr-synth binary\nconst bin = \"attr-synth\";\n",
  "playwright/oracle/attr-synth/src/main.rs": "fn main() {}\n",
  "playwright/tools/run-suite.sh": "#!/bin/sh\n",
};
const SPECS = ["a.spec.ts", "b.spec.ts", "c.spec.ts", "d.spec.ts", "oracle/hydration-parity.spec.ts", "witness/w.spec.ts"];
const put = (cwd, files) => { for (const [f, c] of Object.entries(files)) { const p = path.join(cwd, f); if (c === null) fs.rmSync(p, { force: true }); else { fs.mkdirSync(path.dirname(p), { recursive: true }); fs.writeFileSync(p, c); } } };
fs.mkdirSync(repo, { recursive: true });
git(repo, "init", "-q", ".");
git(repo, "config", "user.email", "t@t"); git(repo, "config", "user.name", "t");
put(repo, base);
git(repo, "add", "-A"); git(repo, "commit", "-q", "-m", "base");
const BASE = git(repo, "rev-parse", "HEAD");
// a commit on top of the base with `edits` applied (null deletes); returns the commit
const variant = (edits) => {
  git(repo, "checkout", "-q", "-B", "case", BASE);
  put(repo, edits);
  git(repo, "add", "-A"); git(repo, "commit", "-q", "--allow-empty", "-m", "case");
  return git(repo, "rev-parse", "HEAD");
};

const ENV = { TIMING_PASS: "off", PW_BUILD_PROFILE: "release", FIRST_PAINT_ROUTES: "all" };
const RT = "rt-1";
const pub = path.join(tmp, "public"); fs.mkdirSync(pub); fs.writeFileSync(path.join(pub, "index.html"), "site v1");
const popts = (extra = {}) => ({ env: ENV, runtime: RT, buildEnv: "dx 0.7.9", publicDir: pub, config: "suite.local.config.ts", ...extra });
const treeOf = (c) => openTree(repo, c);

// a run directory as run-suite.sh leaves it: the plan, two chunks of the files that ran, their JSON reports
const test3 = (n, status = "expected") => Array.from({ length: n }, (_, i) => ({ title: `t${i}`, status, results: [] }));
function fakeRun(out, pl, over = {}, { chunks = 2, errors = {}, drop = [] } = {}) {
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(path.join(out, "plan.json"), JSON.stringify(pl));
  const halves = Array.from({ length: chunks }, (_, i) => pl.run.filter((_f, k) => k % chunks === i));
  halves.forEach((files, i) => {
    fs.writeFileSync(path.join(out, `specs-${i + 1}.txt`), files.join("\n") + (files.length ? "\n" : "\n"));
    if (!files.length || drop.includes(i + 1)) return;
    const suites = files.map((f) => ({ title: f, file: f, specs: [{ title: "case", file: f, tests: over[f] ?? test3(3) }] }));
    const count = (st) => suites.flatMap((s) => s.specs[0].tests).filter((t) => t.status === st).length;
    fs.writeFileSync(path.join(out, `chunk-${i + 1}.json`), JSON.stringify({ stats: { expected: count("expected"), unexpected: count("unexpected"), flaky: count("flaky"), skipped: count("skipped") }, errors: errors[i + 1] ?? [], suites }));
  });
}
const newStore = () => fs.mkdtempSync(path.join(tmp, "store-"));
let runN = 0;
// plan a tree, "run" what the plan says (all green unless `over` says otherwise), record, and return the plan
function runAndRecord(store, commit, over = {}, opts = {}) {
  const pl = plan(treeOf(commit), store, popts(opts.plan));
  const out = path.join(tmp, `run-${++runN}`);
  fakeRun(out, pl, over, opts.run);
  const rec = record(store, out, opts.run?.chunks ?? 2);
  return { pl, out, rec };
}
const runOf = (commit, store, opts) => plan(treeOf(commit), store, popts(opts)).run;

// ---- 1. what a change re-runs ------------------------------------------------------------------------------------
{
  const store = newStore();
  const first = runAndRecord(store, BASE);
  eq("a first run executes every spec file", first.pl.run, SPECS);
  eq("a first run writes a green record for every file", [first.rec.green, first.rec.red, first.rec.unrecorded.length], [6, 0, 0]);
  eq("the same tree needs no run", runOf(BASE, store), []);
  check("the same tree is accepted whole", plan(treeOf(BASE), store, popts()).allGreen);

  eq("a spec-only change re-runs that file", runOf(variant({ "playwright/c.spec.ts": base["playwright/c.spec.ts"] + "// edit\nconst x = 1;\n" }), store), ["c.spec.ts"]);
  eq("a shared helper re-runs every importer, direct and through another helper", runOf(variant({ "playwright/helper.ts": "export const h = 2;\n" }), store), ["a.spec.ts", "b.spec.ts"]);
  eq("a helper no spec imports re-runs nothing", runOf(variant({ "playwright/unused.ts": "export const u = 1;\n" }), store), []);
  eq("a file outside playwright/ that a spec imports re-runs that spec and nothing else of the key", runOf(variant({ "primitives/ts-test/witness.ts": "export const w = 2;\n" }), store), ["d.spec.ts"]);
  for (const [name, edits] of [
    ["fixtures.ts", { "playwright/fixtures.ts": "export const test = 2;\n" }],
    ["the config", { "playwright/suite.local.config.ts": base["playwright/suite.local.config.ts"] + "// c\nconst x = 1;\n" }],
    ["a file the config imports", { "playwright/base-url.ts": base["playwright/base-url.ts"] + "export const Q = 1;\n" }],
    ["the browser launcher the config requires", { "playwright/sandbox-chromium.cjs": base["playwright/sandbox-chromium.cjs"] + "// c\n" }],
    ["pnpm-lock.yaml", { "playwright/pnpm-lock.yaml": "lock: 2\n" }],
    ["package.json", { "playwright/package.json": "{\"a\":1}\n" }],
  ]) eq(`a change to ${name} re-runs every spec file`, runOf(variant(edits), store), SPECS);
  eq("a site change re-runs every spec file", runOf(variant({ "preview/src/main.rs": "pub const A: &str = \"b\";\n" }), store), SPECS);
  eq("a lock file change re-runs every spec file", runOf(variant({ "Cargo.lock": "# lock v2\n" }), store), SPECS);
  eq("a build script change re-runs every spec file", runOf(variant({ "scripts/build-ssg.sh": "#!/bin/sh\necho other\n" }), store), SPECS);
  eq("a file the build includes from dev-docs/ re-runs every spec file", runOf(variant({ "dev-docs/motion.md": "# motion v2\n" }), store), SPECS);
  eq("dev-docs/ prose the build does not include re-runs nothing", runOf(variant({ "dev-docs/STATUS.md": "# status 2\n" }), store), []);
  eq("a backlog row re-runs nothing", runOf(variant({ "dev-docs/backlog/002-b.md": "# row 2\n", "dev-docs/backlog/001-a.md": "# changed\n" }), store), []);
  eq("the deploy's own output in docs/ re-runs nothing", runOf(variant({ "docs/index.html": "<html>2</html>\n", "docs/new.js": "x" }), store), []);
  eq("the suite tooling (tools/, durations.json) re-runs nothing", runOf(variant({ "playwright/tools/run-suite.sh": "#!/bin/sh\n# x\n", "playwright/durations.json": "{\"a\":1}\n" }), store), []);
  eq("a build environment change re-runs every spec file", runOf(variant({}), store, { buildEnv: "dx 0.7.10" }), SPECS);
  eq("a different browser or runtime re-runs every spec file", runOf(variant({}), store, { runtime: "rt-2" }), SPECS);
  eq("a spec file renamed is a new file with no record", runOf(variant({ "playwright/c.spec.ts": null, "playwright/c2.spec.ts": base["playwright/c.spec.ts"] }), store), ["c2.spec.ts"]);

  // ---- 2. run-time reads ---------------------------------------------------------------------------------------
  eq("a binding read through the registered reader re-runs the specs that load the reader", runOf(variant({ "playwright/witness/bindings/x.json": "{\"files\": [\"a\"]}\n" }), store), ["witness/w.spec.ts"]);
  eq("a new binding re-runs the specs that load the reader", runOf(variant({ "playwright/witness/bindings/y.json": "{}\n" }), store), ["witness/w.spec.ts"]);
  eq("the data of a registered spec reader re-runs that spec", runOf(variant({ "playwright/oracle/attr-synth/src/main.rs": "fn main() { 1; }\n" }), store), ["oracle/hydration-parity.spec.ts"]);
  eq("data no reader is registered for re-runs every spec file", runOf(variant({ "playwright/fixtures/unknown.json": "{}\n" }), store), SPECS);
  eq("an environment variable a spec reads re-runs the specs that read it", runOf(variant({}), store, { env: { ...ENV, FIRST_PAINT_ROUTES: "some" } }), ["c.spec.ts"]);
  eq("an environment variable no spec reads re-runs nothing", runOf(variant({}), store, { env: { ...ENV, SOME_OTHER: "1" } }), []);
  eq("the port and the site path are not part of the key", runOf(variant({}), store, { env: { ...ENV, PLAYWRIGHT_BASE_URL: "http://127.0.0.1:1", SSG_SITE_DIR: "/x" } }), []);
  eq("a record written with the timing pass off is not used when it is on", runOf(variant({}), store, { env: { ...ENV, TIMING_PASS: "on" } }), SPECS);
  eq("the build profile is part of the key", runOf(variant({}), store, { env: { ...ENV, PW_BUILD_PROFILE: "debug" } }), SPECS);

  // ---- the built site ------------------------------------------------------------------------------------------
  const pub2 = path.join(tmp, "public2"); fs.mkdirSync(pub2); fs.writeFileSync(path.join(pub2, "index.html"), "site v2");
  eq("records of another build of the same site key are not used", runOf(BASE, store, { publicDir: pub2 }), SPECS);
  check("hashing a directory names its bytes", hashDir(pub) !== hashDir(pub2) && hashDir(pub) === hashDir(pub));
}

// ---- 3. what is accepted ----------------------------------------------------------------------------------------
{
  // a red file: its record is red, the file runs again, the tree is not accepted
  const store = newStore();
  const r1 = runAndRecord(store, BASE, { "b.spec.ts": [...test3(2), { title: "bad", status: "unexpected", results: [] }] });
  eq("a red file gets a red record, the others green", [r1.rec.green, r1.rec.red], [5, 1]);
  eq("a red record is not accepted: the red file runs again", runOf(BASE, store), ["b.spec.ts"]);
  check("a tree with a red file is not accepted", !plan(treeOf(BASE), store, popts()).allGreen);
  const fixed = variant({ "playwright/b.spec.ts": base["playwright/b.spec.ts"] + "// fix\n" });
  eq("after a spec-only fix only the red file runs again", runOf(fixed, store), ["b.spec.ts"]);
  const r2 = runAndRecord(store, fixed);
  eq("the fix run executes that one file and records it", [r2.pl.run, r2.rec.green], [["b.spec.ts"], 1]);
  check("the fixed tree is accepted whole, with no build and no run", plan(treeOf(fixed), store, { ...popts(), publicDir: undefined }).allGreen);

  // a record that says otherwise on its face is refused even under the right key
  const keys = specKeys(treeOf(BASE), { runtime: RT, env: ENV });
  const site = siteKey(treeOf(BASE), { buildEnv: "dx 0.7.9" }).key;
  const good = JSON.parse(fs.readFileSync(path.join(store, site, fs.readdirSync(path.join(store, site)).find((f) => f.startsWith("a.spec.ts@"))), "utf8"));
  const accepts = (mut) => {
    const s = newStore(); const rec = { ...JSON.parse(JSON.stringify(good)), ...mut };
    const f = path.join(s, site, `${encodeURIComponent("a.spec.ts")}@${keys.get("a.spec.ts").key.slice(0, 16)}.json`);
    fs.mkdirSync(path.dirname(f), { recursive: true }); fs.writeFileSync(f, JSON.stringify(rec));
    return !!lookup(s, site, "a.spec.ts", keys.get("a.spec.ts").key, { publicSha: hashDir(pub), timing: "off" }).record;
  };
  check("control: the planted good record is accepted", accepts({}));
  check("a record that says red is refused", !accepts({ status: "red" }));
  check("a record that says partial is refused", !accepts({ partial: true }));
  check("a green record with an unexpected test is refused", !accepts({ counts: { ...good.counts, unexpected: 1 } }));
  check("a record with no counts is refused", !accepts({ counts: undefined }));
  check("a record for another spec key is refused", !accepts({ specKey: "0".repeat(64) }));
  check("a record for another site key is refused", !accepts({ siteKey: "0".repeat(64) }));
  check("a record for another file is refused", !accepts({ file: "b.spec.ts" }));
  check("a record with no built-site hash is refused", !accepts({ publicSha: "" }));
  check("a record of another format is refused", !accepts({ format: 0 }));
  check("a record that ran with another timing switch is refused", !accepts({ timingPass: "on" }));

  // a run whose chunk did not finish records nothing for its files
  const s2 = newStore();
  const pl2 = plan(treeOf(BASE), s2, popts());
  const out2 = path.join(tmp, "run-err");
  fakeRun(out2, pl2, {}, { errors: { 2: ["Timed out waiting 900s for the test suite to run"] } });
  const rec2 = record(s2, out2, 2);
  eq("a chunk that timed out records none of its files", [rec2.green, rec2.unrecorded.length], [3, 3]);
  eq("the files of the chunk that timed out run again", runOf(BASE, s2).length, 3);
  const s3 = newStore();
  const out3 = path.join(tmp, "run-missing");
  fakeRun(out3, plan(treeOf(BASE), s3, popts()), {}, { drop: [1] });
  const rec3 = record(s3, out3, 2);
  eq("a chunk with no report records none of its files", [rec3.green, rec3.unrecorded.length], [3, 3]);
  eq("the files of a chunk that never reported run again", runOf(BASE, s3).length, 3);

  // an added spec file with no record blocks the tree, through the CLI the deploy uses
  const store4 = newStore();
  runAndRecord(store4, BASE);
  const withNew = variant({ "playwright/e.spec.ts": "import { test } from \"./fixtures\";\n" });
  const cli = (commit, store, ...extra) => spawnSync("node", [path.join(HERE, "suite-key.mjs"), "accept", "--repo", repo, "--tree", commit, "--store", store, "--build-env", "dx 0.7.9", "--runtime", RT, ...extra],
    { encoding: "utf8", env: { ...process.env, ...ENV } });
  eq("control: the CLI accepts the recorded tree (exit 0)", cli(BASE, store4).status, 0);
  const blocked = cli(withNew, store4);
  eq("a spec file added with no record blocks the deploy (exit 1)", blocked.status, 1);
  check("the blocked tree runs exactly the new file", /1 to run/.test(blocked.stdout), blocked.stdout);

  // an unkeyable spec always runs and never accepts
  const unk = variant({ "playwright/f.spec.ts": "import { x } from \"./nowhere\";\n" });
  const keysU = specKeys(treeOf(unk), { runtime: RT, env: ENV });
  check("a spec importing a missing file has no key", keysU.get("f.spec.ts").key === null && /nowhere/.test(keysU.get("f.spec.ts").why));
  const dyn = variant({ "playwright/g.spec.ts": "const m = await import(`./${name}`);\n" });
  check("a spec with a computed import has no key", specKeys(treeOf(dyn), { runtime: RT, env: ENV }).get("g.spec.ts").key === null);
  const bare = variant({ "playwright/h.spec.ts": "const e = { ...process.env };\n" });
  check("a spec using process.env as a whole has no key", specKeys(treeOf(bare), { runtime: RT, env: ENV }).get("h.spec.ts").key === null);
  const r5 = runAndRecord(store4, unk);
  check("an unkeyable file runs on every plan and is never recorded", r5.pl.run.includes("f.spec.ts") && r5.rec.unrecorded.some((u) => u.startsWith("f.spec.ts")) && runOf(unk, store4).includes("f.spec.ts"));
  check("a tree with an unkeyable file is not accepted by records", !plan(treeOf(unk), store4, popts()).allGreen);
}

// ---- site key: includes ----------------------------------------------------------------------------------------
{
  const inc = rustIncludes(treeOf(BASE));
  eq("the include scan finds the excluded file the build reads", [...inc.targets], ["dev-docs/motion.md"]);
  eq("an include of a build script's output is generated, not a path", inc.generated, 1);
  const sk = siteKey(treeOf(BASE), { buildEnv: "e" });
  eq("the key names the pulled-back file", sk.pulledBack, ["dev-docs/motion.md"]);
  const bad = variant({ "preview/src/x.rs": "const A: &str = include_str!(some_path);\n" });
  check("an include this scan cannot read stops the key", (() => { try { siteKey(treeOf(bad), {}); return false; } catch (e) { return e instanceof KeyError; } })());
  const bad2 = variant({ "preview/src/x.rs": "const A: &str = include_str!(concat!(\"../../dev-docs/\", stringify!(x)));\n" });
  check("a computed include into an excluded directory stops the key", (() => { try { siteKey(treeOf(bad2), {}); return false; } catch (e) { return e instanceof KeyError; } })());
  const ok2 = variant({ "preview/src/x.rs": "const A: &str = include_str!(concat!(\"components/\", stringify!(x), \"/mod.rs\"));\n" });
  check("a computed include under an included directory is covered by the key", (() => { try { siteKey(treeOf(ok2), {}); return true; } catch { return false; } })());
  const cmt = variant({ "preview/src/x.rs": "// include_str!(oops)\n/// also include_bytes!(\"../../dev-docs/gone.md\")\nfn f() {}\n" });
  check("an include named in a comment is not an include", (() => { try { return siteKey(treeOf(cmt), {}).pulledBack.length === 1; } catch { return false; } })());
  const miss = variant({ "preview/src/m.rs": "const A: &str = include_str!(\"../../dev-docs/gone.md\");\n" });
  const present = variant({ "preview/src/m.rs": "const A: &str = include_str!(\"../../dev-docs/gone.md\");\n", "dev-docs/gone.md": "x\n" });
  check("an include of an absent file differs from one that exists", siteKey(treeOf(miss), {}).key !== siteKey(treeOf(present), {}).key);
}

// ---- 4. the verdict ---------------------------------------------------------------------------------------------
{
  const store = newStore();
  runAndRecord(store, BASE);
  const next = variant({ "playwright/c.spec.ts": base["playwright/c.spec.ts"] + "// e\nconst x = 1;\n" });
  const pl = plan(treeOf(next), store, popts());
  const out = path.join(tmp, "run-verdict");
  fakeRun(out, pl, { "c.spec.ts": [...test3(4), ...test3(1, "skipped"), ...test3(1, "flaky")] });
  const v = verdict(out, 2, { repo, store, tree: next, env: { TIMING_PASS: "off", TIMING_REENABLE_ROW: "494" } });
  check("a reuse run is green when the file that ran and the reused records are", v.ok, JSON.stringify(v));
  check("the totals are the whole suite: reused (15) plus run (4)", /^passed=19 failed=0 flaky=1 skipped=1 /.test(v.line), v.line);
  check("the reused files are stated separately", /reused_files=5 reused_passed=15 run_files=1/.test(v.line), v.line);
  check("the flaky pass is listed", v.flaky.some((t) => t.startsWith("c.spec.ts")));
  record(store, out, 2);
  check("the fixed tree is accepted after the record step", plan(treeOf(next), store, popts()).allGreen);

  const outRed = path.join(tmp, "run-verdict-red");
  fakeRun(outRed, pl, { "c.spec.ts": [...test3(2), { title: "x", status: "unexpected", results: [] }] });
  const vr = verdict(outRed, 2, { repo, store, tree: next, env: ENV });
  check("a failed file makes the whole verdict red and names it", !vr.ok && vr.failed.some((t) => t.startsWith("c.spec.ts")) && /failed=1/.test(vr.line));
  const outMissing = path.join(tmp, "run-verdict-missing");
  fakeRun(outMissing, pl, {}, { drop: [1, 2] });
  check("a missing chunk report makes the verdict red", !verdict(outMissing, 2, { repo, store, tree: next, env: ENV }).ok);
  const outErr = path.join(tmp, "run-verdict-err");
  fakeRun(outErr, pl, {}, { errors: { 1: ["Timed out"], 2: ["Timed out"] } });
  const ve = verdict(outErr, 2, { repo, store, tree: next, env: ENV });
  check("a chunk with run errors counts as missing", !ve.ok && /chunks_missing=1/.test(ve.line), ve.line);
  // a reused record that disappeared during the run
  const outGone = path.join(tmp, "run-verdict-gone");
  const store2 = newStore();
  runAndRecord(store2, BASE);
  const pl2 = plan(treeOf(next), store2, popts());
  fakeRun(outGone, pl2, {});
  const site = pl2.siteKey;
  fs.rmSync(path.join(store2, site, fs.readdirSync(path.join(store2, site)).find((f) => f.startsWith("a.spec.ts@"))));
  const vg = verdict(outGone, 2, { repo, store: store2, tree: next, env: ENV });
  check("a reused record that is gone makes the verdict red", !vg.ok && vg.reasons.some((r) => r.startsWith("reused a.spec.ts")), JSON.stringify(vg.reasons));
  // a plan that does not cover the tree's files
  const outPartial = path.join(tmp, "run-verdict-partial");
  fakeRun(outPartial, { ...pl2, run: pl2.run.filter((f) => f !== "c.spec.ts") }, {});
  check("a plan that leaves a spec file of the tree out makes the verdict red", !verdict(outPartial, 2, { repo, store: store2, tree: next, env: ENV }).ok);
  // no reuse: the verdict is the plain whole-suite one
  const outPlain = path.join(tmp, "run-plain");
  fs.mkdirSync(outPlain);
  fs.writeFileSync(path.join(outPlain, "specs-1.txt"), "a.spec.ts\n"); fs.writeFileSync(path.join(outPlain, "specs-2.txt"), "\n");
  fs.writeFileSync(path.join(outPlain, "chunk-1.json"), JSON.stringify({ stats: {}, errors: [], suites: [{ file: "a.spec.ts", specs: [{ title: "t", file: "a.spec.ts", tests: test3(2) }] }] }));
  const vp = verdict(outPlain, 2, { env: ENV });
  check("without a plan the verdict is the plain suite verdict (no reuse)", vp.ok && /^passed=2 failed=0 .*reused_files=0/.test(vp.line), vp.line);
  check("an empty report is not green", !verdict(path.join(tmp, "nothing"), 2, { env: ENV }).ok);
}

// ---- 5. the real tree --------------------------------------------------------------------------------------------
if (!process.env.SUITE_KEY_SELFTEST_FIXTURES_ONLY) { // the mutation runs of scripts/check-suite-reuse.sh copy the tools only
  const files = execFileSync("git", ["-C", ROOT, "ls-files", "-z", "--cached", "--others", "--exclude-standard"], { encoding: "utf8" }).split("\0").filter((f) => f && fs.existsSync(path.join(ROOT, f)));
  const text = (f) => fs.readFileSync(path.join(ROOT, f), "utf8");
  const head = openTree(ROOT, "HEAD");
  const keys = specKeys(head, { runtime: RT, env: ENV });
  eq("every spec file of the real tree has a key", [...keys].filter(([, k]) => !k.key).map(([f, k]) => `${f}: ${k.why}`), []);
  check("the spec set is the set shard.mjs deals", JSON.stringify(specFilesOf(head)) === JSON.stringify(specFilesOnDisk(path.join(ROOT, "playwright")).filter((f) => files.includes(`playwright/${f}`) && head.entries.has(`playwright/${f}`)).sort()));
  check("the selector uses the same import walker", /from "\.\/import-graph\.mjs"/.test(text("playwright/tools/select.mjs")) && /from "\.\/import-graph\.mjs"/.test(text("playwright/tools/suite-key.mjs")));
  check("the site key of the real tree derives (every include is readable)", (() => { try { return !!siteKey(head, {}).key; } catch (e) { console.error(e.message); return false; } })());
  eq("the real build reads dev-docs/motion.md and nothing else of the excluded paths", rustIncludes(head).targets.size > 0 && [...rustIncludes(head).targets].filter((p) => /^(playwright|dev-docs|docs|primitives\/ts-test)\//.test(p)), ["dev-docs/motion.md"]);

  // each registered reader exists, and only the reader (and the data itself) names its data
  const pwModules = files.filter((f) => f.startsWith("playwright/") && /\.(?:ts|cjs|mjs|js)$/.test(f) && !f.startsWith("playwright/tools/") && !f.startsWith("playwright/node_modules/"));
  for (const r of DATA_READERS) {
    check(`registered readers exist: ${r.readers.join(", ")}`, r.readers.every((p) => files.includes(p)));
    const naming = pwModules.filter((f) => r.mentions.test(stripComments(text(f))));
    eq(`only the registered reader names ${r.mentions}`, naming.filter((f) => !r.readers.includes(f) && !r.data.test(f)), []);
    check(`the registered data of ${r.readers[0]} exists`, files.some((f) => r.data.test(f)));
  }

  // The release build's inputs name no excluded path, except where reviewed (display text, a comment, a gate-only crate).
  const REVIEWED = {
    "preview/index.html": "a comment",
    "preview/src/components/form/component.rs": "rendered text",
    "preview/src/components/top_layer/component.rs": "rendered text",
    "preview/src/motion_docs.rs": "the include (read through the include scan) and a test that reads the same file",
    "primitives/src/carousel/root.rs": "a message string",
    "scripts/attr-spread-collision-checker/regenerate-attribute-idents.py": "a gate: its crate is not a dependency of the site",
    "scripts/attr-spread-collision-checker/src/main.rs": "a gate: its crate is not a dependency of the site",
    "scripts/build-ssg.sh": "message strings",
  };
  const BUILD_INPUT = /^(?:(?:primitives|preview|dioxus-attributes|decision-kernel|cli|test-harness|labs)\/.*(?:\.rs|Cargo\.toml|Dioxus\.toml|index\.html)|Cargo\.toml|scripts\/(?:build-ssg|ssg-verify|emit-404|lane-target|lane-ssg|lane-netns|lane-touch|lane-lock)\.sh|scripts\/ssg-css-bundle\.mjs|scripts\/attr-spread-collision-checker\/.*)$/;
  const namesExcluded = files.filter((f) => BUILD_INPUT.test(f) && stripComments(text(f)).split("\n").some((l) => !/^\s*(?:\/\/|#|\*)/.test(l) && /(?:dev-docs|playwright|primitives\/ts-test)\//.test(l)));
  eq("no build input names an excluded path unless it is reviewed here", namesExcluded.filter((f) => !(f in REVIEWED)), []);
  check("every reviewed entry still names one (a stale entry is removed)", Object.keys(REVIEWED).every((f) => namesExcluded.includes(f)), Object.keys(REVIEWED).filter((f) => !namesExcluded.includes(f)).join(" "));
}

console.log(`suite-key self-test: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
