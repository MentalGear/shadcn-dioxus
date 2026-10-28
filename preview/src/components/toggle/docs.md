The Toggle component is used to create a toggle button that can be pressed or unpressed. It is useful for creating a binary option, such as enabling or disabling a feature.

## Component Structure

```rust
// The Toggle component is used to create a toggle button that can be pressed or unpressed.
Toggle {
    // The look: ToggleVariant::Default (no edge, the default) or ToggleVariant::Outline (a 1px edge).
    variant: ToggleVariant::Outline,
    // The size: ToggleSize::Sm (28px), ToggleSize::Default (32px) or ToggleSize::Lg (36px).
    size: ToggleSize::Sm,
    // Icon-only and icon-plus-text toggles need a name for assistive technology.
    aria_label: "Toggle bookmark",
    // The contents of the toggle button.
    "Bookmark",
}
```

## Variants and sizes

`variant` and `size` follow shadcn's `Toggle`. The height is the theme's control ladder (`--dx-control-height-sm` / `-default` / `-lg`, the same 28 / 32 / 36px as `Button`); `Sm` also uses 0.8rem text and a 14px icon. A child marked `"data-icon": "inline-start"` (or `"inline-end"`) trims the padding on that side to 8px (6px at `Sm`), as shadcn's `has-data-[icon]` rules do.

An icon that should fill with the ink while the toggle is pressed (shadcn's `group-aria-pressed/toggle:fill-foreground`) carries `"data-pressed-fill": "true"`.

## State attributes

A pressed toggle carries `aria-pressed="true"`, `data-state="on"` and a `data-pressed` presence attribute; an unpressed one carries `aria-pressed="false"`, `data-state="off"` and no `data-pressed`. Style on `[aria-pressed="true"]` (as the shipped sheet does), `[data-state="on"]` or `[data-pressed]`. `ToggleGroup` items are toggles, so they carry the same attributes.
