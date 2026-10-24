# General backlog -- open rows

One file per row. Everything outstanding across the plan, the harness and the follow-ups found during execution
lives in `dev-docs/backlog/`; **this file is only the generated index of the OPEN rows** and
[`backlog-done.md`](./backlog-done.md) the generated index of the done ones. A citation such as
"`dev-docs/backlog.md` row 116" (about 250 files and 530 comments carry one) resolves by id: look in the table below
for an open row and in `backlog-done.md` for a finished one, or glob `dev-docs/backlog/*116-*` and
`dev-docs/backlog/done/*116-*`.

## Scope

**Owner rule (2026-10-21): the library keeps to primitives and shadcn element parity**, even where an element is more
complex (the chat kit counts as parity). Bigger stateful items that need their own state management, such as a
Markdown or rich-text editor, belong to a higher-level framework or an external library, not to this backlog; a row
proposing one is closed as out of scope (row 337 is the first).

## How it works

- **Open rows:** `dev-docs/backlog/<NNN>-<slug>.md` (`NNN` = the zero-padded row id, never renumbered, ids are never reused).
- **Done rows:** the same file under `dev-docs/backlog/done/` once its status starts with LANDED / FIXED / RESOLVED /
  CLOSED / SUPERSEDED / DONE and names no open remainder. Anything partly done stays in the open set. A done row keeps
  its front matter, the four sections and a short summary with the landing commit or PR; the long narrative lives in
  git history (code cites rows by id, so the file stays).
- **A row file** has front matter (`id`, `title`, `status`, `size`, `area`, `updated`) and four sections, in this
  order: `## Item`, `## Status / trigger`, `## Size`, `## Detail`. `title`, `area` and `updated` are hand-editable
  (`area` is a coarse tag, `updated` the latest date the row mentions); `status` and `size` are derived from their
  sections.
- Sub-ids such as "row 31a / 31b" or "item 21" in citations live inside that row's file (31, 21).
- **To add a row:** pick the next free id (the highest id in the two indexes, plus one), copy any row file, rename it
  `<NNN>-<slug>.md`, fill the front matter and the four sections.
- **To update a row:** edit its sections (append dated notes to Detail, change Status / trigger when it lands). When it
  lands, `git mv` it into `dev-docs/backlog/done/`.
- **Then run `scripts/backlog-index.sh`** (syncs `status`/`size`, rewrites both indexes) and commit the row file and
  the index files together. `scripts/check-backlog-index.sh` (a `run-gates.sh` gate) fails on a stale index, a missing
  front matter field, a duplicate id, a row in both sets, and a broken relative link under `dev-docs/`.
- Never hand-edit between the GENERATED markers; edit this header freely.
- The current order of work is in [`research/backlog-order-2026-10-18.md`](./research/backlog-order-2026-10-18.md) and the
  live state in [`STATUS.md`](./STATUS.md). The original monolithic file (244 rows at `de7059ba`) and its narrative
  sections are in git history before row 266's cleanup (`3c584cc2`).

<!-- BEGIN GENERATED: scripts/backlog-index.sh rewrites everything up to END; edit the row files, not this block -->

## Open rows (30)

