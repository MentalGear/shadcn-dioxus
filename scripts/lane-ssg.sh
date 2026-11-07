#!/usr/bin/env bash
# Queued debug SSG build for a lane worktree, then an optional static server.
#   scripts/lane-ssg.sh <worktree-abs-path> <lane-name> [port]   (LANE_TGT, default /home/user/tgt; see dev-docs/lane-rules.md)
# The build runs inside scripts/lane-lock.sh (FIFO ticket queue, class `lane`, a slot of its own; row 221):
# slot 0 builds in place in the warm template ($T/template; row 503: under the template lock, sources touched when
# the template's .tree-owner names another worktree) and copies public/ to $T/pub-<lane>; a slot >= 1 (or
# LANE_SSG_COPY=1) copies the template to a scratch target ($T/ssg-lane<slot suffix>), builds there, publishes
# and deletes it. The pub dir is replaced atomically (a failed build leaves the previous one).
# The server starts AFTER the lock is released, with every lock fd closed, and (re)starts on [port]
# (kills only the PID it recorded itself). Prints PUBLIC=<dir> and URL=<url>.
# `--locked-build <worktree> <lane>` is the inner mode lane-lock runs; do not call it by hand.
set -euo pipefail
T="${LANE_TGT:-/home/user/tgt}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "${1:-}" = "--locked-build" ]; then
  wt="${2:?worktree}"; lane="${3:?lane}"
  pub="$T/pub-$lane"; log="$T/ssg-$lane.log"
  echo "lane-ssg: building $wt in slot ${LANE_SLOT:-?} (log $log)" >&2
  # Publish atomically: the served dir is replaced only by a complete copy, so a failed or killed build leaves
  # the previous pub dir (or none) and never a half-written one.
  publish() { rm -rf "$pub.tmp.$$"; cp -a "$1" "$pub.tmp.$$" && rm -rf "$pub" && mv "$pub.tmp.$$" "$pub"; }
  export LANE_RESERVE_PID=$$
  trap '"$here/lane-slot-target.sh" release' EXIT
  if [ "${LANE_SLOT:-0}" = 0 ] && [ "${LANE_SSG_COPY:-0}" != 1 ]; then
    # Slot 0 builds IN the warm template (row 503): the slot lock already serialises lane builds, so a 3 GB copy
    # per build buys nothing. Correctness is the mtime rule (row 216): lane-touch touches this worktree's sources
    # unless the template's .tree-owner stamp already names it, and cargo then rebuilds exactly what differs.
    # The exclusive template lock keeps a concurrent copy FROM the template (slot >= 1, lane-target.sh create)
    # out until the build is over. The guard needs only the new pub dir plus the margin: the target is rewritten
    # in place. A failed build leaves a normal cargo target, usable by the next build.
    tpl="$T/template"
    [ -d "$tpl" ] || { echo "lane-ssg: no warm template at $tpl (CARGO_TARGET_DIR=$tpl scripts/build-ssg.sh debug, once)" >&2; exit 1; }
    mkdir -p "$T/lockq"; exec 204>>"$("$here/lane-slot-target.sh" template-lock)"; flock 204
    "$here/lane-slot-target.sh" reserve "$tpl/dx/preview/debug/web/public"
    "$here/lane-slot-target.sh" release
    cd "$wt"
    "$here/lane-touch.sh" "$tpl"
    if ! CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="$tpl" CARGO_INCREMENTAL=0 BUILD_SSG_TOUCH=never BUILD_SSG_NETNS=1 scripts/build-ssg.sh debug >"$log" 2>&1 204>&-; then
      tail -30 "$log" >&2; echo "lane-ssg: BUILD FAILED (full log $log); the template is intact for the next build" >&2; exit 1
    fi
    publish "$tpl/dx/preview/debug/web/public"
  else
    scratch="$T/ssg-lane${LANE_SLOT_SUFFIX:-}"
    rm -rf "$scratch"
    # Reserve the copy's bytes (row 356) so a concurrent slot copy sees them; the copy never carries .tree-owner
    # (a fresh copy is always touched, row 216), and a failed copy leaves no scratch behind.
    "$here/lane-slot-target.sh" reserve "$T/template"
    if ! "$here/lane-slot-target.sh" copy "$T/template" "$scratch"; then rm -rf "$scratch"; echo "lane-ssg: copy of the template failed" >&2; exit 1; fi
    "$here/lane-slot-target.sh" release
    cd "$wt"
    "$here/lane-touch.sh" "$scratch" --force
    if ! CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="$scratch" CARGO_INCREMENTAL=0 BUILD_SSG_TOUCH=never BUILD_SSG_NETNS=1 scripts/build-ssg.sh debug >"$log" 2>&1; then
      rm -rf "$scratch"; tail -30 "$log" >&2; echo "lane-ssg: BUILD FAILED (full log $log)" >&2; exit 1
    fi
    publish "$scratch/dx/preview/debug/web/public"; rm -rf "$scratch"
  fi
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
