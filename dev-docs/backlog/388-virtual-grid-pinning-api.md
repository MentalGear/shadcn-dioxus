---
id: 388
title: "Virtual Grid: explicit row and column pinning in the API and the docs (pinned_columns_start, pinned_columns_end, pin_header)"
status: "Owner decision 2026-10-24 (Tom): \"API and docs\". Pre-1.0, so the breaking rename is accepted. DONE…"
size: "M"
area: components
updated: 2026-10-24
---

# Row 388: Virtual Grid pinning API

## Item

Make `VirtualGrid`'s row and column pinning explicit in the API and the docs. The header row was pinned implicitly
whenever `render_header` was passed (no way to have a header that scrolls away), column pinning was one bool
(`sticky_first_column`, exactly one column at the inline start), and the docs mentioned it only in a quick-start
comment and the opening paragraph, in "sticky" vocabulary while shadcn and TanStack Table users search for "pin"
(`columnPinning: { left, right }`).

## Status / trigger

Owner decision 2026-10-24 (Tom): "API and docs". Pre-1.0, so the breaking rename is accepted. DONE on
`lane/grid-pinning`, merge pending.

## Size

M

## Detail

API as shipped (`primitives/src/virtual_grid.rs`): `pinned_columns_start: usize` (default 0, replaces
`sticky_first_column`), `pinned_columns_end: usize` (default 0) and `pin_header: bool` (default true; false lets the
header scroll with the body). The two counts are clamped to the column count, the start side first.

Construction:
- A `Pinning` value (the clamped counts and the widths of the pinned columns) is the one place the offset arithmetic
  lives: start columns stack from the inline start at the cumulative width of the pinned columns before them, end
  columns from the inline end, each cell `position: sticky` with `inset-inline-start` / `inset-inline-end`, so both
  sides follow RTL through logical properties. The innermost end column takes the free space before it
  (`margin-inline-start: auto`), so the end columns reach the inline end of a grid narrower than its container.
- The columns engine (`primitives/src/virtual/grid.rs`) indexes only the scrolling columns, between the two pinned
  sides; `Pins` carries what covers the viewport (the pinned header, the inline width of both pinned sides) and the
  px of an unpinned header that precede the body. Pinned columns are always mounted outside the column window and
  rendered once.
- Keyboard: the focus-and-reveal script takes one `Cover` (header height, pinned header px, start px, end px) and
  scrolls the active cell clear of all of them, in either direction; the cover is read when the effect runs, so a
  toggled `pin_header` or a changed count applies to the next move.
- ARIA: the first column's cells are `role="rowheader"` only when `pinned_columns_start >= 1`.
- Attributes: `data-grid-sticky` is replaced by `data-grid-pinned="start|end"` plus `data-grid-pin-edge="true"` on the
  innermost pinned column of each side, which `style.css` uses for the separating edge.
- Sheet: pinned cells are opaque; a hovered or `data-state="selected"` row paints its fill on its pinned cells as a
  `background-image` layer over the opaque base, through `--dx-press-under` (check-press-under), inside the hover
  media query for the hover half. The header row is excluded by structure, as in row 386.

Docs: a "Pinning rows and columns" section (example, props table, windowing, appearance, keyboard, RTL, ARIA, data
attributes), the quick-start, the layouts table and the Styling paragraph; the `pinning` demo variant (start and end
columns, a `pin_header` switch, selection through a pinned checkbox column).

Tests: unit tests for the offset arithmetic, clamping, styles and the engine's pins in `virtual_grid.rs` and
`virtual/grid.rs`; `playwright/virtual-grid-pinning.spec.ts` (x positions in LTR and RTL while scrolling sideways,
mounted once, header pinned and unpinned, the active cell clear of every pinned region for the arrows, Home, End and
PageDown, opaque pinned cells, hover and selected fill).
