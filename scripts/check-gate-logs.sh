#!/usr/bin/env bash
# check-gate-logs.sh   (backlog row 440)
#
# Per-run gate log directories keep their promises when `scripts/run-gates.sh` runs concurrently on one target. Two
# `--fast --only <cheap gate>` runs start together against one throwaway CARGO_TARGET_DIR and must:
#   1. write separate run directories (`run-gates/<UTC time>-<pid>/`), each holding only its own gate's log;
#   2. print in the summary a `logs:` line naming its own directory, and every per-gate path under it;
#   3. leave `run-gates/latest` pointing at a FINISHED run directory (one holding the `finished` marker);
# and retention (`RUN_GATES_KEEP`) must delete older FINISHED run directories only: an older run that has not finished
# (no marker, touched recently) and anything that is not a run directory survive, and the run that did the pruning is kept.
# Hermetic: throwaway target dir, the two cheapest gates, no cargo build, no network, no browser; a few seconds.
#
# Usage: scripts/check-gate-logs.sh
set -uo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1" >&2; fail=1; }

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
export CARGO_TARGET_DIR="$tmp/target"
root="$CARGO_TARGET_DIR/run-gates"
unset RUN_GATES_KEEP RUN_GATES_LIVE_MIN
ga=check-cfg-axis gb=check-render-clock

# --- 1-3: two concurrent runs ---------------------------------------------------------------------------
scripts/run-gates.sh --fast --only "^$ga\$" > "$tmp/a.out" 2>&1 & pa=$!
scripts/run-gates.sh --fast --only "^$gb\$" > "$tmp/b.out" 2>&1 & pb=$!
wait "$pa"; ra=$?; wait "$pb"; rb=$?
[ "$ra" = 3 ] && [ "$rb" = 3 ] && ok "both partial runs finished green (exit 3)" || bad "exit codes $ra/$rb (expected 3/3)"

da="$(sed -n 's/^logs: //p' "$tmp/a.out")"; db="$(sed -n 's/^logs: //p' "$tmp/b.out")"
[ -n "$da" ] && [ -n "$db" ] || bad "a summary has no 'logs:' line"
[ "$da" != "$db" ] && ok "the two summaries name different logs: directories" || bad "both runs share one log directory: $da"
case "$da" in "$root"/*) ok "run A logs under run-gates/" ;; *) bad "run A logs outside $root: $da" ;; esac
n="$(find "$root" -mindepth 1 -maxdepth 1 -type d | wc -l)"
[ "$n" = 2 ] && ok "exactly two run directories exist" || bad "expected 2 run directories, found $n"
[ -f "$da/$ga.log" ] && [ ! -e "$da/$gb.log" ] && ok "run A's directory holds only its own gate log" || bad "run A's directory holds the wrong logs: $(ls "$da" 2>/dev/null | tr '\n' ' ')"
[ -f "$db/$gb.log" ] && [ ! -e "$db/$ga.log" ] && ok "run B's directory holds only its own gate log" || bad "run B's directory holds the wrong logs: $(ls "$db" 2>/dev/null | tr '\n' ' ')"
grep -q "  $da/$ga.log\$" "$tmp/a.out" && ! grep -q "$db" "$tmp/a.out" && ok "run A's summary paths stay in its own directory" || bad "run A's summary names another run's directory"
grep -q "  $db/$gb.log\$" "$tmp/b.out" && ! grep -q "$da" "$tmp/b.out" && ok "run B's summary paths stay in its own directory" || bad "run B's summary names another run's directory"
if [ -L "$root/latest" ]; then
  lt="$(readlink -f "$root/latest")"
  if { [ "$lt" = "$(readlink -f "$da")" ] || [ "$lt" = "$(readlink -f "$db")" ]; } && [ -e "$lt/finished" ] && [ -s "$lt/summary.tsv" ]; then
    ok "latest points at a finished run directory"
  else bad "latest -> $lt is not a finished run"; fi
else bad "run-gates/latest is not a symlink"; fi
compgen -G "$root/.latest.*" >/dev/null && bad "a temporary .latest.<pid> link was left behind"

# --- retention: only finished older runs are pruned -----------------------------------------------------
for s in 20250101T000001Z-11 20250101T000002Z-12 20250101T000003Z-13; do mkdir "$root/$s"; : > "$root/$s/finished"; done
mkdir "$root/20250101T000004Z-14"                     # older than the new run, not finished, just touched: a live run
mkdir "$root/not-a-run"; : > "$root/not-a-run/keep.txt"
RUN_GATES_KEEP=1 scripts/run-gates.sh --fast --only "^$ga\$" > "$tmp/c.out" 2>&1; rc=$?
dc="$(sed -n 's/^logs: //p' "$tmp/c.out")"
[ "$rc" = 3 ] && [ -d "$dc" ] && ok "the pruning run finished and kept its own directory" || bad "pruning run rc=$rc, dir '$dc'"
gone=1; for s in 20250101T000001Z-11 20250101T000002Z-12 20250101T000003Z-13; do [ -e "$root/$s" ] && gone=0; done
[ "$gone" = 1 ] && ok "older finished runs were pruned" || bad "an older finished run survived RUN_GATES_KEEP=1"
[ -d "$root/20250101T000004Z-14" ] && ok "an older unfinished run was not pruned" || bad "an unfinished run was pruned"
[ -e "$root/not-a-run/keep.txt" ] && ok "a non-run directory was not pruned" || bad "a non-run directory was pruned"

# an unfinished run idle past RUN_GATES_LIVE_MIN is a crashed one and is pruned
RUN_GATES_LIVE_MIN=0 RUN_GATES_KEEP=1 scripts/run-gates.sh --fast --only "^$ga\$" > "$tmp/d.out" 2>&1
[ ! -e "$root/20250101T000004Z-14" ] && ok "a stale unfinished run is pruned" || bad "a stale unfinished run survived"

if [ "$fail" = 0 ]; then echo "check-gate-logs: OK"; else echo "check-gate-logs: FAILED" >&2; fi
exit "$fail"
