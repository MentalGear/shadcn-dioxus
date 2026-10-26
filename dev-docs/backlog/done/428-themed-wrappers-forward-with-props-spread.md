---
id: 428
title: "Themed wrappers forward primitive props with a struct spread, not field by field"
status: "LANDED. Owner approved 2026-10-25 (row 365 sweep, P3)."
size: "M"
area: components
updated: 2026-10-26
---

# Row 428: Themed wrappers forward with `..props`, not field by field

## Item

A themed wrapper forwards every prop of the primitive it renders (row 391). Today 826 forwarded-field lines in 47 `preview/src/components/*/component.rs` files are written out by hand, 816 of them identity (`x: props.x`), and a 25-entry allowlist plus a Rust scanner exist only to notice a field someone forgot. Construction: a wrapper that changes only `attributes` is `prim::Comp { attributes: merged, ..props }`, so a prop added to a primitive reaches the themed API with no edit. Explicit fields remain only where a wrapper deliberately changes a prop.

## Status / trigger

LANDED. Owner approved 2026-10-25 (row 365 sweep, P3).

## Size

M

## Detail

Source: [final sweep note, P3](../../research/final-sweep-2026-10-25.md).

- **Measured:** `grep -E "^\s+[a-z_#]+: props\.[a-z_#]+,?$" preview/src/components/*/component.rs`. Largest: `calendar/component.rs` 69, `date_picker` 60, `dropdown_menu` 52 (`dropdown_menu/component.rs:21-35` forwards eight fields, `:60-70` five), `context_menu` 51, `virtual_grid` 47, `combobox` 42, `select` 37. The allowlist is `scripts/check-wrapper-props.allow.tsv` (25 live entries, ten of them "Internal part; the primitive generates its id").
- **Precedent in the repo:** `chart/component.rs:67,98,126` forward with `chart::ChartContainer { attributes: merged, ..props }`; `:58` clones `props.attributes` first; `:85-86` cites `primitives/src/toast.rs`'s own `Toast { ..props }`.
- **Subsumes:** the ~800 identity-forward lines; the `*.id` allowlist entries whose only reason is "the wrapper does not touch it" (the spread forwards them, which is correct: the primitive's own id generation is unaffected when the prop is `None`). **Does not subsume:** deliberate changes (slider's `index` pinned to 0; `select::SelectValue.placeholder` passed conditionally; `color_picker` composing a popover with its own surface props). Those stay explicit with their allowlist reasons.
- **Gate:** `check-wrapper-props.sh` already treats `..props` as forwarding every prop. Add a rule that a wrapper call naming more than N fields all as `x: props.x` fails ("use `..props`"). The allowlist shrinks to the deliberate cases.
- **Friction (inferred):** Rust partial moves (`props.children` read after a spread). The `.clone()` pattern in `chart/component.rs` handles it. First step: convert three wrappers (`dropdown_menu`, `calendar`, one small one) and report the friction before the rest, which is mechanical.

## Outcome

- **Converted.** Every wrapper whose `props` type is the primitive's own struct now renders `prim::Comp { attributes: merged, ..props }` (161 call sites in 37 `preview/src/components/*/component.rs`; `chart` and `scroll_area`, 5 more, already did). Identity `x: props.x` lines: 828 -> 388, the 440 removed are the ones that could not be written any other way than by hand. A prop added to a primitive now reaches the themed API with no edit, and `check-wrapper-props`' allowlist did not need a new entry.
- **Friction, measured (three wrappers first, then the rest by script).** The rsx spread expands to `let mut __manual_props = props; __manual_props.<field> = <value>; ..`, so: (1) `props` is moved whole, so any `props.x` read in a field value or in the call's body after it is a use-after-move and every earlier read of a non-`Copy` field must be a clone (`props.attributes.clone()`, as `chart` did; `props.class.clone()` in the three class-merging wrappers; `let children = props.children.clone();` for the 20 wrappers that add an icon after `{props.children}`); (2) a field assignment takes the field's exact type where the builder setter took `impl Into`, so `class: String` becomes `class: Some(class)`; (3) Dioxus refuses a custom attribute (`"data-*": ..`) and a builtin-attribute field (`padding: ".."`) on a spread call; (4) a body child is assigned to `children`, so an `Option<Element>` children prop takes the explicit field `children: Some(rows)` (`DragAndDropListItems`) and `InputOtpSeparator`'s `if let Some(children)` fork collapses to one call, because `..props` already carries the `Option`.
- **Kept explicit, with the reason.** `ContextMenuTrigger` (builtin style attributes on the call), `Drawer` (custom `data-slot`), `PopoverContent` (custom `data-dx-surface`): the three entries of `check-wrapper-spread`'s `ALLOW`. Overrides kept beside the spread: `attributes: merged` everywhere, `class` (toggle, radio item, drawer content, navbar item), `class: None` (tab trigger and content, whose class is merged into the base attributes), `children: Some(rows)`.
- **Not convertible, left alone.** 388 identity lines in 26 files stay: those wrappers define their OWN props struct (calendar, date_picker, combobox, select, command, tabs, toggle, ... add `variant`, `size`, `trigger_class`, composite part props), so there is no struct to spread. `check-wrapper-props` still lists every primitive prop they do not pass. The row's largest-file figures (calendar 69, date_picker 60, combobox 42, select 37) were all of this kind.
- **Gate.** `scripts/check-wrapper-spread.sh` (in `run-gates.sh`, `dev-docs/gates.md` regenerated): a `#[component] fn F(props: P)` whose `P` is not a struct of the same file must have `..props` at the top level of its `module::F { .. }` / `module::C { .. }` (`C` + `Props` = `P`) call. Reasoned allowlist through `scripts/lib/allow.py`, stale entries fail, its fixture self-test runs on every invocation. Against the pre-change tree it reports 153 sites.
- **Props docs.** Unchanged by construction: `preview/props_scan.rs` reads the `#[derive(Props)]` structs, never a call site, so no table lost a row and the scanner needed no change; `check-props-docs` stays strict.
