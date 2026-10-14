The Switch component allows users to toggle between two states, such as on and off.

## Component Structure

```rust
// The Switch component includes the switch thumb.
Switch {
    // The current state of the switch, true for on and false for off.
    checked: true,
    // Callback function triggered when the switch state changes.
    on_checked_change: |checked: bool| {
        // Handle the change in switch state.
    }
}
```

## Size

`size: SwitchSize::Sm` shrinks the track to 24 x 14px with a 12px thumb (`SwitchSize::Default` is 32 x 18.4px, 16px thumb), shadcn's `size="sm" | "default"`. It sets `data-size` on the switch; the style sheet changes three slot variables (`--dx-switch-width`, `--dx-switch-height`, `--dx-switch-thumb-size`), so the thumb travel follows. See the `sizes` variant.
