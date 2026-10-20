#!/usr/bin/env bash
# The one command lanes use for cargo (backlog row 221, rows 168/216):
#
#   scripts/lane-cargo.sh [gate|lane] [--label T] [--target NAME] -- cargo clippy ...
#
# Takes a slot in the lane-lock queue, then (inside it) points CARGO_TARGET_DIR at this slot's
# `check` dir (slot 0: $LANE_TGT/check, slot 1: check-s1, created by lane-slot-target.sh),
# touches this worktree's sources if the target was last used by another tree (lane-touch.sh,
# row 216), and runs the command with CARGO_BUILD_JOBS=2 (LANE_CARGO_JOBS) and CARGO_INCREMENTAL=0.
# Run it from inside your worktree. Never start a long-lived process through it.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ "${1:-}" = "--inner" ]; then
  shift; name="$1"; shift
  [ "$1" = "--" ] && shift
  CARGO_TARGET_DIR="$("$here/lane-slot-target.sh" dir "$name")"; export CARGO_TARGET_DIR
  "$here/lane-touch.sh" "$CARGO_TARGET_DIR"
  export CARGO_BUILD_JOBS="${LANE_CARGO_JOBS:-2}" CARGO_INCREMENTAL=0
  exec "$@"
fi
class=lane label="" name=check
while [ $# -gt 0 ]; do
  case "$1" in
    gate|lane) class="$1"; shift ;;
    --label) label="${2:?}"; shift 2 ;;
    --target) name="${2:?}"; shift 2 ;;
    --) shift; break ;;
    *) break ;;
  esac
done
[ $# -gt 0 ] || { echo "usage: lane-cargo.sh [gate|lane] [--label T] [--target NAME] -- cargo ..." >&2; exit 2; }
exec "$here/lane-lock.sh" "$class" --label "${label:-lane-cargo: $*}" -- "$here/lane-cargo.sh" --inner "$name" -- "$@"
