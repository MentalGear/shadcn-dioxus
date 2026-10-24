#!/usr/bin/env bash
# Make a shared/copied cargo target dir rebuild THIS worktree's sources (backlog row 216).
#
#   scripts/lane-touch.sh <target-dir> [--force]     (run from inside the worktree)
#
# cargo gives a workspace crate the SAME unit hash in every worktree and judges freshness by mtime,
# so a lane whose files are older than another tree's artifact in a shared target silently builds
# or lints that other tree's code. The cure is to
# touch the sources before every cargo/dx run -- but only when it is needed: the target's
# `.tree-owner` stamp names the worktree that last touched for it, and when that is THIS worktree
# its own edits are already newer than every artifact, so the touch is skipped (a touch of ~1500
# files costs a full preview rebuild on its own). `--force` always touches (BUILD_SSG_TOUCH=always).
# `scripts/lane-target.sh create` and a fresh template copy clear the stamp, so the first run
# in a new target always touches. Callers: build-ssg.sh, run-gates.sh (before its first cargo gate),
# lane-cargo.sh, lane-ssg.sh, gate-at.sh.
set -euo pipefail
target="${1:?usage: lane-touch.sh <target-dir> [--force]}"
force=0; [ "${2:-}" = "--force" ] && force=1
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
stamp="$target/.tree-owner"
if [ "$force" = 0 ] && [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$root" ]; then
  echo "lane-touch: $target is already owned by $root; sources not touched" >&2
  exit 0
fi
dirs=()
for d in preview/src preview/assets preview/build.rs primitives/src dioxus-attributes/src; do
  [ -e "$root/$d" ] && dirs+=("$root/$d")
done
find "${dirs[@]}" -type f -exec touch {} +
mkdir -p "$target"
printf '%s\n' "$root" > "$stamp"
echo "lane-touch: touched ${#dirs[@]} source trees of $root for $target" >&2
