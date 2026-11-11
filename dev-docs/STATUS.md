# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** What each batch contained is in `git log` and the
backlog row files, not here: this page says only what is deployed, what runs, what is next and what waits for the owner.

Last updated: **2026-11-11**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (still serving main `f38662dc`, PRs #166-#168, deploy commit
`85c05466`; release suite 4192 passed, 0 failed). **Main itself is ahead of the live site**: PR #170 (2026-11-11, the
2026-11-07 batch plus the icon-dedup and demo-title work below) and PR #171 (2026-11-11, a comment-analysis pass and
the paragraph below) are merged (main is at `b077881b`), full gate green (113/113), `registry-install-smoke` green
(83/83), but **the deploy is parked, not forced through**: the release suite on PR #170's head (`b78c231e`, before
PR #171's docs-only commit) failed two tests, neither confirmed real nor confirmed a flake (row
[506](./backlog/506-release-suite-two-failures-block-deploy.md)). The next session's first job is closing that row,
then deploying.

## Where a fresh session picks up

Session `session_01MsL6RCeq7QSe81nNBiuKP1` owns merges and deploys. Main carries PRs #170-#171 (`b077881b`, not yet
deployed; see row 506). A new
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

The no-base-path validation tree (`$LANE_TGT/release-template`) was removed on 2026-11-07 to free disk for lane builds;
the next deploy first rebuilds it with `scripts/lane-target.sh warm-release` (a cold release build).

## Merged and deployed

Merge queue batches 1-29 are on main (PRs #121-#155); PRs #156-#165 followed from session
`session_01MsL6RCeq7QSe81nNBiuKP1` and are deployed (main `d040554a`, release suite 4159 passed, 0 failed). PRs #166-#168 are deployed (main `f38662dc`, deploy `85c05466`, release suite 4192 passed, 0 failed). What each batch carried is in `git log` and the row files under
[`backlog/done/`](./backlog/done/).

## Next

The owner's order of 2026-11-07 (refining 2026-10-26): **phase 0.1 of the JavaScript plan (row 444) is finished as far as
its artefacts exist (on main, PR #167); before the plan continues, every row that would be needed whether or not the
port happens comes first**, then the rest of the plan, then the final review. Rows in one step are independent unless a
row says otherwise, so a step runs as parallel lanes (one verifier per batch, `lane-rules.md`).

**Before the plan continues (port-independent)**

1. **First: row 506** (two unconfirmed release-suite failures are parking the deploy of `b78c231e`). Nothing else in
   this step matters until the site is caught up to main.
2. Done, merged in PR #170 (2026-11-11): the deploy blockers of PR #168 (rows 468, 501); row **436**'s overlay and
   class groups (F1, F2, F4, F5, F6, F8, F9, F10, F11, F12), each reproduced red first (CHANGELOG.md has the
   user-visible ones: F1, F10, F11); **499** (safe triangle; CHANGELOG.md too); **152** slices 4-5; **371**'s
   dependency register gate (C23 enforced) and the demo-site icon deduplication (115 files moved off `dioxus-icons`
   onto the lifted central library; row 371's own stale-name fix too); **440**'s concurrent run-gates self-test;
   **486**'s pairing and resolved references; **399**'s `title.txt` override, the small-word case rule and its 78
   renames; **488** (no repro; RTL guard spec); **491** (name confirmed); **505**'s grant (PR #169) and the
   readable-name revert it covers.
   Done, merged in PR #171 (2026-11-11, on top of #170): **486**'s N1 rules and **507**'s first comment-analysis pass
   over `primitives` (two more stale names fixed; three follow-ups left open, below).
3. Open from row 436: one cancellable delayed-task helper and per-window state (F7, F3), and `menu_root::use_root_id`,
   which still writes a caller's `id` once (needs `use_reactive!`).
4. Open elsewhere: **504** (the hydrated Textarea shows no value; a fix is built on `lane/textarea-ssr-504`, local
   only, and needs the owner's call on a native-slot grant, see the row), **505**'s token-counting construction
   itself (the grant only covered the one readable-name revert), **371**'s remaining owner call (a D6 exception for
   `dioxus-icons`'s ~70 decorative-only demo icons vs. lifting those too), **152** (the prune continues on other
   files), **387** (the standing docs review), **507** (the standing comment-analysis backlog; pass 1's three
   follow-ups are open) and **496** (incremental gates, on an idle machine).
5. Owner and hardware: the iPhone checks (rows 028, 030, 295, 296, 305, 476, 477, 479, 480), Android (473), a desktop
   build (023), Firefox and Safari (153), and the owner's open questions (004/033 CI, 266 remote branches, 372 part 1,
   412-415, 469).

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
   Trigger on the plan's last step: **494** re-measures the `@timing` cases (deactivated since 2026-11-02, row 493) on a
   release build and re-enables the timing pass.

**Last**

10. **383**, the final adversarial review of the finished library. Parked behind the plan: **438** (Blitz renderer).

## Owner inputs pending

- Optional rows, each needing the owner's approval before anyone adopts or schedules it (owner 2026-10-24), not part of
  the adopted order: 412 (docs-site CSP meta tag), 413 (property tests and Miri), 414 (Kani/Aeneas proof spike), 415
  (GitHub Actions pinned to SHAs).
- For the measured ports (row 443): which devices the owner has (Mac, iPhone and iOS version, Windows or Android).
- Row 469: start or drop (pre-hydration typing: autofill, back/forward form restoration, fast typing on a slow phone).
- iPhone re-tests: rows 28, 30, 295 and 305 (307 is closed; 296 is fixed by 477, landed and deployed, and closes with 477 after the owner's iPhone check).
- Owner question from the Radix-versus-Base-UI scan (2026-10-28): (3) `as` (Radix `asChild` semantics, row 395) against
  base-nova's `render` prop for badge, item, sidebar, breadcrumb, attachment, bubble and marker; the main loop recommends
  keeping `as`.
- Decided 2026-10-28: Select defaults to item-aligned (row 478); overlays stay open while scrolling and Menubar does not
  lock (rows 477, 479).
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
