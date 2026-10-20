#!/usr/bin/env bash
# Run scripts/run-gates.sh on a PINNED commit in its own worktree, so merges in the main checkout
# can never change the tree under a running gate (2026-10-14: a merge during a run left conflict
# markers in the checked tree; backlog row 221). Usage: scripts/gate-at.sh <commit> <logfile>
# LANE_TGT (default /home/user/tgt) holds the lock queue, the gates target and the worktree root.
set -uo pipefail
export T="${LANE_TGT:-/home/user/tgt}" LANE_TGT="${LANE_TGT:-/home/user/tgt}"; repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
c="${1:?commit}"; log="${2:?log}"
cd "$repo"
c="$(git rev-parse --verify "$c^{commit}")" || exit 2
# One worktree PER COMMIT: a shared one let a second gate-at re-check-out the tree under a first
# run still queued for the lock, so that run tested (and its log, named for its commit, vouched
# for) another commit (2026-10-18, row 221).
wt="${LANE_WT:-/home/user/wt}/gates-${c:0:12}"
git worktree remove --force "$wt" >/dev/null 2>&1 || true
git worktree add -q --detach "$wt" "$c" || exit 2
trap 'cd "$repo" && git worktree remove --force "$wt" >/dev/null 2>&1' EXIT
ln -sfn "$repo/preview/node_modules" "$wt/preview/node_modules"
ln -sfn "$repo/playwright/node_modules" "$wt/playwright/node_modules"
cd "$wt"
# Class `gate` in the lane-lock queue (row 221): a merge gate goes before every waiting lane build, and the
# wrapper script paths are the MAIN checkout's, because an older commit under test may not have them.
# Inside the slot: this slot's gates target dir (slot 0 = $T/gates), then touch the worktree (row 216).
# The queue's own chatter (queued/acquired) goes to "$log.queue": line 1 of "$log" must be the tested HEAD.
LANE_REPO="$repo" GATE_LOG="$log" "$repo/scripts/lane-lock.sh" gate --label "gate-at ${c:0:12}" -- sh -c '
  exec >"$GATE_LOG" 2>&1
  git rev-parse --short HEAD
  CARGO_TARGET_DIR="$("$LANE_REPO/scripts/lane-slot-target.sh" dir gates)" || exit 1
  export CARGO_TARGET_DIR CARGO_BUILD_JOBS=2
  "$LANE_REPO/scripts/lane-touch.sh" "$CARGO_TARGET_DIR" || exit 1
  exec scripts/run-gates.sh' 2> "$log.queue"
rc=$?
got="$(head -1 "$log")"
if [ "$got" != "${c:0:${#got}}" ] || [ -z "$got" ]; then echo "gate-at: ran on '$got', not $c -- NOT a pass for $c" >> "$log"; rc=9; fi
echo "EXIT $rc" >> "$log"
grep -E "FAIL|ALL GREEN|gates run|EXIT" "$log"
