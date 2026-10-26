---
id: 431
title: "One test runtime harness; controlled state only through use_controlled"
status: "DONE. Owner approved 2026-10-25 (row 365 sweep, P6)."
size: "S"
area: primitives
updated: 2026-10-25
---

# Row 431: One test runtime harness, and controlled state only through `use_controlled`

## Item

A concern implemented once in `primitives/src` is used, not copied, and two concerns are copied today: the test runtime harness (eight `with_runtime` and seven `TestHarnessProps` copies) and controlled-or-uncontrolled state (three hand-rolled copies of what `use_controlled` already does, one of them exact). Construction: `test_support::with_runtime`, generic over a return value, imported by every test; all controlled state goes through `use_controlled` (`primitives/src/lib.rs:256`, 28 callers). After it lands, a ninth harness or a fourth hand-rolled controlled state is a grep failure.

## Status / trigger

DONE. Owner approved 2026-10-25 (row 365 sweep, P6).

## Size

S

## Detail

Source: [final sweep note, P6](../../research/final-sweep-2026-10-25.md). Primitives, tests and three signals; one grep gate.

- **Harness copies:** `primitives/src/slider.rs:1045,1065`, `pointer.rs:143,163`, `selection.rs:236,258`, `color_picker.rs:748,768`, `direction.rs:287,303` (a variant returning a `Direction`), `calendar/tests.rs:1070,1090`, `date_picker/tests/state.rs:9,29`, `virtual/identity.rs:92`. `slider.rs:1043` says it mirrors `virtual/virtualizer.rs`. `primitives/src/test_support.rs` exists (`#[cfg(test)]`, an SSR start-tag parser from row 108) but does not hold the harness.
- **Controlled-state copies:** `alert_dialog.rs:119-124` is an exact copy (`use_signal(default_open)`, a set plus `on_open_change` callback, `use_memo(props.open or signal)`), while its sibling `dialog.rs:239` calls `use_controlled`. `accordion.rs:313` is the same shape. `drawer.rs:894-896` (snap-point state) differs in its reset to default, so it is a candidate, not an exact copy.
- **Subsumes:** the eight harness copies and the two exact controlled-state copies. **Does not subsume:** `drawer.rs` unless its reset can be expressed through `use_controlled` (then it is documented as such); `VirtualDom::new` calls that are not this harness (111 in total, most of them SSR).
- **Gate:** no `struct TestHarnessProps` outside `test_support.rs`; no `use_memo(... unwrap_or_else(&*` in a component outside `use_controlled`. The pattern `grep -rnE "unwrap_or_else\(&\*[a-z_]+\)|\.or_else\(&\*[a-z_]+\)"` matches exactly the three sites plus `navbar.rs:696`, which is a different thing.
- **Root cause:** the harness was written as a private test module twice before anyone looked, and `use_controlled` landed after `AlertDialogRoot` was written with nothing to find the stragglers.

## Outcome

- **Test harness:** `with_runtime<R>(f: impl Fn() -> R) -> R` in `primitives/src/test_support.rs` (the `direction.rs` variant that returns a `Direction` is the same function with `R = Direction`). The 8 private copies and 7 `TestHarnessProps` structs are gone from `slider.rs`, `pointer.rs`, `selection.rs`, `color_picker.rs`, `direction.rs`, `calendar/tests.rs`, `date_picker/tests/state.rs` and `virtual/identity.rs` (a net of about 250 lines); each imports `crate::test_support::with_runtime`. `preview/` had no copy. `with_runtime` also fails when the component never ran, as every copy did.
- **Controlled state:** `lib.rs` gained `use_controlled_state` (the internal signal, the resolved memo and the set callback as a `ControlledState`) and `use_controlled` is now a two-field view of it, so "internal signal, memo of `prop.or(internal)`, callback that writes and notifies" is written once. `AlertDialogRoot` calls `use_controlled`; `Accordion` calls `use_controlled_state` (its `seed_open` writes the internal signal without notifying, which `use_controlled` does not expose) and its context's `commit` is the helper's `set`. Behaviour is unchanged in both paths; the one construction difference is the alert dialog's fallback: `use_controlled` seeds the internal signal from a controlled prop at mount (the accordion already did), where the alert dialog seeded it from `default_open`, which shows only when a controlled alert dialog later turns uncontrolled (the lib test pins the helper's rule).
- **`drawer.rs`: not migrated, allowlisted with the reason.** Its snap-point state is `Option<SnapPoint>` whose uncontrolled start is `default_active_snap_point` even when the prop is `Some` at mount, and `reset_active_snap_point` writes that default back without calling `on_active_snap_point_change`. `use_controlled_state` seeds the fallback from the controlled value and has no silent reset, so expressing the drawer through it needs a prop-wrapping memo (`Option<Option<_>>`) and changes what an uncontrolled-after-controlled drawer shows: not provably identical.
- **Gate:** `scripts/check-test-harness-controlled.sh` (in `run-gates.sh` under "Rust and rsx source rules", `dev-docs/gates.md` regenerated): no `struct TestHarnessProps` or `fn with_runtime` outside `test_support.rs`, no `use_memo(.. .unwrap_or_else(&*sig) / .or_else(&*sig))` outside `use_controlled_state`, with `scripts/lib/allow.py` for the two reasoned entries (the implementation, the drawer) and a self-test of six mutations. `navbar.rs:696` (`unwrap_or_else(&*is_open)` outside a memo) is not the pattern.
- **Tests added:** `lib.rs` `controlled_state_tests` (uncontrolled start and write, `set` notifying twice when the value is unchanged, a controlled value winning while a set still notifies and seeds the fallback, `use_controlled` as a view) and five `AccordionContext` tests (a seed does not call `on_value_change`, a seed does not override a controlled value, opening the open item reports the unchanged set, a controlled click reports without changing what is shown, a non-collapsible accordion keeps its last open item). A memo recomputes on the next render pass, so the tests read the signal the callback wrote.
- **Results:** `cargo test -p dioxus-primitives` 992 unit and 171 doc tests passed, 0 failed (9 of the unit tests are new); clippy default and `web` arms `--all-targets -- -D warnings` clean; `check-primitives-features` OK; `run-gates.sh --fast` 0 red; `test:changed` on a lane-ssg debug build: 211 spec files (the diff names `lib.rs`, `direction.rs` and `test_support.rs`, so `select.mjs` maps it to the shared-plumbing set; it does not read `#[cfg(test)]`), 3279 passed, 24 skipped, 0 failed (`hydration-parity` `selftest:unfixed:id` is the deliberate expected failure).

