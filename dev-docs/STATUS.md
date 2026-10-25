# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-10-25**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `f9860a21`, PR #148, merge queue batch 21).

## Merge queue

One session owns merges to main and deploys: the backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`). Lanes push a
fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the owner merges a batch of
4-5, runs ONE full `scripts/run-gates.sh` and the specs the batch's diff selects, merges it and deploys once, the
deploy running the full Playwright suite on a release build. Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue".

## Merged and deployed

Merge queue batches 1-21 are on main (PRs #121-#148), batch 21 deployed as `f9860a21`.

## In the queue

Batch 22 (`queue/batch-22`): row 267, six oversized files split into module directories (public paths unchanged).

## Running

Stage 3 (structure): row 267 is in batch 22. Next 384, then 355 and 382.

## Next

Owner order of 2026-10-24 (backlog thread), each stage reading the finished output of the one before:

1. Open bugs: 397.
2. API shape, pre-1.0: **396**, **395**, **398**, **391**.
3. Structure: **267** with **384**, at a quiet point in the queue; then **355** and **382**.
4. Dependency review **371** (outcomes to the owner).
5. One review pass: the code-quality review sweep together with **365**.
6. Docs on the settled API: **393**, **392**, **394**, **399**, then the standing review **387**.
7. The 152b suite prune, then **383** last.

Independent, in parallel at any point: 153, 268, 170.

## Owner inputs pending

- Optional rows, each needing the owner's approval before anyone adopts or schedules it (owner 2026-10-24), not part of
  the adopted order: 412 (docs-site CSP meta tag), 413 (property tests and Miri), 414 (Kani/Aeneas proof spike), 415
  (GitHub Actions pinned to SHAs).
- iPhone re-tests: rows 28, 30, 295, 296, 305, 307.
- Enable private vulnerability reporting on the repository ([`SECURITY.md`](../SECURITY.md) names it).
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
