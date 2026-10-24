The input element is used to accept a text input from the user. It can be styled and configured to fit various use cases, such as forms or search bars.

## Component Structure

```rust
input {
    // Global html attributes
    class: "dx-input",
    // Children
    {children}
}
```

## Slot variables

A component that composes `Input` (such as `InputGroup` and `DataTable`'s filter) retunes it by setting a custom property on the input's own element, never by declaring the property on its own class: two stylesheets declaring one property on one element are decided by which sheet loads last, and that order differs from route to route. `.dx-input` reads each of these with its own default, so a variable that is not set changes nothing.

| Variable | Default | Controls |
| --- | --- | --- |
| `--dx-input-background` | `var(--dx-field-fill)` | `background-color` in every state (rest, hover, focus-visible, disabled); each state keeps its own fill when unset |
| `--dx-input-box-shadow` | the 1px `--dx-input` edge | `box-shadow` at rest; the focus-visible state reads it too and defaults to `var(--dx-ring-field)` |
| `--dx-input-radius` | `var(--dx-radius-lg)` | all four corners, ahead of a group's per-position `--dx-group-radius-*` |
| `--dx-input-max-width` | `none` | `max-width` |
| `--dx-input-disabled-opacity` | `var(--dx-opacity-disabled)` | `opacity` of a disabled input |
| `--dx-input-invalid-shadow` | `var(--dx-ring-invalid-field)` | `box-shadow` of an input with `aria-invalid="true"` |

```css
/* In the composing component's own stylesheet, on the input's element: */
.dx-data-table-filter { --dx-input-max-width: 20rem; }
```
