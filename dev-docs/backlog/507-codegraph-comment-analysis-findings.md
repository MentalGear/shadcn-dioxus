---
id: 507
title: "Code analyzer -> comment analysis: a standing backlog for what the tool's metrics turn up"
status: "OPEN, STANDING. Owner-requested 2026-11-11: run the comment analysis and record what can be learned…"
size: "S per pass (a read of the Comments tab…"
area: tooling
updated: 2026-11-19
---

# Row 507: code analyzer -> comment analysis

## Item

The code graph explorer's comment metrics (row [486](./486-code-graph-explorer-tools-codegraph.md), M1-M6 and N1-N6,
MentalGear/code-analyser, run here by `scripts/codegraph.sh`) are a dev tool, not a gate: nothing runs them automatically, so what they find only turns into
a fix when someone reads the Comments tab and acts on it. This row is that standing backlog: each pass records what
it found and fixed, and what it found but left for later, so the next pass (or the Rust port, when it reaches the
code N4 points at) has a starting point instead of re-discovering the same things.

## Status / trigger

OPEN, STANDING. Owner-requested 2026-11-11: run the comment analysis and record what can be learned from it.

**Pass 1 (2026-11-11, scope: `primitives`; `preview` left out at the owner's request).** Refreshed the graph and
comment metrics on main `b78c231e` (`CARGO_TARGET_DIR=/home/user/tgt/codegraph scripts/codegraph/run.sh --blame`).

- **N1 (stale-looking names), fixed:** `select/components/list.rs` named a component `SelectRoot`; the component
  has always been `Select` (`select/components/select.rs:321`). `command.rs`'s "what `CommandContext` drops" list
  cited two methods, `open_with_empty_query_and_focus_first`/`_last`, that were never real; `Combobox`'s equivalent
  is plain `set_open` (`combobox/context.rs:167`), confirmed by `git grep` finding zero other hits of either
  invented name anywhere in the crate. Both comments now name the real thing.
- **N1, left alone (not a defect):** `poll_vdom` (`bridge/transport.rs`), `GenerationalBox` (`chart/config.rs`),
  `next_element_sibling` (`input_otp.rs`) and `dioxus_sdk_time` (`sleep.rs`) are upstream Dioxus or web-platform
  names cited for comparison, not local identifiers that went stale.
- **Follow-up 1, a tool gap:** N1's citation rules (the ones that excuse "Forked from", a crate version, a URL from
  being flagged) catch a citation by its phrasing, not by knowing every upstream name. `sleep.rs`'s "It is
  `dioxus_sdk_time::sleep`'s own pair of calls..." is a legitimate citation phrased as plain prose, so it still
  reads as an N1 hit. A seventh rule for a bare `crate::path` citation would close this, trading it for the risk of
  a real stale name phrased the same way slipping through unflagged. Not decided whether that trade is worth it.
- **Follow-up 2, input for the Rust port (row 434):** N4 (repeated comment blocks) found real duplication, not
  noise: the menu family (`dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, about 245 lines of near-identical
  comment between them, over near-identical code) and the date pickers (`calendar/view.rs`,
  `date_picker/{popover,picker,input}.rs`, about 254 lines) repeat the same blocks closely enough that one shared
  implementation likely belongs where several hand-kept copies are today. Worth reading again when the port reaches
  those components (rows 451-467); not a refactor to do now, on its own, outside that port.
- **Follow-up 3, a metric-accuracy note:** M3 (pub doc coverage)'s worst-scoring files, `combobox/context.rs` and
  `chart/components/layout.rs`, are not real public-API gaps: their flagged fields are bare `pub` on a struct that
  is itself `pub(super)` (`command.rs`'s own comment says as much for `ComboboxContext`), so nothing outside the
  module can even name the type. M3's lexical `pub` match does not resolve the enclosing type's visibility, so it
  counts these as public. `check-rustdoc`, `check-props-docs.sh` and `check-slot-docs.sh` (which do resolve
  visibility) stay the authority on real documentation debt; M3 could subtract a struct's own non-pub visibility
  from its fields' count, if a future pass finds the false positives costly enough to fix.
- **Follow-up 4, stale top-layer comments (found 2026-11-16, at row 449's landing):** the comments around the popover
  hooks describe them as they were before rows 519 and 449. `top_layer/styles.rs`'s coverage list and its prepend
  paragraph say three hooks prepend `TOP_LAYER_INK_STYLES_INJECT_JS`; only `use_dialog_panel_shown_while_mounted` and
  `lib.rs`'s `use_dialog_open_driver` still do (the executor's level is applied by an eval created after the ink eval,
  `popover_sync.rs`). The "Bug 1" / "Bug 2" and "`use_popover_sync` calls `hidePopover()`" citations in `tooltip.rs`,
  `hover_card.rs`, `popover.rs`, `navbar.rs`, `navigation_menu.rs`, `select/components/list.rs`,
  `combobox/components/list.rs`, `toast.rs`, `lib.rs` and `menu_root.rs` point at doc text that is gone. One pass over
  those files against `popover_sync.rs` as it is now.

## Size

S per pass (a read of the Comments tab plus whatever fixes are unambiguous); the follow-ups above are each their own
small decision, not sized here.

## Detail

- How to run a pass: `CARGO_TARGET_DIR=/home/user/tgt/codegraph LANE_SLOTS=1 scripts/codegraph.sh --blame` (the tool
  is a checkout of MentalGear/code-analyser at `$CODEGRAPH_HOME` or beside the main checkout of this repo), then
  open `target-codegraph/explorer.html`'s Comments tab (or script over `comments.json`, as this pass did, for a
  specific metric across a whole crate rather than one file at a time).
- Next pass: re-run after the Rust port's phase 0 (row 444) lands, since `primitives/src/ts/` and `primitives/src/js/`
  will have moved a great deal of code and the N4 duplication in the menu family may already be gone by construction.

**Evaluation pass 2026-11-19 (this session could only read MentalGear/code-analyser at `374bf4a`; these are items for that
repository).** Measured run and recipes: [`research/repo-evaluation-2026-11-19/evidence.md`](../research/repo-evaluation-2026-11-19/evidence.md), section 9.
- Accuracy per metric (ledger A): undocumented public items genuine in 5 of 5 sampled (the headline 83.9 % counts items
  plus fields; items alone are 2,001 of 2,638, 75.9 %); duplicated comment text genuine in 6 of 6 (3 of them rustdoc
  example boilerplate); TODO 1 of 2 real; the stale-name heuristic (N1) 1 real of 6 (`top_layer/component.rs:45`, `DocsSidebar`).
- A wrong edge for a package with two bin targets (`public-api-snapshot.rs` to `src/main.rs`: the targets share the symbol
  `crate/`), and 20 duplicate-symbol warnings of the same class.
- Blindness to `cfg(target_family = "wasm")` code (the host-cfg index misses `primitives/src/sleep.rs` to `gloo_timers`).
- The index already holds every allocation call site with line and column (`format!` 752, `to_string` 1,514, `vec!` 1,483,
  `collect` 706, `String::clone` 412); a ranking layer of about 50 lines is possible, detection stays with clippy.
