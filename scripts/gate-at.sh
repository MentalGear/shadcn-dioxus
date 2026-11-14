#!/usr/bin/env bash
# Run scripts/run-gates.sh on a PINNED commit in its own worktree, so merges in the main checkout
# can never change the tree under a running gate (a merge during a run would leave conflict
# markers in the checked tree; backlog row 221). Usage: scripts/gate-at.sh <commit> <logfile>
# <logfile> holds run-gates' stdout; the per-gate logs are in the run's own directory, named by its `logs:` line
# (printed here too; row 440).
# LANE_TGT (default /home/user/tgt) holds the lock queue, the gates target and the worktree root.
set -uo pipefail
export T="${LANE_TGT:-/home/user/tgt}" LANE_TGT="${LANE_TGT:-/home/user/tgt}"; repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
c="${1:?commit}"; log="${2:?log}"
cd "$repo"
c="$(git rev-parse --verify "$c^{commit}")" || exit 2
# One worktree PER COMMIT: a shared one would let a second gate-at re-check-out the tree under a first
# run still queued for the lock, so that run would test (and its log, named for its commit, vouch
# for) another commit (row 221).
wt="${LANE_WT:-/home/user/wt}/gates-${c:0:12}"
. "$repo/scripts/lib/pinned-worktree.sh"
trap 'pinned_worktree_remove "$repo" "$wt"' EXIT
pinned_worktree_add "$repo" "$c" "$wt" || exit 2
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
grep -E "FAIL|ALL GREEN|gates run|^logs: |EXIT" "$log"
