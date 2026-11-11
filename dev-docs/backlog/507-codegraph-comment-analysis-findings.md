---
id: 507
title: "Code analyzer -> comment analysis: a standing backlog for what the tool's metrics turn up"
status: "OPEN, STANDING. Owner-requested 2026-11-11: run the comment analysis and record what can be learned…"
size: "S per pass (a read of the Comments tab…"
area: tooling
updated: 2026-11-11
---

# Row 507: code analyzer -> comment analysis

## Item

The code graph explorer's comment metrics (row [486](./486-code-graph-explorer-tools-codegraph.md), M1-M6 and N1-N6,
`scripts/codegraph/`) are a dev tool, not a gate: nothing runs them automatically, so what they find only turns into
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

## Size

S per pass (a read of the Comments tab plus whatever fixes are unambiguous); the follow-ups above are each their own
small decision, not sized here.

## Detail

- How to run a pass: `CARGO_TARGET_DIR=/home/user/tgt/codegraph LANE_SLOTS=1 scripts/codegraph/run.sh --blame`, then
  open `target-codegraph/explorer.html`'s Comments tab (or script over `comments.json`, as this pass did, for a
  specific metric across a whole crate rather than one file at a time).
- Next pass: re-run after the Rust port's phase 0 (row 444) lands, since `primitives/src/ts/` and `primitives/src/js/`
  will have moved a great deal of code and the N4 duplication in the menu family may already be gone by construction.
