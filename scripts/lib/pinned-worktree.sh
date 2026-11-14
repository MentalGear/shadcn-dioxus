#!/usr/bin/env bash
# Sourced by scripts/gate-at.sh and scripts/deploy-preview.sh: a PRIVATE detached worktree of one commit, so that nothing that
# happens in the shared checkout while a long run is in progress (a merge, a branch switch and back, a stash and pop, an edit)
# can change the tree the run reads (backlog rows 221 and 513).
#
#   pinned_worktree_add REPO COMMIT WT     replaces a worktree already at WT, checks COMMIT out detached into WT, links the
#                                          checkout's node_modules directories (untracked, shared) into the directories of the
#                                          commit that have them; returns 0 when the checkout succeeded
#   pinned_worktree_remove REPO WT         removes it
#   worktree_link_node_modules REPO WT     the link step alone, shared with scripts/lane-worktree.sh (lane worktrees on a branch)
# The worktree holds the tracked files of COMMIT and nothing else, so an untracked or ignored file of the shared checkout cannot
# reach a run that happens in it. Callers set their cleanup trap BEFORE the add: a failed add can leave a directory behind.
# Links REPO's node_modules directories (untracked, shared) into the same directories of WT. A directory the worktree's commit
# does not have gets no link, a directory that already has node_modules (a tracked link) keeps it, and a REPO without the
# node_modules gives nothing to link. The links are invisible to git: .gitignore's `node_modules` line matches a symlink too.
worktree_link_node_modules() {
  local repo="$1" wt="$2" d
  for d in preview playwright scripts/ts-tools; do
    if [ -d "$wt/$d" ] && [ -d "$repo/$d/node_modules" ] && [ ! -e "$wt/$d/node_modules" ]; then ln -sfn "$repo/$d/node_modules" "$wt/$d/node_modules"; fi
  done
}
pinned_worktree_add() {
  local repo="$1" commit="$2" wt="$3"
  git -C "$repo" worktree remove --force "$wt" >/dev/null 2>&1 || true
  git -C "$repo" worktree add -q --detach "$wt" "$commit" || return 1
  worktree_link_node_modules "$repo" "$wt"
  return 0
}
pinned_worktree_remove() {
  git -C "$1" worktree remove --force "$2" >/dev/null 2>&1 || true
}
