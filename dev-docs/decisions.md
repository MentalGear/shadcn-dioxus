# Owner decisions in force

An index, one line per decision: the decision, its date and the row (or doc) that carries the reasoning. **Current
decisions only**: one that is reversed or superseded leaves this page and stays in its row. A decision lives in its row;
the line here is the pointer, so when a row's text and this line disagree, the row wins and this line is fixed.

## Scope and product

- The library keeps to primitives and shadcn element parity, the chat kit included; a Markdown or rich-text editor, or
  any item that needs its own state management, is out of scope and closed as such. 2026-10-21 ([`backlog.md`](./backlog.md), row [337](./backlog/done/337-text-editor-component-rich-text-markdown-composer.md)).
- Where shadcn has a pattern, follow shadcn: ToggleGroup gets `spacing` (default 2), Popover is non-modal by default.
  2026-10-10, 2026-10-13 (rows [155](./backlog/done/155-shadcn-parity-wave-group-1-the-invalid-ring.md), [169](./backlog/done/169-open-owner-question-should-popover-be-non-modal.md)).
- `VirtualList` and `VirtualGrid` stay two components (list and grid ARIA semantics differ) on one shared engine;
  `VirtualJustifiedGrid` packs variable-aspect items into gapless rows on it. 2026-10-20 (row [309](./backlog/done/309-virtuallist-and-virtualgrid-share-one-engine.md)).
- The upstream Dioxus colour ramp is deleted; theme fallbacks are shadcn Neutral. 2026-10-23 (row [373](./backlog/done/373-theme-defaults-are-shadcn-neutral.md)).
- The theme owns the baseline once: `--dx-root-line-height` defaults to `normal` and an app sets it (Tailwind apps set
  1.5); the interactive-element cursor is one theme option, `data-dx-interactive-element-cursor`. 2026-10-21 (row [328](./backlog/done/328-audit-2-declared-typography-every-text-or-icon.md)),
  2026-10-24 (row [381](./backlog/done/381-interactive-element-cursor-option.md)).
- Press feedback stays on for every pointer, and components that miss it are fixed. 2026-10-19 (row [295](./backlog/295-press-feedback-on-touch-taps-already-on-by.md)).
- Older WebKit and iOS momentum scrolling are not targets; old engines keep their native behaviour. 2026-10-23 (row [15](./backlog/done/015-scrollbar-gutter-on-older-webkit-ios-momentum.md)).
- RTL demos come last in a component's demo list; the docs header has no language switcher; the docs preview card keeps
  its near-white light tint (a docs-page detail). 2026-10-20, 2026-10-21, 2026-10-22 (rows [351](./backlog/done/351-rtl-demos-always-come-last.md), [354](./backlog/done/354-remove-the-docs-header-language-switcher.md), [285](./backlog/done/285-docs-preview-card-light-mode-tint.md)).
- No upstream engagement: local fixes and workarounds stay, nothing is filed with Dioxus or the forks. 2026-10-22 (rows [6](./backlog/done/006-upstream-engagement-where-fixes-land-open.md), [14](./backlog/done/014-nudge-dioxus-framework-issues-2962-onclose-4319.md)).

- JavaScript: the row 434 step 3 plan is approved with scope 2: the construct and two proof ports first (phase 0, rows
  444-450), then every other port as an ordinary row in order of JavaScript removed per hour; the uncertain ports go last,
  one at a time, with the old version switchable so the owner measures each. 2026-10-26 (row [434](./backlog/434-javascript-in-typescript-modules-not-rust-strings.md), [`js-interop.md`](./js-interop.md)).

## Dependencies and supply chain

- Markdown parsing is an optional `markdown` component with `pulldown-cmark` pinned `=x.y.z`, default features off, only
  tables, strikethrough and task lists, a closed node tree and `SafeHref`; no other component may list a parser.
  2026-10-15 (row [213](./backlog/done/213-message-scroller-parity-with-the-original-folds.md), [`check-markdown-parser.sh`](./gates.md), [`check-cargo-dep-versions.sh`](./gates.md)).
- pnpm in `playwright/` and `preview/`: every direct dependency pinned exactly, a minimum release age of 30 days, no
  dependency install script. 2026-10-22 (row [359](./backlog/done/359-js-tooling-pnpm-minimum-release-age.md)).
