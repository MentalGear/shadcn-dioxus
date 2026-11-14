#!/usr/bin/env bash
#
# lane-worktree.sh
#
# The one command that creates and removes a LANE worktree (a worktree on a branch, where a lane commits), so that a lane's
# first Playwright or TypeScript-tool command finds its `node_modules`: `git worktree add` leaves `playwright/`, `preview/`
# and `scripts/ts-tools/` without one, and `scripts/lib/pinned-worktree.sh` holds the link step (shared with the detached
# worktrees of scripts/gate-at.sh). The links are ignored by git (`.gitignore`'s `node_modules` line matches a symlink), so
# `git status` in the worktree stays clean. The node_modules come from the main checkout, found through the repository's common
# git dir, so the command gives the same result when it runs from a lane worktree.
#
# Usage:
#   scripts/lane-worktree.sh add <branch> <path> [<start-point>]
#       <branch> that does not exist is created at <start-point> (default HEAD) and checked out in <path>; an existing <branch>
#       is checked out as it is and takes no <start-point>. Refuses when <path> exists, or <branch> is checked out elsewhere.
#   scripts/lane-worktree.sh remove <path>
#       removes the worktree; refuses while it has uncommitted or untracked changes (the links do not count). The branch stays.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
usage() { sed -n '/^# Usage:/,/^set /p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2; exit 2; }
die() { echo "lane-worktree: $*" >&2; exit 1; }
# the main checkout: the parent of the common git dir (a worktree's own .git is a file that points into it)
main="$(cd "$here" && cd "$(git rev-parse --git-common-dir)/.." && pwd)"
. "$here/scripts/lib/pinned-worktree.sh"

abs() { case "$1" in /*) printf '%s' "$1" ;; *) printf '%s/%s' "$PWD" "$1" ;; esac; }

cmd="${1:-}"; [ $# -gt 0 ] && shift
case "$cmd" in
  add)
    [ $# -ge 2 ] && [ $# -le 3 ] || usage
    branch="$1"; wt="$(abs "$2")"; start="${3:-}"
    [ ! -e "$wt" ] && [ ! -L "$wt" ] || die "$wt exists; refusing to overwrite it"
    git -C "$main" check-ref-format --branch "$branch" >/dev/null 2>&1 || die "'$branch' is not a valid branch name"
    if git -C "$main" show-ref --verify --quiet "refs/heads/$branch"; then
      [ -z "$start" ] || die "branch $branch exists; it is checked out as it is and takes no start point"
      git -C "$main" worktree add -q "$wt" "$branch" || exit 1
    else
      git -C "$main" worktree add -q -b "$branch" "$wt" "${start:-HEAD}" || exit 1
    fi
    worktree_link_node_modules "$main" "$wt"
    linked=""
    for d in preview playwright scripts/ts-tools; do [ -L "$wt/$d/node_modules" ] && linked="$linked $d"; done
    echo "lane-worktree: $wt on $branch at $(git -C "$wt" rev-parse --short HEAD); node_modules linked:${linked:- none}"
    ;;
  remove)
    [ $# -eq 1 ] || usage
    wt="$(abs "$1")"
    [ -e "$wt/.git" ] || die "$wt is not a worktree"
    [ "$(cd "$wt" && pwd -P)" != "$(cd "$main" && pwd -P)" ] || die "$wt is the main checkout"
    [ -z "$(git -C "$wt" status --porcelain)" ] || die "$wt has uncommitted or untracked changes; commit or discard them first"
    git -C "$main" worktree remove "$wt" || exit 1
    echo "lane-worktree: removed $wt (its branch is kept)"
    ;;
  *) usage ;;
esac
