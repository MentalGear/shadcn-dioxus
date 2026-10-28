The RadioGroup component is used to create a group of radio buttons that allows the user to select one option from a set.

## Component Structure

```rust
// The RadioGroup component wraps all radio items in the group.
RadioGroup {
    // The value property represents the currently selected radio button in the group.
    value: "option1",
    on_value_change: |value: String| {
        // This callback is triggered when the selected radio button changes.
        // The value parameter contains the value of the newly selected radio button.
    },
    // The RadioItem component represents each individual radio button in the group.
    RadioItem {
        // The value of the radio button, which is used to identify the selected option and will be passed to the on_value_change callback when selected.
        value: "option1",
        // The contents of the radio item button
        {children}
    }
}
```

## Direction / RTL

`RadioGroup` accepts a `dir: Option<Direction>` prop. When `horizontal: true`, `RadioItem`'s `ArrowLeft`/`ArrowRight` roving focus swaps under RTL the same way `Tabs`' does. See the `rtl` variant.

## Outside its form

Pass `form: "form-id".to_string()` (the HTML `form` attribute) and the control's hidden mirror input belongs to that `<form>` even when it is rendered elsewhere in the page: it submits with it and the form's reset restores it. Leave it out and the nearest ancestor `<form>` owns it.

## Read-only

`read_only: true` on the group keeps its selection fixed while the items stay focusable: a click or Space does not select an item, the arrow keys still move focus without selecting, the group carries `aria-readonly="true"` and a `data-readonly` presence attribute (the items carry `data-readonly` too), and the selected value still submits with a form. It differs from `disabled`, which leaves the tab order and submits nothing. A form reset still restores the default. See the `read_only` variant.
