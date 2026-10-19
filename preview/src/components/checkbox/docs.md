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