| # | Item | Status / trigger | Size | Area |
|---|---|---|---|---|
| [4](backlog/004-first-3-browser-ci-run-treat-as-calibration-not.md) | First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and… | A manual dispatch of playwright.yml on a GitHub-hosted runner: all workflows are… | S–M | ci |
| [10](backlog/010-phase-5-collision-detection.md) | Phase 5 — collision detection | 5.1 REFRAMED by the 2026-09-01 bits-ui/Radix source comparison, FLIP sub-problem landed same day… | S (flip, done) + S (JS-fallback shift… | theme |
| [23](backlog/023-preview-s-desktop-feature-lacks-dioxus.md) | preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the… | OPEN (verification only): FIXED 2026-09-26 (3633b20), verified at feature-resolution level only… | S | overlays |
| [28](backlog/028-verify-the-2026-09-02-ios-keyboard-anchor.md) | Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback… | User re-test after deploy — the sandbox has no WebKit, and page.setViewportSize() cannot reproduce… | S to verify | overlays |
| [30](backlog/030-verify-the-2026-09-02-touch-text-entry-font.md) | Verify the 2026-09-02 touch text-entry font-size floor… | User re-test after deploy — the sandbox has no WebKit, and while Playwright's iPhone 13 emulation… | S to verify | tests |
| [33](backlog/033-3-browser-playwright-on-every-push-pr-sharded.md) | 3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit… | CI reactivation (user: off until further notice, 2026-08-30) — then it is a workflow-trigger… | S | ci |
| [152](backlog/152-test-suite-review-revisit-the-playwright-suite.md) | Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for… | IN PROGRESS 2026-10-24. Done: 152a (below); the shared-page fixture, through rows 327, 358 and 334… | M | tests |
| [153](backlog/153-native-find-in-page-for-windowed-virtual-lists.md) | Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"… | PROTOTYPED 2026-10-19 (lane virtual-grid, round 4); Chromium Ctrl+F verified 2026-10-23 -- works in… | M | virtual |
| [155](backlog/155-shadcn-parity-wave-group-1-the-invalid-ring.md) | shadcn parity wave (group 1, the invalid ring, LANDED 2026-10-13 with row 145's state recipes): the 27 "Add… | OPEN (last follow-up on branch claude/row155-picker-month-short, move to done when it merges)… | M-L in total (23 of the 27 are S; four… | theme |
| [170](backlog/170-cross-component-css-overrides-190-sites-in-12.md) | Cross-component CSS overrides: 190 sites in 12 sheets declare, in one component's stylesheet, a property that… | NARROWED 2026-10-19 (backlog review): baseline is now 138 sites in 9 sheets (was 190 in 12) | M (the decision and the prototype are… | theme |
| [188](backlog/188-demo-parity-second-audit-the-owner-s-per.md) | Demo parity, second audit: the owner's per-example list against the original shadcn demos (51 components; row… | DEMO-ONLY ITEMS DONE 2026-10-19 (batch 2: every section-5 demo exists; what remains needs component… | M per batch | docs |
| [217](backlog/217-dependency-audit-follow-ups-what-users-pay-for.md) | Dependency audit follow-ups (what users pay for in compile time and supply chain) | PARTLY LANDED 2026-10-18 (lane deps 04ad7ddf): features calendar/color-picker, no build script… | S-M each | build |
| [250](backlog/250-container-reset-loses-unpushed-work.md) | A container reset to a fresh clone loses every unpushed lane commit and all session scratch | CONSTRUCTION LANDED 2026-10-19 (lane-rules: push lane/* after every commit); redo of the lost work… | S | infra |
| [266](backlog/266-repo-cleanup-archive-historical-dev-docs.md) | Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md | Main cleanup LANDED in batch 15. Lane cleanup-claudemd: CLAUDE.md 47 KB to 8 KB with every rule… | M-L | process |
| [267](backlog/267-split-oversized-source-files-into-modules.md) | Split oversized source files into modules (carousel.rs, carousel.spec.ts, main.rs, top_layer.rs, calendar.rs, date_picker.rs) | BACKLOG (owner approved 2026-10-19 on the cleanup plan; Round 6, before the 152b suite prune).… | M | process |
| [268](backlog/268-select-scroll-buttons-and-item-aligned-position.md) | Select scroll buttons and position=item-aligned | NARROWED 2026-10-23. Both parts are on main and their browser specs are green (progress note… | S-M (scroll buttons), M-L (item-aligned) | select |
| [295](backlog/295-press-feedback-on-touch-taps-already-on-by.md) | Press feedback on touch taps: already on by default, so check why taps don't show it | PARTLY LANDED 2026-10-20 (theme-touch lane 1ba0b2e5, cf64037a); device re-test open. Row 205… | S (or none) | theme |
| [296](backlog/296-combobox-list-drifts-from-its-input-on-ios-after.md) | Combobox list drifts from its input on iOS after long scrolling | OPEN (owner report 2026-10-19), needs device verification. Related rows: 28 (iOS keyboard anchor… | M + device check | overlays |
| [305](backlog/305-side-drawer-scrolling-its-content-also-drags-the.md) | Side drawer: scrolling its content also drags the drawer sideways (iOS) | BACKLOG (owner report 2026-10-19, iOS device) | S–M | primitives |
| [307](backlog/307-ios-color-panel-does-not-pick-colors.md) | iOS: the Color Panel opens but tapping its color box does not change the color | OPEN (owner report 2026-10-20, iOS device; being investigated in the 'iOS Color Panel won't pick… | M | overlays |
| [309](backlog/309-virtuallist-and-virtualgrid-share-one-engine.md) | VirtualList and VirtualGrid share one engine: DOM bridge, pinned focus, fade (two components, one engine) | OPEN (all parts built; the grid find-in-page placeholders and content-visibility mode landed… | M | virtual |
| [321](backlog/321-one-merge-queue-owner-for-main-batch-one.md) | One merge-queue owner for main (batch, one full gate and one suite per batch, deploy) | OPEN, documented (owner approved 2026-10-21; the QUEUE format is in lane-rules.md "Merge queue"… | S | process |
| [355](backlog/355-adopt-google-rust-style-guide-items.md) | Adopt Google Rust style guide items: workspace lint table and conventions | BACKLOG -- Round 6 sweep, after row 267's file split (owner approved 2026-10-21) | S-M | process |
| [365](backlog/365-final-sweep-refactor-and-root-cause-review.md) | Final sweep: refactor review with the deconstruct-to-root and solve-by-construction exercise, proposals to the owner | BACKLOG (owner 2026-10-22, backlog thread: "final sweep, check for refactors and also run there the… | M for the review. Each accepted… | process |
| [369](backlog/369-virtual-fractional-rows-grid-table.md) | VirtualGrid and TableRows: carry the exact measured size so a short body with fractional rows cannot overflow by 1px | Open. Found in batch 10 (scroll-fade 199 vs 198). | S | primitives |
| [370](backlog/370-merge-attributes-class-none.md) | merge_attributes: a class: None replaces the base class instead of being ignored | Open. Found by the biggrid lane in batch 10. | S | primitives |
| [371](backlog/371-dependency-evaluation-keep-lift-drop.md) | Dependency evaluation: keep, feature-gate, replace, lift or drop (rule of 2026-10-23) | PARKED by the owner 2026-10-23 ("its own item, and not yet"). Inventory done; no outcome is… | S each outcome | build |
| [372](backlog/372-scroll-fade-overlay-scrollbar-hybrid.md) | Scroll fade: transition the edge depths (landed) and paint the fade as gradients where the scrollbar is an overlay (deferred) | Part 2 LANDED (lane fadehybrid): the registered --dx-scroll-fade-t/-b/-s/-e depths transition… | M | primitives |
| [376](backlog/376-toast-height-follows-content.md) | Toast height follows its content: measured heights, an even collapsed pile, own heights expanded | Owner bug report with a screenshot, 2026-10-23, plus the question "how should dynamic height via… | M | overlays |
| [382](backlog/382-rustdoc-warnings-to-zero.md) | Rustdoc warnings to zero: shrink the check-rustdoc baseline (87 entries, mostly unclosed HTML tags) | OPEN. Fix the doc comments (backticks around the generic or tag), then scripts/check-rustdoc.sh… | S | primitives |

<!-- END GENERATED -->
