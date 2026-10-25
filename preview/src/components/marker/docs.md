The Marker component is an inline conversation marker: a streaming status ("Thinking..."), a tool activity line, a system note, a bordered row or a labeled separator. Compose it with `Message` in a conversation thread.

## Component Structure

```rust
Marker {
    // Available variants: Default, Border, Separator
    variant: MarkerVariant::Default,

    // Pass `role: "status"` for a streaming or in-progress marker
    role: "status",

    MarkerIcon { Spinner {} }
    MarkerContent {
        // The `dx-shimmer` class (assets/dx-effects.css) animates streaming text
        class: "dx-shimmer",
        "Thinking..."
    }
}
```

To make the whole marker a link or button, pass `as` (Radix's `asChild`). The callback gets an `AsChild`: spread its `attributes` on your element and render its `children` (the marker's icon and content) inside it:

```rust
Marker {
    r#as: |el: AsChild| rsx! {
        a { href: "/files", ..el.attributes, {el.children} }
    },
    MarkerIcon { FileText {} }
    MarkerContent { "Explored 4 files" }
}
```

## Accessibility

- A marker is presentational by default. Set `role: "status"` on a streaming or in-progress marker so assistive tech announces the update as it appears.
- A labeled separator needs no role: the divider lines are decorative pseudo-elements and the text is read as ordinary content. Do not add `role: "separator"`, which would hide the label.
- `MarkerIcon` is `aria-hidden`; the adjacent `MarkerContent` carries the meaning. An icon-only marker needs an `aria-label` on the `Marker`.
- A marker that links or acts must be a real `a` or `button` (through `as`), so it is focusable and has the right role.

## Motion

`MarkerContent` with the `dx-shimmer` class sweeps a highlight across the text (`dx-effects.css`; tune with `--dx-shimmer-duration`, `--dx-shimmer-color`, `--dx-shimmer-spread` and `--dx-shimmer-angle`; `prefers-reduced-motion` turns it off). The sweep is paused whenever the text is off-screen, in a skipped `content-visibility` subtree or in a hidden tab (`MarkerContent` calls `use_motion_when`); a marker without the class subscribes to nothing. See [Motion](/docs#motion) for the tokens, reduced motion and the per-component table.
