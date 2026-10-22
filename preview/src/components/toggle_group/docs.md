The Toggle Group component is used to create a group of toggle buttons that allows the user to select one or more options from a set. It is useful for creating a set of options.

## Component Structure

```rust
// The ToggleGroup component wraps all toggle items in the group.
ToggleGroup {
    // The look of every item: ToggleVariant::Default or ToggleVariant::Outline.
    variant: ToggleVariant::Outline,
    // The size of every item: ToggleSize::Sm, ToggleSize::Default or ToggleSize::Lg.
    size: ToggleSize::Default,
    // The gap between items in 4px steps. The default 2 keeps separate toggles 8px apart;
    // 0 joins them into one segmented bar.
    spacing: 2u8,
    // The orientation of the toggle group, true for horizontal (the default) and false for vertical.
    horizontal: true,
    // More than one item may be pressed (the default is single-select, radio-style).
    allow_multiple_pressed: true,
    // The values of the items pressed at first (shadcn's `defaultValue`). Drive the group instead with
    // `value: Some(set)` and `on_value_change: move |set: HashSet<String>| ...` (controlled).
    default_value: HashSet::from(["bold".to_string()]),
    // The toggle item represents each individual toggle button in the group.
    ToggleItem {
        // The item's identity in the pressed set (shadcn's item `value`), unique within the group.
        value: "bold",
        // The position of the toggle item, used only to order keyboard focus.
        index: 0,
        // The contents of the toggle item button
        {children}
    }
}
```

## Pressed state follows the value

The pressed set is a `HashSet<String>` of item `value`s, like the selection of every other collection (the Data Table's row keys), so reordering the items or inserting one before the others never moves the pressed state to a different button. `index` only orders keyboard focus. The group is single-select unless `allow_multiple_pressed` is set.

`default_pressed`, `pressed` and `on_pressed_change` (a `HashSet<usize>` of positions) are deprecated and still work: an item without a `value` is keyed by its index.

## Variant, size and spacing

`variant` and `size` are `ToggleVariant` and `ToggleSize` from the Toggle component, and every item inherits them (shadcn's `ToggleGroup` context): the group redeclares the Toggle's slot variables and the items read them, so an item is a Toggle sized and edged by its group.

`spacing` is the gap in 4px steps (shadcn's `spacing`, default 2). With `spacing: 0` the items join into one bar: square corners with the radius on the two ends, and, with `Outline`, one shared 1px edge between neighbours (a vertical bar shares its top edges instead).

## Direction / RTL

`ToggleGroup` accepts a `dir: Option<Direction>` prop. When `horizontal: true`, `ToggleItem`'s `ArrowLeft`/`ArrowRight` roving focus swaps under RTL. See the `rtl` variant.
