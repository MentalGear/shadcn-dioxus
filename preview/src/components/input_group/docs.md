The Input Group component wraps an `InputGroupInput` with leading/trailing `InputGroupAddon`s (icons, buttons, static text) inside one bordered control.

## Component Structure

```rust
InputGroup {
    InputGroupAddon {
        Search {}
    }
    InputGroupInput { placeholder: "Search..." }
}
```

## Alignment

`InputGroupAddon { align }` takes `InputGroupAlign::Start` (the default) and `End`, the inline edges beside the control, and `BlockStart` and `BlockEnd`, full-width rows above and below it:

```rust
InputGroup {
    InputGroupInput { placeholder: "Ask anything" }
    InputGroupAddon { align: InputGroupAlign::BlockEnd,
        InputGroupText { "Enter to send" }
        Button { style: "margin-inline-start: auto;", size: ButtonSize::IconSm, ArrowUp {} }
    }
}
```

A group that holds a block addon stacks its children in a column and sizes to them; the addon's `order` places it above (`BlockStart`) or below (`BlockEnd`) the control wherever it appears in the markup. This is the shape of a chat composer: the text on top, the actions in a bottom row.
