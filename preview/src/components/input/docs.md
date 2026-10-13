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

`InputGroup` composes `Input` through two slot variables instead of overriding its properties (so no stylesheet order decides anything):

| Variable | Default | Controls |
| --- | --- | --- |
| `--dx-input-disabled-opacity` | `var(--dx-opacity-disabled)` | `opacity` of a disabled input |
| `--dx-input-invalid-shadow` | `var(--dx-ring-invalid-field)` | `box-shadow` of an input with `aria-invalid="true"` |
