The Checkbox component is used to create an accessible checkbox input.

## Component Structure

```rust
// The themed Checkbox renders a button that already contains its own indicator: a check
// for checked and a minus for indeterminate. It takes no children, so there is nothing to
// put inside it.
Checkbox {
    name: "terms",
    default_checked: CheckboxState::Unchecked,
    on_checked_change: move |state: CheckboxState| { /* ... */ },
}
```

## Indeterminate

Pass `CheckboxState::Indeterminate` (`checked` or `default_checked`) for a "some selected" parent box: it is filled like a checked box and shows a minus instead of the tick, with `aria-checked="mixed"`. A click on an indeterminate box unchecks it (the next state is `Unchecked`, delivered to `on_checked_change`); a controlled parent box that wants "select all" instead sets `Checked` itself in the handler.

## Outside its form

Pass `form: "form-id".to_string()` (the HTML `form` attribute) and the control's hidden mirror input belongs to that `<form>` even when it is rendered elsewhere in the page: it submits with it and the form's reset restores it. Leave it out and the nearest ancestor `<form>` owns it.

## Read-only

`read_only: true` keeps the checkbox focusable and announced but unchangeable: a click or Space does nothing, the element carries `aria-readonly="true"` and a `data-readonly` presence attribute (absent when the checkbox is writable), and its value still submits with a form. It differs from `disabled`, which leaves the tab order, is not announced as editable and submits nothing. A form reset still restores the default. See the `read_only` variant.