- Supply-chain scope: SECURITY covers everything that builds or touches the project; performance budgets are measured on
  the closure users compile. 2026-10-23 ([`supply-chain/README.md`](./supply-chain/README.md), row [217](./backlog/done/217-dependency-audit-follow-ups-what-users-pay-for.md)).
- The dependency evaluation (keep, lift, drop) is parked until the owner reopens it. 2026-10-23 (row [371](./backlog/371-dependency-evaluation-keep-lift-drop.md)).

## Process, gates and the merge queue

- CI is frozen: every workflow is `workflow_dispatch`-only, and only `mutants.yml` remains; `scripts/run-gates.sh` is the
  gate. 2026-08-30, workflows deleted 2026-10-24 (rows [33](./backlog/033-3-browser-playwright-on-every-push-pr-sharded.md), [4](./backlog/004-first-3-browser-ci-run-treat-as-calibration-not.md), [266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md)).
- Lanes run the fast gates; the main loop runs one full gate run per merge batch; the build lock is first-come,
  first-served with merge gates first; 4-5 lanes per batch. 2026-10-18 ([`lane-rules.md`](./lane-rules.md), row [221](./backlog/done/221-infra-session-process-lessons-from-the-2026-10.md)).
- Nothing lives only in the container: lanes push after every commit, briefs live in the repo. 2026-10-19 (rows [250](./backlog/done/250-container-reset-loses-unpushed-work.md), [221](./backlog/done/221-infra-session-process-lessons-from-the-2026-10.md)).
- One session owns merges to main and deploys, in batches; the deploy runs the full Playwright suite on a release build
  once. 2026-10-21 ([`lane-rules.md`](./lane-rules.md), rows [321](./backlog/done/321-one-merge-queue-owner-for-main-batch-one.md), [322](./backlog/done/322-run-the-full-release-suite-per-deploy-not.md)).
- Cargo gates are reused when their inputs are byte-identical; only the warm release validation tree is kept.
  2026-10-21 (rows [320](./backlog/done/320-skip-the-cargo-gates-when-the-rust-tree.md), [324](./backlog/done/324-keep-warm-release-target-trees-for-validation-and.md)).
- The fast loop: iterate on `dx serve`, verify with the diff-based `test:changed`, leave the full suite to the batch.
  2026-10-21 (row [326](./backlog/done/326-lanes-iterate-with-dx-serve-hot-reload-and.md)).
- Change only what a measurement shows holds up (test-speed and gate-speed work). 2026-10-21 (rows [331](./backlog/done/331-audit-5-delete-the-sweeps-that-a-static.md), [320](./backlog/done/320-skip-the-cargo-gates-when-the-rust-tree.md)).
- Sonnet and Haiku subagents do lane work; never substitute Opus for them; the evidence log of `agent-model-fit.md` is
  closed. 2026-10-20, 2026-10-24 ([`agent-model-fit.md`](./agent-model-fit.md), row [266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md)).
- The docs site is built from the library's own components (no native `details`/`summary`). 2026-10-19
  ([`check-docs-own-components.sh`](./gates.md)).
- Code comments describe only the current state; history is linked to its row, not retold. 2026-10-20
  ([`history-in-comments.md`](./history-in-comments.md), row [266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md)).
- The repo cleanup (archive, trim, generated `gates.md`, `decisions.md`) runs in the Round 6 sweep, before the file split
  (267), the Rust style items (355), the final sweep (365) and the suite prune (152b). 2026-10-19, extended 2026-10-20
  (rows [266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md), [267](./backlog/done/267-split-oversized-source-files-into-modules.md), [355](./backlog/done/355-adopt-google-rust-style-guide-items.md), [365](./backlog/done/365-final-sweep-refactor-and-root-cause-review.md), [152](./backlog/152-test-suite-review-revisit-the-playwright-suite.md)).
- Google Rust style items adopted: a workspace lint table, `unwrap_used` denied in primitives and cli, `must_use` on
  builders, `#[expect]` with a reason, and the conventions in [`guides/rust-conventions.md`](./guides/rust-conventions.md).
  2026-10-21 (row [355](./backlog/done/355-adopt-google-rust-style-guide-items.md)).
- Remote branch deletion needs the owner's explicit go; none is deleted without it. 2026-10-24 (row [266](./backlog/266-repo-cleanup-archive-historical-dev-docs.md)).
