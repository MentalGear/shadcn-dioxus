Color picker used when the user needs to make a customized color selection.

## Component Structure

```rust
ColorPicker {
    // The currently selected color in the color picker.
    color,
    on_color_change: move |c: Hsva<encoding::Srgb, f64>| {
        // This callback is triggered when a color is selected in the popover.
        // The parameter holds hue, saturation, value and alpha (1.0 is opaque).
    },

    // Optional: edit the alpha channel too.
    alpha: false,

    // Optional label on the trigger button.
    label: "Pick",
}
```

### Alpha

The color is a `palette::Hsva<encoding::Srgb, f64>`, so `color` and `on_color_change` carry alpha; an opaque
color has alpha `1.0`. With `alpha: true` the popover adds an "Alpha" slider (a checkerboard track fading from
transparent to the current color, `aria-valuetext` as a percentage) and the hex field shows `#RRGGBBAA` and accepts
`#RGB`, `#RGBA`, `#RRGGBB` (opaque) and `#RRGGBBAA`. With `alpha` off (the default) the field shows `#RRGGBB`, and
an alpha below 1 in `color` is kept as it is, not reset. The trigger and the swatches always paint the alpha over a
checkerboard. In the primitive, `ColorPickerContext::set_alpha` sets the channel and `set_hue` / `set_sv` keep it.

### Custom Popover

`ColorPicker` renders the default trigger and `ColorPickerSelect` content. If you
want to fully replace the default popover UI, compose the picker from
`ColorPickerRoot`, `ColorPickerTrigger`, and `ColorPickerPopover`:

```rust
ColorPickerRoot {
    color,
    on_color_change: move |c: Hsva<encoding::Srgb, f64>| { /* ... */ },

    ColorPickerTrigger {
        label: "Pick",
    }

    ColorPickerPopover {
        div {
            "Custom color picker content"
        }
    }
}
```

If you only need to add content after the default controls, pass children to
`ColorPicker`; they will be appended inside the default popover.
