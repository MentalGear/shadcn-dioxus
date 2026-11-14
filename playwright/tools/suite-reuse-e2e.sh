#!/usr/bin/env bash
# End-to-end exercise of the real chunk path of the per-file suite reuse (backlog row 513): the real tools/run-suite.sh with
# `--reuse`, the real Playwright runner and suite.local.config.ts, on a scratch git repo with planted specs and a tiny planted
# site (no build, no browser: the planted tests use no page). It drives a pass, a failing test, a `test.only`, a test that
# interrupts its own runner, a skip decided at run time and a script no spec imports through record, plan, verdict and a second
# run, and checks that only what should run runs and that nothing wrong is accepted. Runs through the repo's Playwright lock
# (tools/run-suite.sh takes it per chunk), two workers.
#
#   playwright/tools/suite-reuse-e2e.sh        Exit status: 0 = every expectation held.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
real="$(cd "$here/../.." && pwd)"
[ -d "$real/playwright/node_modules" ] || { echo "suite-reuse-e2e: playwright/node_modules is missing"; exit 1; }
tmp="$(mktemp -d)"; [ -n "${SUITE_E2E_KEEP:-}" ] || trap 'rm -rf "$tmp"' EXIT
[ -z "${SUITE_E2E_KEEP:-}" ] || echo "suite-reuse-e2e: keeping $tmp"
sc="$tmp/repo"; store="$tmp/records"; site="$tmp/site"
fail=0
ok() { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1" >&2; fail=1; }
expect() { if [ "$2" = "$3" ]; then ok "$1 [$3]"; else bad "$1 (got '$2', want '$3')"; fi; }

# ---- the scratch repo: the real tools and config, planted specs, a planted site ---------------------------------------------
mkdir -p "$sc/playwright" "$sc/scripts" "$sc/preview/src" "$site/demos"
cp -r "$real/playwright/tools" "$sc/playwright/tools"
for f in base-url.ts sandbox-chromium.cjs suite.local.config.ts package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc; do cp "$real/playwright/$f" "$sc/playwright/$f"; done
cp "$real/scripts/emit-404.sh" "$real/scripts/serve-spa.py" "$sc/scripts/"
ln -s "$real/scripts/ts-tools" "$sc/scripts/ts-tools"
ln -s "$real/playwright/node_modules" "$sc/playwright/node_modules"
printf '#!/usr/bin/env bash\nexit 0\n' > "$sc/playwright/tools/ensure-attr-synth.sh"
printf '{"files":{}}\n' > "$sc/playwright/durations.json"
printf 'export {};\n' > "$sc/playwright/fixtures.ts"
printf '[workspace]\n' > "$sc/Cargo.toml"
printf 'fn main() {}\n' > "$sc/preview/src/main.rs"
printf 'node_modules\ntest-results\n' > "$sc/.gitignore"
printf '<html><head><link rel="stylesheet" href="/assets/x.css"></head><body><script>window.initial_dioxus_hydration_data=1</script><script type="module"></script></body></html>\n' > "$site/demos/index.html"
printf '<html><body>home</body></html>\n' > "$site/index.html"
git -C "$sc" init -q . && git -C "$sc" config user.email t@t && git -C "$sc" config user.name t

spec() { printf '%s\n' "$2" > "$sc/playwright/$1"; }
PASS='import { test, expect } from "@playwright/test";
test("adds", () => { expect(1 + 1).toBe(2); });
test("subtracts", () => { expect(2 - 1).toBe(1); });'
FAIL1='import { test, expect } from "@playwright/test";
test("holds", () => { expect(1).toBe(1); });
test("breaks", () => { expect(1).toBe(2); });'
FAIL2='import { test, expect } from "@playwright/test";
test("holds", () => { expect(1).toBe(1); });
test("breaks", () => { expect(2).toBe(2); });'
SKIP='import { test, expect } from "@playwright/test";
test("needs a transient capability", () => { test.skip(true, "CDP unavailable"); });
test("runs", () => { expect(1).toBe(1); });'
INTR='import { test } from "@playwright/test";
test("stops its own runner", async () => { process.kill(process.ppid, "SIGINT"); await new Promise((r) => setTimeout(r, 4000)); });
test("after it", () => {});'
ONLY='import { test, expect } from "@playwright/test";
test.only("focused", () => { expect(1).toBe(1); });'
commit() { git -C "$sc" add -A && git -C "$sc" commit -q --allow-empty -m "$1"; }
tree() { git -C "$sc" rev-parse 'HEAD^{tree}'; }
sk() { ( cd "$sc/playwright" && TIMING_PASS=off TIMING_REENABLE_ROW=494 PW_BUILD_PROFILE=release FIRST_PAINT_ROUTES=all LS_STEP=32 node tools/suite-key.mjs "$@" ); }
run_suite() { # out; the real command line of the deploy, minus the build
  ( cd "$sc/playwright" && PW_BUILD_PROFILE=release SSG_SITE_DIR="$site" FIRST_PAINT_ROUTES=all LS_STEP=32 SUITE_BUILD_ENV=e2e \
      tools/run-suite.sh --out "$1" --chunks 2 --workers 2 --reuse "$store" --tree "$(tree)" ) > "$1.log" 2>&1
}
run_list() { node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).run.join(" "))' "$1/plan.json"; }
records() { find "$store" -name '*.json' 2>/dev/null | sed -E 's|.*/||; s|%2F|/|g; s|@.*||' | sort | tr '\n' ' ' | sed 's/ $//'; }
status_of() { # file -> status of its latest record
  node -e 'const fs=require("fs"),p=require("path");const out={};const w=(d)=>{for(const e of fs.readdirSync(d,{withFileTypes:true})){const f=p.join(d,e.name);if(e.isDirectory())w(f);else if(f.endsWith(".json")){const r=JSON.parse(fs.readFileSync(f,"utf8"));if(!out[r.file]||out[r.file].at<r.recordedAt)out[r.file]={at:r.recordedAt,s:r.status}}}};try{w(process.argv[1])}catch{};console.log(out[process.argv[2]]?.s??"none")' "$store" "$1"
}
verdict() { ( cd "$sc/playwright" && TIMING_PASS=off TIMING_REENABLE_ROW=494 node tools/suite-key.mjs verdict --out "$1" --chunks 2 --tree "$(tree)" --store "$store" 2>&1 ); }

