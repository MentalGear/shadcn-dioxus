A popover shows rich content in a panel anchored to its trigger. The content is arbitrary: a form, a short explanation, a set of actions. It is not a menu; reach for `DropdownMenu` when the content is a list of commands.

`PopoverRoot` is **non-modal by default** (`is_modal: false`), like shadcn's (Radix / Base UI) popover. Focus moves into the panel when it opens and returns to the trigger on Escape or close. The page behind stays interactive and scrollable: it is not inert and the page scroll lock is never taken. One click outside closes it and that same click reaches its target; the browser's own light dismiss (`popover="auto"`) does the closing, so there is no swallowed first click.

Pass `is_modal: true` for a modal popover (the `modal` variant): a native `<dialog>` opened with `showModal()`. Focus is trapped, the page behind it is inert, the page scroll is locked while it is open, and a click outside only dismisses it. A modal popover does not dim the page either, like shadcn's: `overlay` defaults to `false`, and `overlay: true` paints the same scrim every modal overlay shares behind the panel (the `overlay` variant). `overlay` only affects a modal popover; a non-modal one has no overlay to switch.

> **Breaking change:** `is_modal` used to default to `true`. A `PopoverRoot` that relied on the old default (focus trap, inert page, scroll lock, a backdrop click that does not reach the page) now behaves like shadcn's popover instead. Add `is_modal: true` to keep the old behaviour. `DatePickerPopover` follows the same default flip (it already forwarded an explicit `is_modal: false` in the styled pickers).

## Component Structure

```rust
// The PopoverRoot is the root component that contains the trigger and content.
// `is_modal` (default false, like shadcn) keeps the page interactive and
// scrollable; `is_modal: true` traps focus, makes the page inert and locks the
// page scroll, and `overlay: true` then also dims the page with the shared scrim.
PopoverRoot {
    // The PopoverTrigger is the button that opens the popover (styled like the outline Button).
    PopoverTrigger {
        "Open popover"
    }
    // The PopoverContent is the panel shown when the popover is open.
    PopoverContent {
        side: ContentSide::Bottom,
        align: ContentAlign::Center,
        // Optional: a header with a title and description, then your own content.
        PopoverHeader {
            PopoverTitle { "Dimensions" }
            PopoverDescription { "Set the dimensions for the layer." }
        }
        {children}
    }
}
```

## Motion

The panel fades in and out over `--dx-overlay-duration` (default `--dx-motion-duration-slow`) with `--dx-overlay-ease`. A modal popover uses the shared dialog exit driver, staying open until its exit settles (at most 1500 ms) before it closes. The keyframes animate `opacity` only, so the anchor's own `transform` centring is never replaced. Reduced motion swaps in `--dx-motion-duration-reduced`. Nothing loops. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.

## Initial and final focus

`initial_focus` and `final_focus` take a function of the open modality (`OpenModality::{Keyboard, Mouse, Touch, Pen}`, Base UI's `initialFocus(openType)`) that returns a `FocusTarget`: `Default`, `Panel`, `Selector("#id")` or, for `final_focus`, `None`. They are inputs to the one open-focus policy (`dxOpenFocus` / `dxCloseFocus`, `primitives/src/modality.rs`), not a second code path: the function is called once per modality at render, the answers ride on the panel as `data-dx-initial-focus` / `data-dx-final-focus`, and the policy reads the entry for the modality of the open. A selector that matches nothing falls back to the built-in policy (keyboard and mouse: the first control; touch and pen: the panel, so no software keyboard rises). An explicit target is honoured as given, so return `Panel` for `Touch` yourself when it names a field.

```rust
PopoverContent {
    initial_focus: Callback::new(|m: OpenModality| match m {
        OpenModality::Touch => FocusTarget::Panel,
        _ => FocusTarget::selector("#name"),
    }),
    final_focus: Callback::new(|_| FocusTarget::Default),
}
```

`final_focus` defaults to the element that held focus when the panel opened: the trigger for a click open, the previously focused element for a programmatic open. The opener is remembered in exactly one place (`dxOpenFocus`), for every overlay kind. See the `focus_target` variant.
