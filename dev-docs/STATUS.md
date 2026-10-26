# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-10-26**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `4afa18c2`, PR #155, merge queue batch 29).

## Where a fresh session picks up

Nothing runs and nothing is in the merge queue: every lane of the last session landed in batch 29 and is deployed. A new
session reads this page, then takes the first open row of "Next" below; each row file under [`backlog/`](./backlog/)
says what to build, which files, its proof and its size. The rules are [`../CLAUDE.md`](../CLAUDE.md) and
[`lane-rules.md`](./lane-rules.md); the owner's decisions are [`decisions.md`](./decisions.md).

## Merge queue

One session owns merges to main and deploys (the owner names it; the backlog thread did it through batch 29). Lanes push
a fast-gated branch (`scripts/run-gates.sh --fast`) and a QUEUE row on the `claims` branch; the queue owner merges a
batch of 4-5 into `queue/batch-N`, runs ONE full `scripts/run-gates.sh`, the release suite and
`scripts/registry-install-smoke.sh` on the merged tree, opens and merges the PR, and deploys once
(`scripts/deploy-preview.sh`, which runs the full Playwright suite on a release build). Procedure:
[`lane-rules.md`](./lane-rules.md), "Merge queue".

## Merged and deployed

Merge queue batches 1-29 are on main (PRs #121-#155); batch 29 is deployed (main `4afa18c2`, release suite 3945
passed, install smoke 82/82). What each batch carried is in `git log` and the row files under
[`backlog/done/`](./backlog/done/).

## Next

The owner's order of 2026-10-26: **every row that can be done before the JavaScript plan comes first**, then the plan,
then the final review. Rows in one step are independent unless a row says otherwise, so a step runs as parallel lanes.

**Before the JavaScript plan**

1. Bugs and approved fixes (owner 2026-10-26), each reproduced first: **468** (form reset desync), **474** (Input OTP
   paste), **473** (date picker segments), **471** and **475** (desktop only; reasoned from code, since the sandbox has no
   desktop build), **390** (theme preset split; the one item that changes CSS users write), **296** (iPhone Combobox drift:
   place it the way Menubar and Popover are placed), **476** (iPhone checkbox and radio tap lag on the home page) and
   **472** (chart tooltip fade). **469** only on the owner's word.
2. Structure: **435** (one menu engine behind DropdownMenu, ContextMenu and Menubar; before the plan, so the overlay port,
   row 456, ports one engine instead of three), then **436** (code-quality review of the refactored code).
3. **371** (dependency review; each outcome goes to the owner before it is applied), **440** (per-run gate log
   directory) and **266** part (c) (`cargo public-api` report with a diff gate).
4. Docs on the settled API: **392**, **394**, **399**, **422**, then the standing review **387**.
5. Tests: the **152** suite prune (before the plan adds its specs).
6. At any point: **153**, **268**, **170**.

**The JavaScript plan** (row **434**; approved 2026-10-26 with scope 2). The whole plan is recorded, not only its first
step: the decided plan is [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) (with
[`research/sync-js-residue-2026-10-26.md`](./research/sync-js-residue-2026-10-26.md) for the three feel-critical
ports), its working form is [`js-interop.md`](./js-interop.md), and every item has a row:

7. Phase 0, the construct and two proof ports: **444** (gates), **445** (legacy freeze), **446** (witness harness),
   **447** (order runtime), **448** (proof port A), **449** (proof port B), **450** (bridge stats and the desktop latency
   session); about 111 h, nearly all one chain through `ts/bridge.ts`, so it runs mostly serially.
8. The ports, as ordinary rows by yield (lines of JavaScript removed per hour, highest first; a row whose dependencies
   have not landed waits, and each row names them): **451** (3.SC, 27-30/h), **452** (3.PL, 27/h), **453** (3.PV, 22/h),
   **454** (3.OB, 21/h), **455** (3.GS, 15/h), **456** (3.OV, 9/h), **457** (3.VS, 6/h), **458** (3.FC, 5/h), **461**
   (3.PW) and **459** (3.SL, which closes a blocking-listener class rather than removing lines); **460** (3.HV) and
   **462** (roving focus) only if phase 0's witnesses call for them.
9. The measured group, one port at a time with the old version switchable, the owner measuring on real devices after
   each: **442** (desktop proof harness) and **443** (measurement kit), **463** (freeze), **464** (carousel), **465**
   (message scroller), **466** (wheel band), then **467** (remove the old scripts per the owner's verdict).

**Last**

10. **383**, the final adversarial review of the finished library. Parked behind the plan: **438** (Blitz renderer).

## Owner inputs pending

- Optional rows, each needing the owner's approval before anyone adopts or schedules it (owner 2026-10-24), not part of
  the adopted order: 412 (docs-site CSP meta tag), 413 (property tests and Miri), 414 (Kani/Aeneas proof spike), 415
  (GitHub Actions pinned to SHAs).
- For the measured ports (row 443): which devices the owner has (Mac, iPhone and iOS version, Windows or Android).
- Row 469: start or drop (pre-hydration typing: autofill, back/forward form restoration, fast typing on a slow phone).
- iPhone re-tests: rows 28, 30, 295 and 305 (307 is closed; 296 is in Next).
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
