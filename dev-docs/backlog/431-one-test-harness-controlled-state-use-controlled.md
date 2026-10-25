---
id: 431
title: "One test runtime harness; controlled state only through use_controlled"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P6)."
size: "S"
area: primitives
updated: 2026-10-25
---

# Row 431: One test runtime harness, and controlled state only through `use_controlled`

## Item

A concern implemented once in `primitives/src` is used, not copied, and two concerns are copied today: the test runtime harness (eight `with_runtime` and seven `TestHarnessProps` copies) and controlled-or-uncontrolled state (three hand-rolled copies of what `use_controlled` already does, one of them exact). Construction: `test_support::with_runtime`, generic over a return value, imported by every test; all controlled state goes through `use_controlled` (`primitives/src/lib.rs:256`, 28 callers). After it lands, a ninth harness or a fourth hand-rolled controlled state is a grep failure.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P6).

## Size

S

## Detail

Source: [final sweep note, P6](../research/final-sweep-2026-10-25.md). Primitives, tests and three signals; one grep gate.

- **Harness copies:** `primitives/src/slider.rs:1045,1065`, `pointer.rs:143,163`, `selection.rs:236,258`, `color_picker.rs:748,768`, `direction.rs:287,303` (a variant returning a `Direction`), `calendar/tests.rs:1070,1090`, `date_picker/tests/state.rs:9,29`, `virtual/identity.rs:92`. `slider.rs:1043` says it mirrors `virtual/virtualizer.rs`. `primitives/src/test_support.rs` exists (`#[cfg(test)]`, an SSR start-tag parser from row 108) but does not hold the harness.
- **Controlled-state copies:** `alert_dialog.rs:119-124` is an exact copy (`use_signal(default_open)`, a set plus `on_open_change` callback, `use_memo(props.open or signal)`), while its sibling `dialog.rs:239` calls `use_controlled`. `accordion.rs:313` is the same shape. `drawer.rs:894-896` (snap-point state) differs in its reset to default, so it is a candidate, not an exact copy.
- **Subsumes:** the eight harness copies and the two exact controlled-state copies. **Does not subsume:** `drawer.rs` unless its reset can be expressed through `use_controlled` (then it is documented as such); `VirtualDom::new` calls that are not this harness (111 in total, most of them SSR).
- **Gate:** no `struct TestHarnessProps` outside `test_support.rs`; no `use_memo(... unwrap_or_else(&*` in a component outside `use_controlled`. The pattern `grep -rnE "unwrap_or_else\(&\*[a-z_]+\)|\.or_else\(&\*[a-z_]+\)"` matches exactly the three sites plus `navbar.rs:696`, which is a different thing.
- **Root cause:** the harness was written as a private test module twice before anyone looked, and `use_controlled` landed after `AlertDialogRoot` was written with nothing to find the stragglers.
