A Tag Group is a focusable group of tags (labels, categories, filters and similar items) with keyboard navigation, optional selection and removal.

## Structure

Single selection with [`TagGroup`](component.rs):

```rust
TagGroup {
    value: Some(value.into()),
    on_value_change: move |value| { /* ... */ },
    on_remove: move |removed: Vec<String>| { /* drop them from your list */ },
    TagGroupLabel { "Labels" }
    TagList {
        TagGroupEmpty { "No tags" }
        Tag { index: 0usize, value: "bug", "bug" RemoveButton {} }
        Tag { index: 1usize, value: "feature", disabled: true, "feature" }
    }
}
```

## Removal

A tag with a `RemoveButton` child is removable, by the button or by Delete/Backspace on the focused tag (which removes every selected removable tag when the focused one is selected). The tag hides itself and `on_remove` reports the removed values, in list order, so an app that owns the list drops them there (the `multi` demo). Removing a selected tag also updates the selection through `on_value_change`/`on_values_change`.
