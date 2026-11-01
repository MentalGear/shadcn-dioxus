#!/usr/bin/env bash
# The timing pass: every `@timing` case of the selected specs, alone, on a quiet machine (dev-docs/dev-loop.md,
# "Running tests"). The caller ran everything else with `--grep-invert @timing` first.
#
#   playwright/tools/timing-pass.sh --config CONFIG [--out PREFIX] [--lock FILE] [-- playwright args: spec files, ...]
#
#   --out PREFIX   writes PREFIX.json (the JSON report; a stub with zero tests when there was nothing to run), PREFIX.log
#                  and PREFIX.wall (seconds inside the lock, queue and wait excluded)
#   --lock FILE    takes this flock around the wait and the run (run-suite.sh passes pw.lock); without it the caller
#                  holds the lock already (`flock pw.lock pnpm run test:changed`), and taking it again would deadlock
# Runs `playwright test --grep @timing --workers=1` after tools/wait-quiet.sh; the spec selection comes from the
# caller's arguments and PW_SPECS_FILE. A debug build measures nothing (the cases skip themselves), so it neither waits
# nor runs, and says so. Exit: 0 green or nothing to run, 1 a red case or the machine never went quiet.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
config="" out="" lock=""
while [ $# -gt 0 ]; do
  case "$1" in
    --config) config="$2"; shift 2 ;; --out) out="$2"; shift 2 ;; --lock) lock="$2"; shift 2 ;;
    --) shift; break ;; *) echo "timing-pass: unknown option $1" >&2; exit 2 ;;
  esac
done
[ -n "$config" ] || { echo "timing-pass: --config is required" >&2; exit 2; }
stub() { [ -z "$out" ] || printf '{"config":{},"suites":[],"errors":[],"stats":{"expected":0,"unexpected":0,"flaky":0,"skipped":0}}\n' > "$out.json"; }
profile="$(node --experimental-strip-types -e 'import("./base-url.ts").then((m) => console.log(m.buildProfile()))' 2>/dev/null || echo unknown)"
if [ "$profile" = debug ] && [ "${PW_FORCE_TIMING_SPECS:-}" != 1 ]; then
  echo "timing-pass: this is a debug build, where the @timing cases skip themselves (a debug build measures nothing); not run" >&2
  stub; exit 0
fi
count="$(pnpm exec playwright test -c "$config" --list --grep @timing --reporter=json "$@" 2>/dev/null \
  | node -e 'let n=0;const w=(s)=>{for(const x of s.suites??[])w(x);for(const p of s.specs??[])n+=(p.tests??[]).length};w(JSON.parse(require("fs").readFileSync(0,"utf8")));console.log(n)' 2>/dev/null || echo 0)"
if [ "${count:-0}" = 0 ]; then echo "timing-pass: no @timing case in the selected specs; nothing to run"; stub; exit 0; fi
echo "timing-pass: $count @timing case(s): waiting for a quiet machine, then one worker"
body='
  tools/wait-quiet.sh || exit 1
  s=$(date +%s)
  if [ -n "$OUT" ]; then
    PLAYWRIGHT_JSON_OUTPUT_FILE="$OUT.json" pnpm exec playwright test -c "$CONFIG" --grep @timing --workers=1 --reporter=json,line "$@" > "$OUT.log" 2>&1; rc=$?
    echo $(( $(date +%s) - s )) > "$OUT.wall"; tail -n 4 "$OUT.log"
  else
    pnpm exec playwright test -c "$CONFIG" --grep @timing --workers=1 "$@"; rc=$?
  fi
  exit $rc'
export CONFIG="$config" OUT="$out"
if [ -n "$lock" ]; then flock "$lock" bash -c "$body" _ "$@"; else bash -c "$body" _ "$@"; fi
