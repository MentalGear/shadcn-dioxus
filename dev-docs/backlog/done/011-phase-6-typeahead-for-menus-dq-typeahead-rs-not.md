---
id: 11
title: "Phase 6 — typeahead for menus (dq typeahead.rs, not select/)"
status: "LANDED 2026-09-18. APG \"Menu and Menubar\" pattern, Keyboard Interaction (pinned commit 7e4034b of…"
size: "S–M"
area: menus
updated: 2026-09-18
---

# Row 11: Phase 6 — typeahead for menus (dq typeahead.rs, not select/)

## Item

Phase 6 — typeahead for menus (`dq typeahead.rs`, **not** `select/`)

## Status / trigger

**LANDED 2026-09-18.** APG "Menu and Menubar" pattern, Keyboard Interaction (pinned commit `7e4034b` of `w3c/aria-practices`): "Any key that corresponds to a printable character (Optional): Move focus to the next item in the current menu whose label begins with that printable character." None of `DropdownMenu`/`ContextMenu`/`Menubar` implemented this; a conformance test (new rows in `playwright/oracle/tier1-apg/keyboard-matrix.spec.ts`, 12 rows, menu sections only) was written first and would fail red against the pre-existing code. Construction: ported `dignifiedquire`'s `typeahead.rs` *concept* (prefix matching over a timed buffer) — not `select/`'s deliberately-kept Levenshtein matcher (`recommended-implementations.md` §8 already settled that comparison; this row's own trigger note said "not `select/`", honored). New `primitives/src/typeahead.rs`: a pure `find_next_match(labels, current, buffer)` function (unit-tested: wrap, skip-disabled, same-character cycling, case-insensitivity, Radix-parity narrowing) plus `TypeaheadState`, mirroring `select/context.rs`'s buffer/timeout idiom (`Signal<String>` + a self-cancelling clear `Task` via `dioxus_sdk_time::sleep`, 1000ms window). Wired into each host's existing keydown handler next to its Arrow/Home/End arms, and into `menu_sub.rs`'s `DropdownMenuSub`/`ContextMenuSub` content handlers too, since the collection plumbing made it free; never intercepts Ctrl/Meta/Alt-held keys or Space. Item labels: an explicit optional `text_value` prop (on `DropdownMenuItem`/`SubItem`, `ContextMenuItem`/`SubItem`, `MenubarItem`, and the Sub triggers), falling back to the item's own value where meaningful; `collection.rs` gained one additive `text_value` channel through the existing per-item registration plus a `text_entries()` accessor — not a second item registry (commit `6b724bb`). **Follow-up defect found same day:** each menu-family themed wrapper (`preview/src/components/{dropdown_menu,context_menu,menubar}/component.rs`) takes the primitive's own `*Props` type directly but re-lists fields individually when constructing the inner primitive component, so adding `text_value` to the primitive props did not automatically forward it through the wrapper — a caller setting `text_value` on a themed component had it silently dropped before reaching `collection.rs`'s registration. `cargo check` alone did not catch it (an unused/dropped prop field is not a compile error); found by re-checking the compiled demo actually carries the label through. Fixed at all seven forwarding sites this bug class touches, not just the one the demo happened to need, per this repo's "two occurrences means a class" convention (commit `330211b`).

## Size

S–M

## Detail

Also a prerequisite-ish for Command palette in the component queue. 129 lib + 120 doc tests passed (10 new typeahead unit tests); clippy clean outside the pre-existing `menu_sub.rs` doc-lint. Playwright not run by the lane
