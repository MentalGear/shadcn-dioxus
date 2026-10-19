# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** Seeded 2026-10-18 (lane docs-split) from
[`research/next-round-plan-2026-10-15.md`](./research/next-round-plan-2026-10-15.md),
[`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and [`plan.md`](./plan.md).

Last updated: **2026-10-19**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `404e7175`, PR #106).

## Current work (2026-10-19)

| Lane holder | Work |
|---|---|
| backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`, branch `claude/features-1`) | the features group: menus (155 g2 + Command shortcuts), calendar (155 g8, DOB layouts, num-integer), parts (188/187 component parts, Select scroll), chatkit (265: CodeBlock, Trace, Checkpoint), hygiene (217 sleep/strum, row 5 rule 8), snapshot (245, 246), combobox (187 multi-select); plus the bookkeeping close of rows already done |
| earlier session (`session_01PJCSoqAo9bCWeVLpz61Wdy`) | Round 1 (lock 221/216/92, suite speed 152a, baseline reds 242/115, specs 241/164), redone after a container restart lost its unpushed work |
| Round 4 thread | VirtualGrid 244, find-in-page 153, home scroll jank 174, utilities 186 |
| docs-review thread | the owner's docs-site review (rows 285-289) |
| cleanup thread | repo cleanup plan (row 266), runs with the Round 6 sweep |

## Merged and deployed

Rounds 2-3 (PR #102), Rounds 5 and 7 (PR #105: rows 188 demo items, 213, 162, 261, 264, 165, 157, 137, 136), the
card-overflow fix (PR #104, row 280); preview redeployed from `404e7175` (PR #106).

## Features batch (2026-10-19)

Merged lanes: hygiene (row 5 landed: the `form` attribute on the Switch/RadioGroup/Checkbox hidden inputs; one primitives
sleep helper), menus (theme `data-dx-menu-*` hoist, `CommandShortcut`), calendar (`caption_layout`, ISO week numbers, cell
size, date of birth; `num-integer` and `dioxus-sdk-time` removed), chatkit (CodeBlock, Trace, Checkpoint, agent chat demos;
row 265 partial, item 3 is the owner's decision), snapshot (rows 245/246 landed: timer-drain opener, skip-link exclusion,
element-level classifier `scripts/snapshot-classify.mjs`, format 4), combobox (`ComboboxMulti` with chips and clear, row 187
item 6), parts (typed AlertDialog size/media/action variant, Card size, Spinner size, Button radius; rows 187/188 updated;
the Select scroll buttons and item-aligned position split out to row 268). `strum` is also gone from preview.

## Backlog numbers

45 open and 211 done rows on 2026-10-19 (see [`backlog.md`](./backlog.md)), after the bookkeeping pass closed 29 rows
whose status already said done.

## Next round

The Round 6 quality sweep (167 listeners, 170 cross-component overrides, 218 popup isolation, then the 152b suite prune
last) and the repo cleanup (row 266) wait until every component and feature row has landed (owner decision
2026-10-19). Full order: [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md).

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
