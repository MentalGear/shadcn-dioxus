The Checkbox component is used to create an accessible checkbox input.

## Component Structure

```rust
// The checkbox component creates a checkbox button
Checkbox {
    // The checkbox indicator is a child component which will only be visible when the checkbox is checked.
    CheckboxIndicator {
        // The content of the checkbox indicator, typically an icon or image.
        {children}
    }
}
```

## Indeterminate

Pass `CheckboxState::Indeterminate` (`checked` or `default_checked`) for a "some selected" parent box: it is filled like a checked box and shows a minus instead of the tick, with `aria-checked="mixed"`. Clicking it unchecks it.

## Outside its form

Pass `form: "form-id".to_string()` (the HTML `form` attribute) and the control's hidden mirror input belongs to that `<form>` even when it is rendered elsewhere in the page: it submits with it and the form's reset restores it. Leave it out and the nearest ancestor `<form>` owns it.
