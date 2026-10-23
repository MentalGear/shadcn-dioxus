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

## Buttons

`InputGroupButton` (shadcn's `InputGroupButton`) is the themed `Button` with the addon's metrics: ghost and `type="button"` by default, `size: InputGroupButtonSize::{Xs (default), Sm, IconXs, IconSm}` (24px text, 28px text, a 24px square, a 32px square), text-sm, and a radius 3px under `--radius` on the two xs sizes. `variant` takes any `ButtonVariant`.

```rust
InputGroup {
    InputGroupInput { value: "https://ui.shadcn.com/docs", readonly: true }
    InputGroupAddon { align: InputGroupAlign::End,
        InputGroupButton { size: InputGroupButtonSize::IconXs, "aria-label": "Copy link", Copy {} }
    }
}
```

## Textarea and custom controls

`InputGroupTextarea` is the multi-line control (shadcn's `InputGroupTextarea`). It composes `Textarea`, drops the textarea's own fill, edge and resize handle, and leaves the group's outer edge to draw the chrome. Add a `BlockEnd` addon for a footer:

```rust
InputGroup {
    InputGroupTextarea { placeholder: "Write a message...", oninput: move |e| text.set(e.value()) }
    InputGroupAddon { align: InputGroupAlign::BlockEnd,
        InputGroupText { "{count} / 120" }
        Button { style: "margin-inline-start: auto;", size: ButtonSize::IconSm, ArrowUp {} }
    }
}
```

Any other element can be the group's control: give it the `dx-input-group-control` class and strip its own chrome. The group's hover, focus-visible, invalid (`aria-invalid="true"`) and disabled states follow it, and the addons sit around it. The `custom_control` demo uses a plain `textarea` styled by the caller.
