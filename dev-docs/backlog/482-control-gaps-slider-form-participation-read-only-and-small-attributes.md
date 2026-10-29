---
id: 482
title: "Control gaps: Slider form participation, read_only, Toggle data-pressed, Tabs activate_on_focus"
status: "IMPLEMENTED on lane/control-gaps-482, awaiting the merge batch (2026-10-28). Items 1 to 4 are…"
size: "S-M"
area: components
updated: 2026-10-28
---

# Row 482: Control gaps against both upstreams

## Item

Four behaviours that Radix and Base UI both have (or that Base UI has and base-nova uses) and this library lacks. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix B.

## Status / trigger

IMPLEMENTED on `lane/control-gaps-482`, awaiting the merge batch (2026-10-28). Items 1 to 4 are built: Slider and RangeSlider `name`/`form`, one hidden `type=hidden` input per thumb (the first carries the id the shared form-reset listener uses, so a reset restores `default_value`), `on_value_commit` (pointer release after a press that changed a value, each keyboard step that changes one; not a reset) and `RangeSlider::min_steps_between_thumbs` (one `bounds_for` serves clamping and `aria-valuemin/max`); `read_only` on Checkbox, Switch and RadioGroup (`aria-readonly` plus a `data-readonly` presence attribute; the group carries `aria-readonly`, its items only `data-readonly`, because `aria-readonly` is not allowed on `role=radio`; a radio group's arrow keys still move focus); Toggle `data-pressed` (ToggleGroup items inherit it); Tabs `activate_on_focus`. Not done, as the row marks them optional: Tabs `data-active`/`data-hidden`, the Slider push collision, `edge` alignment and Control part. The row 481 helper does not exist yet, so the presence attributes are written with `then_some("")` at each site; row 481 folds them into its helper.

## Size

S-M

## Detail

1. **Slider form participation (M).** No `name`, no hidden input (verified by grep in `primitives/src/slider.rs`). Add a `name` and a hidden input per thumb (Radix: a bubble input per thumb; Base UI: a visually hidden `<input type=range>` in each thumb, `SliderThumb.mjs`), `on_value_commit` (Radix `onValueCommit`, Base UI `onValueCommitted`) and `min_steps_between` thumbs. Optional and not part of this row: push collision, `edge` thumb alignment, a Control part. The form-reset listener in `lib.rs` and `default_checked.rs` are shared files.
2. **`read_only` on checkbox, switch and radio group (S).** Base UI `readOnly` sets `aria-readonly` and `data-readonly` and blocks changes; absent everywhere here. Uses the row 481 helper for the attribute.
3. **`data-pressed` on Toggle and ToggleGroup items (S).** `aria-pressed` and `data-state` exist; base-nova styles through `aria-pressed:`, so this is vocabulary, not a fix.
4. **`activate_on_focus` on Tabs (S).** Ours activates manually (Base UI's default); add the opt-in for automatic activation (Radix's default). Optional with it: `data-active` on the tab, `data-hidden` on the panel.

- **Files owned:** `primitives/src/{slider,checkbox,switch,radio_group,toggle,toggle_group,tabs}.rs`, their themed components and specs. Do not touch `lib.rs` beyond the form-reset listener.
- **Proof:** a form-submission spec for the slider (single and range, with `name`); a spec per `read_only` control; a Tabs spec for focus-activation.
