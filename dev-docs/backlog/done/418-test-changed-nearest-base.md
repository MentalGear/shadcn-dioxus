---
id: 418
title: "test:changed diffs against the nearest base, not always origin/main"
status: "LANDED (lane/select-base-418). Trigger for reopening: test:changed selects far more specs than the…"
size: "S"
area: build
updated: 2026-10-25
---

# Row 418: test:changed diffs against the nearest base, not always origin/main

## Item

`pnpm run test:changed` diffs `origin/main...HEAD` by default. A lane branch cut from an unmerged merge-queue branch (`origin/queue/batch-N`) counts every queue commit as its own change, so the selector picks nearly the whole Playwright suite.

## Status / trigger

LANDED (lane/select-base-418). Trigger for reopening: `test:changed` selects far more specs than the lane's own commits explain, or the stderr base line names a ref the branch was not cut from.

## Size

S

## Detail

`playwright/tools/select-base.mjs` scores each candidate base (`origin/main` and every `refs/remotes/origin/queue/*` ref) by `git rev-list --count <merge-base>..HEAD` and picks the lowest, with ties to `origin/main`. `select.mjs` computes that base once and uses it for both the changed-file list and the rule-level CSS diff, and prints `select: base <ref> (<n> commits ahead)` on stderr. `--range` and `CHANGED_RANGE` still override it. `select-selftest.mjs` builds a throwaway repo with a bare origin, a three-commit queue branch and a one-commit lane on its tip, and asserts the picker chooses the queue ref at 1 commit, falls back to `origin/main` at 4 commits when no queue ref exists, and ties to `origin/main` at equal distance.