# ---- 1. first run: everything runs, only what is proved is recorded ---------------------------------------------------------
echo "1. a pass, a failing test, a runtime skip, a test that interrupts its runner, a script no spec imports"
mkdir -p "$sc/playwright/oracle/reference/v1/js"
spec pass.spec.ts "$PASS"; spec fail.spec.ts "$FAIL1"; spec skip.spec.ts "$SKIP"; spec interrupt.spec.ts "$INTR"
echo 'export const m = 1;' > "$sc/playwright/oracle/reference/v1/js/loader.js"
commit c1
run_suite "$tmp/r1"
expect "every file runs the first time" "$(run_list "$tmp/r1")" "fail.spec.ts interrupt.spec.ts pass.spec.ts skip.spec.ts"
expect "the passing file is recorded green" "$(status_of pass.spec.ts)" green
expect "the failing file is recorded red" "$(status_of fail.spec.ts)" red
expect "the file with a runtime skip is not recorded" "$(status_of skip.spec.ts)" none
expect "the file whose test interrupted its runner is not recorded" "$(status_of interrupt.spec.ts)" none
v="$(verdict "$tmp/r1")"; rc=$?
expect "the first run's verdict is red" "$rc" 1
echo "$v" | sed 's/^/       /'

# ---- 2. a test.only: the lists fail, nothing is recorded -------------------------------------------------------------------
echo "2. a test.only in a new file"
before="$(records)"
spec only.spec.ts "$ONLY"; commit c1b
run_suite "$tmp/r1b"; rc=$?
expect "listing with a test.only stops the run (exit 2)" "$rc" 2
expect "that run records nothing" "$(records)" "$before"
verdict "$tmp/r1b" >/dev/null; expect "and its verdict is red" "$?" 1

