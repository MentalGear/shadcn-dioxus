#!/usr/bin/env bash
#
# setup-merge-drivers.sh   (backlog row 400)
#
# Registers the git merge drivers that `.gitattributes` names (`merge=backlog-index`, `merge=durations`) in a clone's
# repo config. Git never reads a driver from the tree, only from config, so every clone needs this once: the
# SessionStart hook (scripts/session-setup.sh) runs it, and this script is the manual run for any other clone.
# Worktrees share the repo's config, so one registration covers the main checkout and every `git worktree` of it.
# Idempotent: a key that already holds the value is not written again.
#
# Each driver command runs from the top of the work tree being merged and prefers the script in THAT tree; a tree that
# does not hold the driver script (a branch from before the drivers) gets git's own text merge (`git merge-file`), which is
# what the attribute does without a driver, so it can never add a conflict.
#
# Usage: scripts/setup-merge-drivers.sh [--repo DIR] [--check]
#   --repo DIR  register in DIR's repository instead of this one (scratch repos in scripts/check-merge-drivers.sh)
#   --check     write nothing; exit 1 when a key is missing or differs
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check=0
while [ $# -gt 0 ]; do
  case "$1" in
    --repo) repo="$2"; shift 2 ;;
    --check) check=1; shift ;;
    *) echo "usage: $0 [--repo DIR] [--check]" >&2; exit 2 ;;
  esac
done

# name | human name | script (relative to the work tree) | interpreter
drivers=(
  "backlog-index|generated backlog index (3-way merge by row id)|scripts/merge-driver-backlog-index.py|python3 -I"
  "durations|playwright durations table (3-way merge by entry)|playwright/tools/durations-merge.mjs|node"
)

# $1=%O (base) $2=%A (ours, the output) $3=%B (theirs) $4=%P (path)
command_for() { # script interpreter
  printf '%s' "sh -c 'if [ -f $1 ]; then exec $2 $1 \"\$@\"; else exec git merge-file \"\$2\" \"\$1\" \"\$3\"; fi' _ %O %A %B %P"
}

rc=0
for d in "${drivers[@]}"; do
  IFS='|' read -r name human script interp <<<"$d"
  want_driver="$(command_for "$script" "$interp")"
  for kv in "merge.$name.name=$human" "merge.$name.driver=$want_driver"; do
    key="${kv%%=*}"; want="${kv#*=}"
    have="$(git -C "$repo" config --local --get "$key" 2>/dev/null || true)"
    if [ "$have" = "$want" ]; then continue; fi
    if [ "$check" = 1 ]; then echo "setup-merge-drivers: $key is not registered as expected" >&2; rc=1; continue; fi
    git -C "$repo" config --local "$key" "$want"
    echo "setup-merge-drivers: set $key" >&2
  done
done
exit $rc
