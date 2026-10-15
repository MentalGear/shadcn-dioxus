#!/usr/bin/env bash
# Run scripts/run-gates.sh on a PINNED commit in its own worktree, so merges in the main checkout
# can never change the tree under a running gate (2026-10-14: a merge during a run left conflict
# markers in the checked tree; backlog row 221). Usage: scripts/gate-at.sh <commit> <logfile>
# LANE_TGT (default /home/user/tgt) holds the lock, the gates target and the worktree root.
set -uo pipefail
export T="${LANE_TGT:-/home/user/tgt}"; repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
c="${1:?commit}"; log="${2:?log}"; wt="${LANE_WT:-/home/user/wt}/gates"
cd "$repo"
git worktree remove --force "$wt" >/dev/null 2>&1 || true
git worktree add -q --detach "$wt" "$c" || exit 2
ln -sfn "$repo/preview/node_modules" "$wt/preview/node_modules"
ln -sfn "$repo/playwright/node_modules" "$wt/playwright/node_modules"
cd "$wt"
flock "$T/ssg.lock" sh -c 'git rev-parse --short HEAD; find preview/src preview/assets preview/build.rs primitives/src dioxus-attributes/src -type f -exec touch {} + && CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=$T/gates scripts/run-gates.sh' > "$log" 2>&1
echo "EXIT $?" >> "$log"
grep -E "FAIL|ALL GREEN|gates run|EXIT" "$log"
