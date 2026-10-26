# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-10-25**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `99407766`, PR #150, merge queue batch 24).

## Merge queue

One session owns merges to main and deploys: the backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`). Lanes push a
fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the owner merges a batch of
4-5, runs ONE full `scripts/run-gates.sh` and the specs the batch's diff selects, merges it and deploys once, the
deploy running the full Playwright suite on a release build. Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue".

## Merged and deployed

Merge queue batches 1-24 are on main (PRs #121-#150), batch 24 deployed as `99407766`.

## In the queue

Batch 26 (deployed 2026-10-26, main 360ecc92): rows 423, 422's outline, 348 (solid input edges), 393 (docs fixes), 424 (DatePicker `read_only`), 425 (HoverCard `force_mount`) and 439 (chart-prehydration font race). Batch 27 (deployed 2026-10-26, main 92ba1034; PR #153; release suite 3923 passed, smoke 82/82): the first refactors from the row 365 sweep. 429 (one CSS walker and allowlist engine for the gates), 430 (generated component manifests), 427 (spec page and read kinds as helpers), 431 (one test harness, `use_controlled_state`), 428 (wrappers forward with `..props`) and 426 (the theme owns the scrim and visually-hidden recipes), plus 437 (no early-exit reader after a pipe under pipefail) and a conflict-marker gate. Batch 28 (deployed 2026-10-26, main daf90a8f; PR #154; release suite 3925 passed, smoke 82/82): rows 432, 433 and 441, row 434 step 2 lanes 2A to 2E (focus, scroll, measure, head sheets, toast pause and demo sleeps in Rust; 2D kept the scroll bridge), the shrink-only JS-volume ratchet and `js-interop.md`, and rows 440/441 filed. Batch 29 (`queue/batch-29`): the bridge (row 434 lane 3.0) and the step 3 plan folded through adversarial board round 5, backlogged as rows 444-471 (444-467 await the owner's approval and scope; 468-471 are bugs the board found). The paper review ended at the owner's cap of five rounds: the outer construct held in all five, admission and slot ownership are proven first by phase 0.

## Running

Stage 3 (structure): 267 and 384 landed; 382 landed, 355 is in batch 25. Stage 4: 371 landed (the icon lift deferred).

## Next

Owner order of 2026-10-24 (backlog thread), each stage reading the finished output of the one before:

1. Open bugs: toast stack jump with mixed toast types (owner report 2026-10-25, toast thread reproducing).
2. API shape, pre-1.0: **396**, **395**, **398**, **391**.
3. Structure: **267** with **384**, at a quiet point in the queue; then **355** and **382**.
4. Dependency review **371** (outcomes to the owner).
5. Refactors from the **365** sweep (owner approved all ten 2026-10-25): rows **426-435**, small first, **435** (one menu engine, P10a first) last. Then the code-quality skill review **436** (owner: after the refactors).
6. Docs on the settled API: **393**, **392**, **394**, **422** (Basic / Component-Specific section groups, outline approved), **399** (now with demo renames and the focus-target readout), then the standing review **387**.
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