# ---- 3. the spec-only fix: only what the fix can affect runs --------------------------------------------------------------
echo "3. the fix: the failing test is repaired and the test.only is gone; the site is the same"
rm "$sc/playwright/only.spec.ts"; spec fail.spec.ts "$FAIL2"; commit c2
run_suite "$tmp/r2"
expect "the passing file is reused, the rest runs" "$(run_list "$tmp/r2")" "fail.spec.ts interrupt.spec.ts skip.spec.ts"
expect "the repaired file is recorded green" "$(status_of fail.spec.ts)" green
verdict "$tmp/r2" >/dev/null; expect "the verdict is still red: the interrupted file did not run" "$?" 1

# ---- 4. the interrupting file is removed: a green verdict with the reuse stated separately ---------------------------------
echo "4. the interrupting file is removed"
rm "$sc/playwright/interrupt.spec.ts"; commit c3
run_suite "$tmp/r3"
expect "only the file with a runtime skip runs (it is never recorded)" "$(run_list "$tmp/r3")" "skip.spec.ts"
v="$(verdict "$tmp/r3")"; rc=$?
expect "the verdict is green" "$rc" 0
case "$v" in *"reused_files=2 "*"run_files=1"*) ok "whole-suite totals with the reuse stated: $v" ;; *) bad "totals do not state the reuse: $v" ;; esac
sk accept --store "$store" --tree "$(tree)" --build-env e2e >/dev/null 2>&1; expect "the tree is not accepted from records alone (a file with a skip runs every time)" "$?" 1

# ---- 5. no file with a skip: accepted from records, no run ----------------------------------------------------------------
echo "5. the file with the skip is removed"
rm "$sc/playwright/skip.spec.ts"; commit c4
out="$(sk accept --store "$store" --tree "$(tree)" --build-env e2e 2>&1)"; rc=$?
expect "every file has a green record: accepted with no build and no run" "$rc" 0
echo "$out" | tail -1 | sed 's/^/       /'

# ---- 6. a script no spec imports changes: every file runs again -----------------------------------------------------------
echo "6. the vendored script no spec imports changes"
echo 'throw new Error("x");' >> "$sc/playwright/oracle/reference/v1/js/loader.js"; commit c5
sk accept --store "$store" --tree "$(tree)" --build-env e2e >/dev/null 2>&1; expect "the tree is no longer accepted" "$?" 1
run_suite "$tmp/r5"
expect "every spec file runs again" "$(run_list "$tmp/r5")" "fail.spec.ts pass.spec.ts"
verdict "$tmp/r5" >/dev/null; expect "and the verdict is green" "$?" 0

# ---- 7. a worktree edit during the run: nothing is recorded ---------------------------------------------------------------
echo "7. a tracked file changes while the run is in progress"
echo 'export const m = 3;' > "$sc/playwright/oracle/reference/v1/js/loader.js"; commit c6
( cd "$sc/playwright" && PW_BUILD_PROFILE=release SSG_SITE_DIR="$site" FIRST_PAINT_ROUTES=all LS_STEP=32 SUITE_BUILD_ENV=e2e \
    tools/run-suite.sh --out "$tmp/r6" --chunks 2 --workers 2 --reuse "$store" --tree "$(tree)" > "$tmp/r6.log" 2>&1 ) &
pid=$!
while [ ! -s "$tmp/r6/plan.json" ] && kill -0 "$pid" 2>/dev/null; do sleep 0.2; done
echo '// edited during the run' >> "$sc/playwright/pass.spec.ts"
wait "$pid"
grep -q "nothing is recorded" "$tmp/r6.log" && ok "the records of the run an edit landed in are refused" || bad "records were written although a tracked file changed during the run"
verdict "$tmp/r6" >/dev/null; expect "and its verdict is red" "$?" 1
git -C "$sc" checkout -q -- .

[ "$fail" = 0 ] && echo "suite-reuse-e2e: OK" || { echo "suite-reuse-e2e: FAIL"; exit 1; }
