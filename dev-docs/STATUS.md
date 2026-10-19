# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** Seeded 2026-10-18 (lane docs-split) from
[`research/next-round-plan-2026-10-15.md`](./research/next-round-plan-2026-10-15.md),
[`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and [`plan.md`](./plan.md).

Last updated: **2026-10-19**. Session branch: `claude/zealous-davinci-08se9a` (head `de7059ba` at seeding).
Preview site: https://mentalgear.github.io/shadcn-dioxus/

## Current round

**Round 1 of the backlog order: procedure, suite speed, test health** (started 2026-10-18 from `de7059ba`). Lanes, each
in its own worktree `/home/user/wt/<lane>` on `lane/<lane>`:

| Lane | Rows | Goal |
|---|---|---|
| lock | 221, 216, 92 | ticket queue for the build lock with merge gates first, a second build slot, mtime touch inside the scripts, `preview/build.rs` rerun line |
| suite-speed | 152a | measure where the suite time goes; release-build validation; sharding; changed-files spec selector; slow-spec budget gate |
| baseline-red | 242, 115 | the four specs red on the deployed baseline, the residual flakes |
| specs2 | 241, 164 | block-demo mirror, hydrated `goto` everywhere |
| docs-split | (docs) | this documentation map, `STATUS.md`, the backlog split into one file per row |

Until `/home/user/tgt/DEPLOY-DONE` exists, round-1 lanes take no build lock (the previous round is being validated and
deployed first).

## Merged, not yet deployed

The round planned in `next-round-plan-2026-10-15.md` (twelve lanes, incl. the follow-up mousefocus: mouse opens focus the first control, run 2026-10-18) is merged on
the session branch: rows 214, 215, 217 (partly), 219, 220, 222-236, 238, plus the follow-ups 239-244 recorded as new
rows. The full gates went red once on the merged head 88cd93c9 (check-attr-spread-collision, ScrollArea), fixed in
fafcb96b; its full gate run and the validation (agent on 88cd93c9) are in progress. Open before the deploy: the full-suite validation on the merged tree (classified against the recorded baseline failures), the
computed-style snapshot compared with N5, the registry install smoke, then PR, merge and the release deploy. Post
the site link with the main commit once the Pages publish succeeds.

## Deployed

The preview site at the link above; row 242 lists the specs that are red on the deployed baseline too. The
2026-10-18 round is not deployed yet (see above).

## Backlog numbers

88 open and 156 done rows at the split (see [`backlog.md`](./backlog.md)). The main loop's round-0 bookkeeping closes
18 more rows that the backlog review found done, duplicate or obsolete and narrows 10 partly-done ones.

## Next round

**Round 2, planned next round:** scroller 213 (Message Scroller API gaps); markdown 213 (the optional `markdown`
component, owner decisions E2: pulldown-cmark `=0.13.4`, images off by default); theme 239 (`all: unset` vs the
baselines) + 243 (hover fills) + 124/125 (chart nits); chat-nav 148. Then round 3 (overlay and interaction primitives:
focus 240, drawer 156, toast 138, carousel 237), 4 (virtualization and scrolling), 5 (docs and site), 6 (quality
sweeps, test-suite prune last), 7 (held items 136, 157). Full table with the reasons:
[`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md).

## Owner inputs pending

- iPhone re-test right after the deploy: rows 28 and 30, plus the touch-focus and keyboard-inset checklist of the
  2026-10-18 round (rows 226, 227).
- Decisions before round 1 ends: CI back on (rows 4, 33); upstream engagement and the upstream filings of rows 130,
  131, 98 (rows 6, 14: outward-facing, only with the owner's go); older WebKit (row 15); components beyond shadcn
  (row 16, then its own round).

## Rules in force

- Lanes run the FAST gates only (see [`lane-rules.md`](./lane-rules.md), "Gate procedure"); the main loop runs the one
  full `scripts/run-gates.sh` per merge batch on the merged tree, and nothing is pushed without that green on the exact
  commit. Green = `ALL GREEN`, exit 0.
- One global build lock; Playwright under its own lock; 4-5 lanes per batch.

## Second session: rounds 2 and 3 (owner split 2026-10-18)

Owner decision 2026-10-18 ("Split, plus queue"): session `session_01QoX93TwsKJ9GtMXWUXXfKw` takes rounds 2 and 3 of the
backlog order on its own branch `claude/rounds-2-3` (separate container, so its build locks are its own); this session
keeps rounds 0 and 1. Later rounds and new row ids are claimed on the `claims` branch (`CLAIMS.md`, fast-forward-only
pushes, so two sessions cannot claim the same round or mint the same row id). Each session PRs and merges its own
rounds to main after a full green; the second merger resolves shared files (`CLAUDE.md`, `scripts/run-gates.sh`, this page).

Rounds 2 and 3 are merged on `claude/rounds-2-3` (2026-10-19, PR #102): rows 213 (API items 1, 2, 4-9 and the
`markdown` component), 148, 239, 243, 124, 125 (round 2); 240, 138, 237, 156 (round 3); follow-ups found on the way and
landed in the same PR: 260 (hover mix space + gate), 262 and 263 (toast F6, pause, vertical swipe). Still open from these
rounds: row 213 item 10 (upstream diff), 261 (`--dx-font-mono` token), 264 (non-modal Dialog `aria-modal` and outside-press
switch). Next unclaimed round on the `claims` branch: 4.

Rounds 5 and 7 (claimed on `claims` 2026-10-19; 7 taken before 6 because its files were disjoint) run on
`claude/round-5`: rows 188 (demo-only items), 213 item 10 (upstream diff: new row 265), 162 and 261 (self-hosted Geist,
`--dx-font-mono` token), 165 (generated props tables + `check-props-docs.sh`), 137 (reactive direction, RTL toggle,
`check-direction-keys.sh`), 264 (non-modal `aria-modal`, `disable_pointer_dismissal`) and 157 (InputOtp `OtpPattern`)
landed there, and so did row 136 (Questionnaire, owner lifted the hold 2026-10-18). Round 6 (a sweep) follows.
The archived first session's unpushed round 1 work (152a spec budget/shard planner, two-slot build lock) and its round 4
claim stay with that session (owner chose to restore it, 2026-10-19).

Round 4 (owner moved it to its own project thread 2026-10-19; session_019CSAknzLi1PvcaAvfgpych, branch `claude/round-4`, claimed on `claims`): row 244 (`VirtualGrid`/`VirtualWrapGrid` on the shared engine, own docs page with an image gallery and a 10k x 50 data grid), row 186's remaining items (a Utilities section on `/docs` for scroll-fade and shimmer, `utilities.spec`; installing `dx-effects.css` alone still needs a CLI option), row 153 prototyped (opt-in `hidden="until-found"` find-in-page placeholders on windowed VirtualList; Ctrl+F find-next still needs a manual check per engine), row 174 closed on the owner's word. Round 6 (the sweep) stays last.

## Keeping this page current

Edit the section that changed, bump "Last updated", commit with the change that made it true. Row-level state belongs in
the row files under [`backlog/`](./backlog/), not here: this page says only which round runs, what is merged and
deployed, and what waits for the owner.
