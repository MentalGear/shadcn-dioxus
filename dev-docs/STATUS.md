# Status: where we are

**Living page. The main loop updates it at every merge batch, deploy and round change; if the date below is old, treat
the page as stale and check `git log` and the flag files named here.** Seeded 2026-10-18 (lane docs-split) from
[`research/next-round-plan-2026-10-15.md`](./research/next-round-plan-2026-10-15.md),
[`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and [`plan.md`](./plan.md).

Last updated: **2026-10-22**. Several sessions work in parallel; who holds what is on the `claims` branch (`CLAIMS.md`).
Preview site: https://mentalgear.github.io/shadcn-dioxus/ (deployed from main `ac99f4e8`, PR #132, merge queue batch 5; batch 6 is this commit's PR).

## Merge queue (owner decision 2026-10-21)

One session owns merges to main and deploys: the backlog thread (`session_01QoX93TwsKJ9GtMXWUXXfKw`). Lanes push a
fast-gated branch and a QUEUE row on the `claims` branch; the queue owner merges a batch, runs ONE full
`scripts/run-gates.sh` and ONE full Playwright suite (`playwright/tools/run-suite.sh`) on the merged tree, opens one PR,
merges it after the project coordinator's OK and deploys once. Procedure: [`lane-rules.md`](./lane-rules.md#merge-queue-row-321-owner-decision-2026-10-21).

## Merged and deployed

- Batch B (PR #121), the questionnaire fixes (#123, #124).
- Batch A (PR #125, batch-A thread): rows 253, 254, 255-259, 268, the 291/294 tails, 297-303, 320 (cargo-gate reuse), 324.
- Queue batch 1 (PR #126): the dynamic-style gate merge (#124), rows 327 (all-routes once), 329 (SSR parity test), 321
  (queue docs), 330 (verdict: the `@layer` policy layer does not hold), a stale Select spec.
- Queue batch 2 (PR #127): the 291/294 leftovers, rows 323 (CSS-aware spec selector + `check-select-css`), 328 part 1,
  331 (sweep shrinks, about -435 worker-s), 258, 334-339 (docs), the text-editor research; suite reds fixed as rows 345
  (carousel spec read mid-fade), 346 (Popover focus-return spec race), 347 (message-scroller spec race; product edge
  open) and a chart-tooltip `line-height`.
- Queue batch 3 (PR #128): row 328's construction, one zero-specificity component-root reset on
  `[data-dx-root]` (`check-component-root-reset.sh` replaces `check-declared-typography`).
- Queue batch 4 (PR #131): the coarse sizing switch (`data-dx-coarse`, `check-coarse-sizing.sh`, rows 349, 350), docs
  site fixes (rows 351-354: RTL demos last, MDN link, carousel Next fade, language switcher removed), first-paint poses,
  SSR parity for tag group and resizable, test tooling, the fast-loop docs (row 326).
- Queue batch 5 (PR #132): Select item-aligned position (268), NavigationMenuIndicator (249), the DataTable model and
  component (340), message scroller follow release and demos (347, 335); row 299 closed.
- Queue batch 6 (this commit's PR): one `VirtualizeMode` in the virtual engine and a windowed `TableRows` body (343),
  whole-site sweeps about -20% worker-seconds (334, follow-up 358), the lane disk guard as a reservation (356; 357 was
  already fixed), InputGroupTextarea, the custom-control contract and a breadcrumb router link (188 leftovers).

Earlier: Rounds 2-3 (PR #102), Rounds 5 and 7 (PR #105), the features batch, Round 4 (see below).

## Owner decisions (2026-10-21)

The merge queue is the backlog thread's; cargo-gate reuse is on; only the warm release validation tree is kept; the
library keeps to primitives and shadcn element parity, so no Markdown or rich-text editor is built (row 337 closed as out
of scope; the rule is in [`backlog.md`](./backlog.md#scope)); `--dx-root-line-height` defaults to `normal` (Tailwind apps set
1.5); the `theme-all-reset` golden is kept (row 331). The hold on new backlog rows is lifted (2026-10-21); the
owner said to continue with the backlog until done, skipping rows that wait on his iPhone re-tests or decisions.

## Backlog numbers

46 open and 285 done row files on 2026-10-22 (see [`backlog.md`](./backlog.md)).

## Next round

The Round 6 quality sweep (167 listeners, 170 cross-component overrides, 218 popup isolation, then the 152b suite prune
last) and the repo cleanup (row 266) wait until every component and feature row has landed (owner decision
2026-10-19). Full order: [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md).

## Owner inputs pending

- iPhone re-test of press feedback on touch taps (row 295).
- iPhone re-test right after the deploy: rows 28 and 30, plus the touch-focus and keyboard-inset checklist of the
  2026-10-18 round (rows 226, 227).
- Decisions: CI back on (rows 4, 33); older WebKit (row 15); the preview-card light tint (row 285); radial tooltips
  (row 304); ai-elements item 3 (row 265). Decided 2026-10-22: no upstream engagement (rows 6 and 14 closed; nothing
  from rows 130, 131, 98 is filed upstream).

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

Round 4 (owner moved it to its own project thread 2026-10-19; session_019CSAknzLi1PvcaAvfgpych, branch `claude/round-4`, claimed on `claims`): row 244 (`VirtualGrid`/`VirtualWrapGrid` on the shared engine, own docs page with an image gallery and a 10k x 50 data grid), row 186's remaining items (a Utilities section on `/docs` for scroll-fade and shimmer, `utilities.spec`; installing `dx-effects.css` alone still needs a CLI option), row 153 prototyped (opt-in `hidden="until-found"` find-in-page placeholders on windowed VirtualList; Ctrl+F find-next still needs a manual check per engine), row 174 closed on the owner's word. Round 6 (the sweep) stays last. Follow-ups merged: VirtualGrid stylesheet link (#110), edge scroll chaining (#115, row 308); then row 309: VirtualList and VirtualGrid share one engine, plus `VirtualJustifiedGrid` (gapless, uncropped justified rows) and a Grid/Justified switch on the photo wall.

## Keeping this page current

Edit the section that changed, bump "Last updated", commit with the change that made it true. Row-level state belongs in
the row files under [`backlog/`](./backlog/), not here: this page says only which round runs, what is merged and
deployed, and what waits for the owner.
