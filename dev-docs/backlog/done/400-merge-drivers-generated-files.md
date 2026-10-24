---
id: 400
title: "Merge drivers for the generated backlog index and the durations table"
status: "LANDED. A committed .gitattributes maps dev-docs/backlog.md and dev-docs/backlog-done.md to…"
size: "S."
area: tooling
updated: 2026-10-24
---

# Row 400: merge drivers for the generated backlog index and the durations table

## Item

Every merge of a lane branch into a batch queue conflicted on `dev-docs/backlog.md` (the index `scripts/backlog-index.sh` generates from the row files) or `playwright/durations.json` (per-spec seconds with derived totals), because each lane changes the same table region or the same `meta` totals. The main loop resolved them by hand each time (batch 16: every lane merge).

## Status / trigger

LANDED. A committed `.gitattributes` maps `dev-docs/backlog.md` and `dev-docs/backlog-done.md` to `merge=backlog-index` and `playwright/durations.json` to `merge=durations`.

- `scripts/merge-driver-backlog-index.py` merges the text outside the generated block with `git merge-file` and the block by row id (the side that changed a row's line wins, ours when both did), sorts by id and recomputes the count, so its output equals a regenerated index. A driver runs before the working tree holds the merge (no `MERGE_HEAD`, the other side's row files are not checked out), so it cannot call the generator; it works from the three index versions, and `check-backlog-index.sh` still compares the result with the row files.
- `playwright/tools/durations-merge.mjs` merges per spec entry (changed side wins, ours when both changed), the other `meta` keys the same way, and recomputes `tests`, `files` and `total_seconds`; entries are sorted by code point in the driver and in `durations.mjs` (`localeCompare` made the committed order depend on the machine's ICU data).
- `scripts/setup-merge-drivers.sh` writes `merge.<name>.name/.driver` into the repo config (idempotent, `--check`, `--repo DIR`); `scripts/session-setup.sh` runs it first, so every remote container registers them, and worktrees share the repo config. The driver command prefers the script of the tree being merged and falls back to `git merge-file`, so a tree without the scripts merges as before instead of conflicting.
- `scripts/check-merge-drivers.sh` (a gate) checks the attributes, scripts and hook wiring, runs both self-tests, merges two scratch branches that each add a row file and a durations entry (clean, `backlog-index.py --check` green, table totals valid) and proves the same merge conflicts without the drivers.

A clone that never ran the setup merges the old way; the gate reports that and does not fail on it.

## Size

S.

## Detail

Rejected: regenerating the index from the working tree inside the driver (git runs the driver before the other side's row files exist there), and `merge=ours` plus a post-merge regeneration (leaves a stale index in the merge commit for the gate to find).
