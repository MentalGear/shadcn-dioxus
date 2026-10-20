#!/usr/bin/env bash
# Queued debug SSG build for a lane worktree, then an optional static server.
#   scripts/lane-ssg.sh <worktree-abs-path> <lane-name> [port]   (LANE_TGT, default /home/user/tgt; see dev-docs/lane-rules.md)
# The build runs inside scripts/lane-lock.sh (FIFO ticket queue, class `lane`, a slot of its own; row 221):
# it copies the warm, known-good template to a fresh scratch target ($T/ssg-lane<slot suffix>), builds there,
# copies public/ to $T/pub-<lane>, deletes the scratch target. (A target dir SHARED across worktrees failed 3/5
# builds with "stylesheet ... not in dx's asset manifest", so every build now starts from the same clean tree.)
# The server starts AFTER the lock is released, with every lock fd closed, and (re)starts on [port]
# (kills only the PID it recorded itself). Prints PUBLIC=<dir> and URL=<url>.
# `--locked-build <worktree> <lane>` is the inner mode lane-lock runs; do not call it by hand.
set -euo pipefail
T="${LANE_TGT:-/home/user/tgt}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "${1:-}" = "--locked-build" ]; then
  wt="${2:?worktree}"; lane="${3:?lane}"
  pub="$T/pub-$lane"; log="$T/ssg-$lane.log"; scratch="$T/ssg-lane${LANE_SLOT_SUFFIX:-}"
  echo "lane-ssg: building $wt in slot ${LANE_SLOT:-?} (log $log)" >&2
  rm -rf "$scratch"
  "$here/lane-slot-target.sh" check-disk "$T/template"
  cp -a "$T/template" "$scratch"
  rm -f "$scratch/.tree-owner"   # a fresh copy is always touched (row 216)
  cd "$wt"
  "$here/lane-touch.sh" "$scratch" --force
  if ! CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="$scratch" CARGO_INCREMENTAL=0 BUILD_SSG_TOUCH=never BUILD_SSG_NETNS=1 scripts/build-ssg.sh debug >"$log" 2>&1; then
    rm -rf "$scratch"; tail -30 "$log" >&2; echo "lane-ssg: BUILD FAILED (full log $log)" >&2; exit 1
  fi
  rm -rf "$pub"; cp -a "$scratch/dx/preview/debug/web/public" "$pub"; rm -rf "$scratch"
  # The site's unknown-URL behaviour, as GitHub Pages serves it (row 152a): 404.html + serve-spa.py.
  "$here/emit-404.sh" "$pub" >/dev/null
  exit 0
fi

wt="${1:?worktree}"; lane="${2:?lane}"; port="${3:-}"
pub="$T/pub-$lane"
"$here/lane-lock.sh" lane --label "lane-ssg:$lane" -- "$here/lane-ssg.sh" --locked-build "$wt" "$lane"
echo "PUBLIC=$pub"
if [ -n "$port" ]; then
  pidf="$T/serve-$lane.pid"
  [ -f "$pidf" ] && kill "$(cat "$pidf")" 2>/dev/null || true
  sleep 0.5
  # Outside the lock, and with any inherited lock fd closed: a long-lived child that inherits a locked
  # file description holds the lock after its parent exits (an orphaned server once stalled every lane's build, row 221).
  # serve-spa.py answers unknown URLs with 404.html and status 404, as GitHub Pages does (row 152a).
  nohup python3 "$here/serve-spa.py" "$pub" "$port" >/dev/null 2>&1 9>&- 3>&- </dev/null &
  echo $! > "$pidf"
  echo "URL=http://127.0.0.1:$port"
fi
