The glyphs the other components draw (chevrons, check, close, search, sort arrows and a few more), as Dioxus components in one file. A component that needs one imports it from here, so installing a component adds this file and no icon crate.

## Component Structure

```rust
use crate::components::icons::{Check, X};

Check { size: "1rem" }
X { size: "14px", class: "my-close", "aria-hidden": "true" }
```

Every glyph renders one inline `<svg>` on a 24 x 24 view box with a 2px round stroke in `currentColor`, so it follows the text colour of its parent. `size` sets the width and height (a pixel count or any CSS length, 24 by default); every other SVG attribute you pass (`class`, `stroke`, `aria-hidden`, `data-*`) lands on the `<svg>` and wins over the default.

## Adding a glyph

Add a `glyph!` block to `component.rs` with the shapes copied from the Lucide icon of the same name (<https://lucide.dev>). Lucide is ISC-licensed; the notice at the top of the file stays with it, plus the MIT notice of the Feather project for the glyphs Lucide derived from it.

## Accessibility

A glyph carries no name. Inside a button that has a text name, pass `"aria-hidden": "true"`; for an icon-only control, put the accessible name on the control (`aria_label`), not on the glyph.

## The full Lucide set

The file holds only the glyphs the components use. An app that wants any other Lucide icon adds the `dioxus-icons` crate itself and imports from `dioxus_icons::lucide`; nothing here depends on it.
