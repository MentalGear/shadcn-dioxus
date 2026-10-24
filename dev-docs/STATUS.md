# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-10-24**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `b8357ca8`, PR #140, merge queue batch 13).

## Merge queue

One session owns merges to main and deploys: the backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`). Lanes push a
fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the owner merges a batch of
4-5, runs ONE full `scripts/run-gates.sh` and the specs the batch's diff selects, merges it and deploys once, the
deploy running the full Playwright suite on a release build. Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue".

## Merged and deployed

Merge queue batches 1-13 are on main (PRs #121-#140), batch 13 as `b8357ca8`.

## In the queue

Batch 14 (`queue/batch-14`): row 375 (one page-wide scroll lock with a deferred final release), 376 (toast height follows
its content), 374 and 378 (one Chromium resolver; the Chromium 153 spec reds and the anchored-overlay fixes), 379
(session setup fetches every Cargo.lock), 380 (calendars read the visitor's clock after mount), 381 (one interactive
cursor option), Select item-aligned flip, nested Spinner sizes (188 leftovers). The cleanup lanes of row 266 (this
sweep: `gates.md`, the `rustdoc` gate, the CLAUDE.md trim, the archive moves) come after it.

## Running

Rows 309 (VirtualList and the grids on one engine; the grid `content-visibility` mode and find-in-page placeholders are
deferred), 325 (the heavy cross-page oracles) and 152 (suite review; the 152b prune goes last).

## Next

The cleanup sweep, in this order: **266** (repo cleanup: archive moves, doc trims, generated `gates.md`) -> **267**
(split oversized source files) -> **355** (workspace lint table and Rust conventions,
[`guides/rust-conventions.md`](./guides/rust-conventions.md)) -> **365** (final sweep: refactor and root-cause review),
then the 152b suite prune. Order and reasons: [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md).

## Owner inputs pending

- iPhone re-tests: rows 28, 30, 295, 296, 305, 307.
- Enable private vulnerability reporting on the repository ([`SECURITY.md`](../SECURITY.md) names it).
- Row 371 (dependency evaluation: keep, lift, drop) is parked by the owner (2026-10-23), no work until he reopens it.
- Remote branch deletion (merged and stale `lane/*`, `claude/*` branches): awaiting the owner's go; nothing is deleted
  without it.
- CI back on (rows 4, 33): every workflow except `mutants.yml` is deleted, so reactivation means a new workflow that
  runs `scripts/run-gates.sh`.

## Rules and decisions in force

[`decisions.md`](./decisions.md) lists the current owner decisions with their dates and rows;
[`lane-rules.md`](./lane-rules.md) is the lane brief (gate procedure, build lock, merge queue); the repo-wide rules are in
[`../CLAUDE.md`](../CLAUDE.md).

## Keeping this page current

Edit the section that changed, bump "Last updated", commit with the change that made it true. Row-level state belongs in
the row files under [`backlog/`](./backlog/), not here.
