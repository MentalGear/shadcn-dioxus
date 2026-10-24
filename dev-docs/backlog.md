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

## Open rows (24)

| # | Item | Status / trigger | Size | Area |
|---|---|---|---|---|
| [4](backlog/004-first-3-browser-ci-run-treat-as-calibration-not.md) | First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and… | WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30). The GitHub-hosted… | S–M | ci |
| [23](backlog/023-preview-s-desktop-feature-lacks-dioxus.md) | preview's desktop feature lacks dioxus-primitives/web — a webview-backed desktop build would render the… | WAITING ON OWNER: a desktop build (needs gtk / webkit2gtk, absent in the sandbox). The fix is in… | S | overlays |
| [28](backlog/028-verify-the-2026-09-02-ios-keyboard-anchor.md) | Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback… | WAITING ON OWNER: re-test on a real iPhone after deploy. The fix is in; the sandbox has no WebKit… | S to verify | overlays |
| [30](backlog/030-verify-the-2026-09-02-touch-text-entry-font.md) | Verify the 2026-09-02 touch text-entry font-size floor… | WAITING ON OWNER: re-test on a real iPhone after deploy. The fix is in; Playwright's iPhone 13… | S to verify | tests |
| [33](backlog/033-3-browser-playwright-on-every-push-pr-sharded.md) | 3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit… | WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30); then it is a… | S | ci |
| [152](backlog/152-test-suite-review-revisit-the-playwright-suite.md) | Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for… | IN PROGRESS 2026-10-24. Done: 152a (below); the shared-page fixture, through rows 327, 358 and 334… | M | tests |
| [153](backlog/153-native-find-in-page-for-windowed-virtual-lists.md) | Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"… | Left: a Firefox 139+ and Safari 26.2 run of the find bar, and a release-build mount timing for 100k… | M | virtual |
| [170](backlog/170-cross-component-css-overrides-190-sites-in-12.md) | Cross-component CSS overrides: 190 sites in 12 sheets declare, in one component's stylesheet, a property that… | NARROWED 2026-10-19 (backlog review): baseline is now 138 sites in 9 sheets (was 190 in 12) | M (the decision and the prototype are… | theme |
| [188](backlog/188-demo-parity-second-audit-the-owner-s-per.md) | Demo parity, second audit: the owner's per-example list against the original shadcn demos (51 components; row… | Left: the combobox items of the 2026-10-11 audit that have no part or demo, and one decision.… | M per batch | docs |
| [266](backlog/266-repo-cleanup-archive-historical-dev-docs.md) | Repo cleanup: archive historical dev-docs, trim CLAUDE.md and the big dev docs, split backlog.md | Left: deleting the stale remote branches (WAITING ON OWNER: his go), part (c) and part (d). (c) is… | M-L | process |
| [267](backlog/267-split-oversized-source-files-into-modules.md) | Split oversized source files into modules (carousel.rs, carousel.spec.ts, main.rs, top_layer.rs, calendar.rs, date_picker.rs) | BACKLOG (owner approved 2026-10-19 on the cleanup plan; Round 6, before the 152b suite prune).… | M | process |
| [268](backlog/268-select-scroll-buttons-and-item-aligned-position.md) | Select scroll buttons and position=item-aligned | Only the native (Blitz) arm is left: position=item-aligned emits data-position there and nothing… | S (native arm only) | select |
| [295](backlog/295-press-feedback-on-touch-taps-already-on-by.md) | Press feedback on touch taps: already on by default, so check why taps don't show it | WAITING ON OWNER: iPhone re-test of press feedback on /component/tabs/, /component/menubar/… | S (or none) | theme |
| [296](backlog/296-combobox-list-drifts-from-its-input-on-ios-after.md) | Combobox list drifts from its input on iOS after long scrolling | WAITING ON OWNER: reproduce on an iPhone (owner report 2026-10-19); no fix has landed and the cause… | M + device check | overlays |
| [305](backlog/305-side-drawer-scrolling-its-content-also-drags-the.md) | Side drawer: scrolling its content also drags the drawer sideways (iOS) | WAITING ON OWNER: iPhone re-test of the side drawer (scroll its content, the drawer must not drag… | S–M | primitives |
| [307](backlog/307-ios-color-panel-does-not-pick-colors.md) | iOS: the Color Panel opens but tapping its color box does not change the color | WAITING ON OWNER: reproduce on an iPhone (owner report 2026-10-20: the Color Panel opens but… | M | overlays |
| [355](backlog/355-adopt-google-rust-style-guide-items.md) | Adopt Google Rust style guide items: workspace lint table and conventions | BACKLOG -- Round 6 sweep, after row 267's file split (owner approved 2026-10-21) | S-M | process |
| [365](backlog/365-final-sweep-refactor-and-root-cause-review.md) | Final sweep: refactor review with the deconstruct-to-root and solve-by-construction exercise, proposals to the owner | BACKLOG (owner 2026-10-22, backlog thread: "final sweep, check for refactors and also run there the… | M for the review. Each accepted… | process |
| [371](backlog/371-dependency-evaluation-keep-lift-drop.md) | Dependency evaluation: keep, feature-gate, replace, lift or drop (rule of 2026-10-23) | WAITING ON OWNER: parked ("its own item, and not yet", 2026-10-23); no work until he reopens it.… | S each outcome | build |
| [372](backlog/372-scroll-fade-overlay-scrollbar-hybrid.md) | Scroll fade: transition the edge depths (landed) and paint the fade as gradients where the scrollbar is an overlay (deferred) | WAITING ON OWNER: part 1, the overlay-scrollbar hybrid, is deferred ("leave mask for now"); reopen… | M | primitives |
| [382](backlog/382-rustdoc-warnings-to-zero.md) | Rustdoc warnings to zero: shrink the check-rustdoc baseline (87 entries, mostly unclosed HTML tags) | OPEN, S, no browser: shrink scripts/check-rustdoc.baseline.tsv (87 entries, mostly unclosed HTML… | S | primitives |
| [383](backlog/383-adversarial-review-board.md) | Adversarial review board: independent expert reviewers attack the finished library, last item of the backlog | BACKLOG. Runs last: after row 365's proposals are answered and every other engineering row has… | M | process |
| [384](backlog/384-burn-down-ratchet-baselines.md) | Burn down the ratchet baselines and the leftover TODO comments (props docs, history comments, dead refs) | BACKLOG. The history-comment part overlaps row 267's extension (rewrite history comments while… | M | process |
| [385](backlog/385-accent-focus-ring-clamped.md) | The focus ring follows the picked accent, clamped to 3:1 per mode | DECIDED 2026-10-24 (owner), BUILT on lane/accent-ring. With an accent picked the ring takes the… | S | theme |

<!-- END GENERATED -->
