---
id: 428
title: "Themed wrappers forward primitive props with a struct spread, not field by field"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P3)."
size: "M"
area: components
updated: 2026-10-25
---

# Row 428: Themed wrappers forward with `..props`, not field by field

## Item

A themed wrapper forwards every prop of the primitive it renders (row 391). Today 826 forwarded-field lines in 47 `preview/src/components/*/component.rs` files are written out by hand, 816 of them identity (`x: props.x`), and a 25-entry allowlist plus a Rust scanner exist only to notice a field someone forgot. Construction: a wrapper that changes only `attributes` is `prim::Comp { attributes: merged, ..props }`, so a prop added to a primitive reaches the themed API with no edit. Explicit fields remain only where a wrapper deliberately changes a prop.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P3).

## Size

M

## Detail

Source: [final sweep note, P3](../research/final-sweep-2026-10-25.md).

- **Measured:** `grep -E "^\s+[a-z_#]+: props\.[a-z_#]+,?$" preview/src/components/*/component.rs`. Largest: `calendar/component.rs` 69, `date_picker` 60, `dropdown_menu` 52 (`dropdown_menu/component.rs:21-35` forwards eight fields, `:60-70` five), `context_menu` 51, `virtual_grid` 47, `combobox` 42, `select` 37. The allowlist is `scripts/check-wrapper-props.allow.tsv` (25 live entries, ten of them "Internal part; the primitive generates its id").
- **Precedent in the repo:** `chart/component.rs:67,98,126` forward with `chart::ChartContainer { attributes: merged, ..props }`; `:58` clones `props.attributes` first; `:85-86` cites `primitives/src/toast.rs`'s own `Toast { ..props }`.
- **Subsumes:** the ~800 identity-forward lines; the `*.id` allowlist entries whose only reason is "the wrapper does not touch it" (the spread forwards them, which is correct: the primitive's own id generation is unaffected when the prop is `None`). **Does not subsume:** deliberate changes (slider's `index` pinned to 0; `select::SelectValue.placeholder` passed conditionally; `color_picker` composing a popover with its own surface props). Those stay explicit with their allowlist reasons.
- **Gate:** `check-wrapper-props.sh` already treats `..props` as forwarding every prop. Add a rule that a wrapper call naming more than N fields all as `x: props.x` fails ("use `..props`"). The allowlist shrinks to the deliberate cases.
- **Friction (inferred):** Rust partial moves (`props.children` read after a spread). The `.clone()` pattern in `chart/component.rs` handles it. First step: convert three wrappers (`dropdown_menu`, `calendar`, one small one) and report the friction before the rest, which is mechanical.
