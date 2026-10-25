The Combobox component is an autocomplete input with a filterable popup list.

Filtering preserves the document order of the rendered `ComboboxOption` elements. If you want
query-dependent ranking, control `query`, sort your item data in user code and render the options
in that sorted order. Options that are reordered while mounted (a keyed list) keep their first
place, so such a list sets each option's `index` from the sorted list.

Selecting several options (chips, clear button, form value) is `ComboboxMulti`: its demos come first under Examples below, and its API is in "Multiple selection".

## Behavior

The input shows the selected option's label. Clicking, focusing or arrowing into it never rewrites
that text: the caret lands where the user clicked, the popup lists **every** option, and the selected
one is marked and scrolled into view. Filtering starts with the first edit, and from then on the
input's own text is the query. Escape, blurring, or picking an option ends the edit and puts the
selected label back.

## Component Structure

```rust
let mut value = use_signal(|| None::<String>);
let mut query = use_signal(String::new);

Combobox::<String> {
    value: Some(value.into()),
    item_to_label: |value: String| label_of(&value), // required: the input's text for the value, read at render time
    on_value_change: move |next: Option<String>| {
        value.set(next);
    },
    query: Some(query()),
    on_query_change: move |next| query.set(next),
    placeholder: "Select framework...",
    aria_label: "Select framework",
    list_aria_label: "Frameworks",
    ComboboxEmpty { "No framework found." }
    ComboboxOption::<String> {
        value: "next".to_string(),
        text_value: "Next.js",
        "Next.js"
    }
}
```

`item_to_label` (required on `Combobox` and `ComboboxMulti`, like Base UI's `itemToStringLabel`) is the selected
value's display text. The input's value reads it at render time, never the option registry (which fills in a
client-only effect), so a preselected `Combobox` carries `value="Next.js"` in the server-rendered HTML instead of
an empty input until hydration, and the input cannot flash empty while the list re-registers its options.

## Multiple selection

`ComboboxMulti` selects several options and shows them as removable chips (shadcn's `multiple` +
`chips` combobox). It takes and reports a `Vec<T>` (`values` / `default_values` / `on_values_change`)
where `Combobox` takes `Option<T>`: the two roots have different *types*, so the operations that only
exist on a set (remove one, clear all, Backspace on an empty input, chips) cannot be reached from a
single-select root, and a single root's callback can never see a list. Both share the same input, list,
options and empty state.

```rust
ComboboxMulti::<String> {
    default_values: vec!["next".to_string()],
    item_to_label: |value: String| label_of(&value), // required: the chip's text, read at render time
    name: Some("frameworks".to_string()), // one form entry per selected value
    show_clear: true,                     // a clear button while anything is selected
    placeholder: "Choose frameworks...",
    aria_label: "Frameworks",
    ComboboxEmpty { "No framework found." }
    ComboboxOption::<String> { value: "next".to_string(), text_value: "Next.js", "Next.js" }
}
```

- `item_to_label` (required, like Base UI's `itemToStringLabel`) is a selected value's display text. Chips and their
  remove buttons' accessible names read it, never the option registry, so a preselected chip carries its text in
  the server-rendered HTML instead of rendering empty until hydration.
- The list stays open while options are toggled; picking a selected option removes it. The typed text is the
  query only and is cleared after each pick and when the popup closes.
- The listbox carries `aria-multiselectable="true"` (the single list never does); every option keeps
  `aria-selected`.
- Keyboard: Backspace in the empty input removes the last chip. The "previous" arrow (ArrowLeft in LTR,
  ArrowRight in RTL) moves from the empty input onto the chips' remove buttons, the arrows move between
  them (the "next" arrow past the last chip returns to the input), Enter/Space/Backspace/Delete remove
  the focused chip and return focus to the input. The chips field is the popup's anchor and width.
- Forms: with `name`, every selected value is submitted under it (`FormData.getAll(name)`), as the value's
  `ToString`; `<form reset>` restores `default_values`. Values need `T: ToString`.
- Set `--dx-combobox-width` on the root to size the field and the list together.

## Motion

The list fades and scales in and out over `--dx-floating-duration` (the shared `dx-pop-in`/`dx-pop-out`). Retune with those tokens. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
