---
id: 306
title: "Select and Combobox show no selected label before hydration"
status: "OPEN. Found 2026-10-19 by the features-batch validation; pre-existing on main."
size: "M"
area: select
updated: 2026-10-19
---

# Row 306: Select and Combobox show no selected label before hydration

## Item

With JavaScript off, and on first paint before hydration, a preselected `Select` or single `Combobox` shows no selected label. `Select` shows its placeholder and the single `Combobox` input is empty, even though a value is selected.

## Status / trigger

OPEN. Found 2026-10-19 by the features-batch validation; pre-existing on main.

## Size

M

## Detail

**Cause.** Both read the label of the selected value from the option registry. `Select` uses `selected_text()` for `SelectValue` and the trigger's fallback aria-label, and the single `Combobox` uses `selected_label()` for the input value. Options register in a client-only `use_effect` (`primitives/src/selectable.rs`), so during SSG the registry is empty when the value renders.

**Where it shows.** The Select rtl demo (`default_value: "first"`) renders "Select an option" with `data-placeholder=false`, and the trigger's aria-label falls back to "Select an option". The Combobox "Controlled framework" demo starts at `Some("svelte")` and renders `value=""`. `first-paint-styles.spec.ts` compares styles, not text, so no spec flags either case.

**Construction already used.** `ComboboxMulti` had the same gap for its chips (empty label, `aria-label="Remove "`). It was fixed in the features batch (commit 1299d89d) with a required `item_to_label: Callback<T, String>` on the root, which mirrors Base UI's `itemToStringLabel`. The registry no longer decides that text.

**Wanted.** Apply the same construction to `Select` and `Combobox`: a label source available at render time that the registry cannot override. This breaks the API of every demo. `Select` values are often `&str` or an enum with no label mapping, so weigh a required prop against a `T: Display` default. Add a first-paint TEXT parity check (JS off vs hydrated `textContent`/`value`/`aria-label` of the selected value) so the whole class is gated, not just these two components.
