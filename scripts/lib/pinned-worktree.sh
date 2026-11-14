#!/usr/bin/env bash
# Sourced by scripts/gate-at.sh and scripts/deploy-preview.sh: a PRIVATE detached worktree of one commit, so that nothing that
# happens in the shared checkout while a long run is in progress (a merge, a branch switch and back, a stash and pop, an edit)
# can change the tree the run reads (backlog rows 221 and 513).
#
#   pinned_worktree_add REPO COMMIT WT     removes an old WT of that name, checks COMMIT out detached into WT, links the
#                                          checkout's node_modules directories (untracked, shared) into it
#   pinned_worktree_remove REPO WT         removes it
# The worktree holds the tracked files of COMMIT and nothing else, so an untracked or ignored file of the shared checkout cannot
# reach a run that happens in it.
pinned_worktree_add() {
  local repo="$1" commit="$2" wt="$3"
  git -C "$repo" worktree remove --force "$wt" >/dev/null 2>&1 || true
  git -C "$repo" worktree add -q --detach "$wt" "$commit" || return 1
  local d
  for d in preview playwright scripts/ts-tools; do
    [ -d "$repo/$d/node_modules" ] || continue
    [ -e "$wt/$d/node_modules" ] && continue   # already there (a tracked link): never link into a directory
    ln -sfn "$repo/$d/node_modules" "$wt/$d/node_modules"
  done
}
pinned_worktree_remove() {
  git -C "$1" worktree remove --force "$2" >/dev/null 2>&1 || true
}
