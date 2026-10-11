The Button Group component visually merges a row (or column) of `Button`s into one control -- shared borders, and only the outer corners rounded.

## Component Structure

```rust
ButtonGroup {
    Button { variant: ButtonVariant::Outline, "Archive" }
    Button { variant: ButtonVariant::Outline, "Report" }
    // A visual divider between two clusters of buttons.
    ButtonGroupSeparator {}
    Button { variant: ButtonVariant::Outline, "..." }
}
```

## Joining any control

`ButtonGroup` joins whatever it holds, not just `Button`: `Input`, `Select`, `NativeSelect`, and the `DropdownMenu` and `Popover` triggers (which sit inside a wrapper `<div>`) all get flat seams and shared borders. It does this without styling any of them. On each direct child it sets four inheritable custom properties by position, and each joinable control's own stylesheet reads them with its normal radius as the fallback, so a control outside a group is unchanged.

| Variable | Set on | Flattens |
| --- | --- | --- |
| `--dx-group-radius-start-start` | non-first child (horizontal) / non-first child (vertical) | block-start, inline-start corner |
| `--dx-group-radius-end-start` | non-first child (horizontal) / non-last child (vertical) | block-end, inline-start corner |
| `--dx-group-radius-start-end` | non-last child (horizontal) / non-first child (vertical) | block-start, inline-end corner |
| `--dx-group-radius-end-end` | non-last child (horizontal) / non-last child (vertical) | block-end, inline-end corner |

The names are logical corners, so a right-to-left group mirrors with no extra rule. Overlay content (`[popover]`, `dialog`, `role="dialog" | "menu" | "listbox" | "tooltip"`) resets the four variables, so a `Button` or `Input` inside a popover panel does not inherit its trigger's seam. The 1px overlap between neighbours is a negative margin on the group's own direct child (the wrapper, when there is one).

A control that should join a group reads them in its own stylesheet:

```css
border-start-start-radius: var(--dx-group-radius-start-start, var(--dx-radius-lg));
border-start-end-radius: var(--dx-group-radius-start-end, var(--dx-radius-lg));
border-end-start-radius: var(--dx-group-radius-end-start, var(--dx-radius-lg));
border-end-end-radius: var(--dx-group-radius-end-end, var(--dx-radius-lg));
```
