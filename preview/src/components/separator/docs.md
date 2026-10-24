The Separator component is a simple horizontal or vertical line that can be used to visually separate content in your application.

## Component Structure

```rust
// The Separator component can be oriented horizontally or vertically.
Separator {
    // The orientation of the separator line. true for horizontal, false for vertical.
    horizontal: true,
    // The decorative property controls if the separator is decorative and should not be visible to screen readers.
    decorative: false,
}
```

## Slot variables

A component that composes `Separator` (the sidebar, the button group, an item group) retunes it by setting a custom property on the separator's element, never by declaring `margin` or `background` on its own class: two stylesheets declaring the same property on one element are decided by which sheet loads last, and that order differs from route to route. `.dx-separator` reads each of these with its own default, so a variable that is not set changes nothing. The size along the axis (`width` of a horizontal rule, `height` of a vertical one) is the orientation's, not a slot.

| Variable | Default | Property |
| --- | --- | --- |
| `--dx-separator-margin` | `0` | `margin` |
| `--dx-separator-background` | `var(--dx-border)` | `background-color` |
