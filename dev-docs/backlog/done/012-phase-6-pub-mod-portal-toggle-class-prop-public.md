---
id: 12
title: "Phase 6 — pub mod portal, Toggle class prop, public CalendarDayState (upstream #199)"
status: "LANDED 2026-09-18, three small API-surface commits on one lane, each already flagged during the…"
size: "S each"
area: inputs
updated: 2026-09-18
---

# Row 12: Phase 6 — pub mod portal, Toggle class prop, public CalendarDayState (upstream #199)

## Item

Phase 6 — `pub mod portal`, `Toggle` `class` prop, public `CalendarDayState` (upstream #199)

## Status / trigger

**LANDED 2026-09-18**, three small API-surface commits on one lane, each already flagged during the fork-mining survey (`adopt-fork-fixes-results.md` row 12/269-271): **(a) `pub mod portal`** (commit `b349191`) — `mod portal;` kept `use_portal`/`PortalId`/`PortalIn`/`PortalOut` unreachable from outside the crate though nothing about them is fork-specific or unfinished. Flipped to `pub mod portal;`, added a module-level doc with a runnable example, rustdoc on every public item, and two `#[cfg(test)]` `dioxus_ssr` tests (children render at the matching `PortalOut`, not at `PortalIn`'s own site; `PortalOut` renders nothing before any `PortalIn` has run). `RUSTDOCFLAGS="-Dwarnings --document-private-items" cargo doc` stays clean for this module specifically (zero entries among the crate's pre-existing, unrelated doc-link errors elsewhere — see row 67). **(b) `Toggle`'s unstyled `class` prop** (commit `2eb08f6`) — `ToggleProps` was the odd one out among primitives exposing `class` (`radio_group.rs`, `alert_dialog.rs`, `dialog.rs`, `tabs.rs`, `navbar.rs`, `popover.rs`). Added `pub class: Option<String>` with **no default** (stays unstyled, matching `RadioItem`'s shape rather than the fork commit that flagged this, which defaulted the class to a themed-looking `"toggle"`). Fixed `preview/src/components/toggle/component.rs` to merge a caller's class with `"dx-toggle"` (previously only ever set the theme class, dropping a caller's own) using the same `if let Some(class) = props.class { format!(...) }` shape `radio_group`/`popover`'s wrappers already use — this codebase's own instance of the "merge, don't clobber" construction commit `724bfee` established for `Input`. Auditing `toggle_group/component.rs` (wraps `Toggle` via `ToggleItem`) found the identical clobber one level removed in its `attributes` handling — fixed with `merge_attributes`/`attributes!`, the same mechanism `popover/component.rs` already uses for that exact shape. **(c) Public `CalendarDayState`** (commit `be7b3c7`) — answers upstream `DioxusLabs/components#199` ("the APIs to build each component should be public"). From-scratch port against current source (`lifting-from-forks.md` §3, Shape C; the fork reference that flagged this, `jaysonmaw@3c43ce75`, "uses `.unwrap()` and an `unreachable!()` reachable by misuse outside a Calendar context" — not taken). New `pub struct CalendarDayState`, `pub fn use_calendar_day_state(date) -> Option<CalendarDayState>` (`try_use_context` throughout, `None` outside context, never a panic), and `pub fn calendar_day_attributes(state) -> Vec<Attribute>`. `SingleCalendarDay`/`RangeCalendarDay` refactored to consume both via a shared private builder so the built-in cell and a caller's custom cell can no longer drift apart. Verified value-identical rendered HTML by reading `dioxus-ssr`'s own `write_attribute`/`BOOL_ATTRS` source, not assumed; `RangeCalendarDay`'s attribute *order* shifts (a `data-disabled` that rendered first now renders fourth, matching `SingleCalendarDay`), confirmed not to affect `calendar.spec.ts`'s only attribute assertion (an order-independent CSS selector).

## Size

S each

## Detail

133 unit + 122 doc tests passed (20 new: 6 for (b)'s clobber fixes, 12 for (c)'s day-state/attribute computation, 2 portal SSR tests already counted in (a)); `cargo test -p preview components` 10 passed. Also surfaced, same lane: `cargo doc -p dioxus-primitives --no-deps` with `RUSTDOCFLAGS=-Dwarnings` has 12 pre-existing broken/private intra-doc links outside this lane's own files — recorded as row 67
