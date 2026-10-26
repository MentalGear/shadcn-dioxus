# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-10-26**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `4afa18c2`, PR #155, merge queue batch 29).

## Merge queue

One session owns merges to main and deploys: the backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`). Lanes push a
fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the owner merges a batch of
4-5, runs ONE full `scripts/run-gates.sh` and the specs the batch's diff selects, merges it and deploys once, the
deploy running the full Playwright suite on a release build. Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue".

## Merged and deployed

Merge queue batches 1-29 are on main (PRs #121-#155), batch 29 deployed as `4afa18c2`.

## In the queue

Batch 26 (deployed 2026-10-26, main 360ecc92): rows 423, 422's outline, 348 (solid input edges), 393 (docs fixes), 424 (DatePicker `read_only`), 425 (HoverCard `force_mount`) and 439 (chart-prehydration font race). Batch 27 (deployed 2026-10-26, main 92ba1034; PR #153; release suite 3923 passed, smoke 82/82): the first refactors from the row 365 sweep. 429 (one CSS walker and allowlist engine for the gates), 430 (generated component manifests), 427 (spec page and read kinds as helpers), 431 (one test harness, `use_controlled_state`), 428 (wrappers forward with `..props`) and 426 (the theme owns the scrim and visually-hidden recipes), plus 437 (no early-exit reader after a pipe under pipefail) and a conflict-marker gate. Batch 28 (deployed 2026-10-26, main daf90a8f; PR #154; release suite 3925 passed, smoke 82/82): rows 432, 433 and 441, row 434 step 2 lanes 2A to 2E (focus, scroll, measure, head sheets, toast pause and demo sleeps in Rust; 2D kept the scroll bridge), the shrink-only JS-volume ratchet and `js-interop.md`, and rows 440/441 filed. Batch 29 (deployed 2026-10-26, main 4afa18c2; PR #155; release suite 3945 passed, smoke 82/82): the bridge (row 434 lane 3.0), the step 3 plan folded through all seven adversarial board rounds and backlogged as rows 444-467 (approved 2026-10-26, scope 2), the bug rows the boards found (468, 469, 471, 473-475), the carousel touch fix (row 470: a press on the track stops autoplay) and row 472. The paper review is closed: converged in substance, the outer construct held in all seven rounds and rounds 6-7 only removed mechanisms ([round 7 synthesis](./research/board-2026-10-26-r7/SYNTHESIS.md)).

Nothing is in the queue.

## Running

Nothing. The owner moved all further work to a fresh session (2026-10-26).

## Next

In this order (owner 2026-10-26: "We will do it all in a fresh session"):

1. Approved bugs and fixes, independent of the JavaScript plan, each reproduced first: **468**, **471**, **473**, **474**, **475**, **390** (theme preset split), the iPhone rows **296** (Combobox drift; place it the way Menubar and Popover are placed) and **476** (home-page checkbox and radio tap lag), and **472** (chart tooltip fade). **469** waits on the owner.
2. The JavaScript plan (row **434**, rows **444-467**), approved 2026-10-26 with scope 2: phase 0, rows **444-450** (about 111 h, nearly all one chain through `ts/bridge.ts`), then the ports as ordinary rows in order of JavaScript removed per hour; the measured group (442, 443, 463-467) one port at a time, the owner measuring after each. The plan is [`js-interop.md`](./js-interop.md) and [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md).
3. Then the earlier order's remainder: **435** (one menu engine) and **436** (code-quality review); **371** (dependency review); docs **392**, **394**, **399**, **422**, then the standing review **387**; the **152** suite prune; **383** last. Independent at any point: 153, 268, 170.

## Owner inputs pending

- Optional rows, each needing the owner's approval before anyone adopts or schedules it (owner 2026-10-24), not part of
  the adopted order: 412 (docs-site CSP meta tag), 413 (property tests and Miri), 414 (Kani/Aeneas proof spike), 415
  (GitHub Actions pinned to SHAs).
- For the measured ports (row 443): which devices the owner has (Mac, iPhone and iOS version, Windows or Android).
- Row 469: start or drop (pre-hydration typing: autofill, back/forward form restoration, fast typing on a slow phone).
- iPhone re-tests: rows 28, 30, 295, 305 (307 closed on the owner's re-test; 296 re-tested, still drifting, now in Next).
- Enable private vulnerability reporting on the repository ([`SECURITY.md`](../SECURITY.md) names it).
- Remote branch deletion: the owner said go (2026-10-26), but this environment's proxy refuses deletes; the owner runs
  the command from their own machine (`/mnt/project-files/handoff/merged-remote-branches-2026-10-26.txt`, 262 merged
  branches; the 30 unmerged ones are kept).
- CI back on (rows 4, 33): every workflow except `mutants.yml` is deleted, so reactivation means a new workflow that
  runs `scripts/run-gates.sh`.

## Rules and decisions in force

[`decisions.md`](./decisions.md) lists the current owner decisions with their dates and rows;
[`lane-rules.md`](./lane-rules.md) is the lane brief (gate procedure, build lock, merge queue); the repo-wide rules are in
[`../CLAUDE.md`](../CLAUDE.md).

## Keeping this page current

Edit the section that changed, bump "Last updated", commit with the change that made it true. Row-level state belongs in
the row files under [`backlog/`](./backlog/), not here.
