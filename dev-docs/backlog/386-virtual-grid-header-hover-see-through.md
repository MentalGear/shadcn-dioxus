---
id: 386
title: "Virtual Grid: hovering the sticky header row makes it see-through (a translucent row hover replaced the opaque cover)"
status: "FIXED on lane/grid-header-hover, merge pending."
size: "S"
area: components
updated: 2026-10-24
---

# Row 386: Virtual Grid header row hover

## Item

Owner report (2026-10-24): on a scrolled Virtual Grid, the pointer over the sticky header row turns the header
see-through, so body cell numbers paint through the header labels.

## Status / trigger

FIXED on `lane/grid-header-hover`, merge pending.

## Size

S

## Detail

Root cause: `virtual_grid/style.css` painted the Nova row hover (`--dx-table-hover-background`, muted at 50 %,
translucent) on every `[role="row"]` of a data grid. The sticky header is itself a `[role="row"][data-grid-header]`
whose opaque `--dx-background` is what covers the body cells scrolling under it, and the hover rule out-ranks that
declaration, so the cover became translucent while the pointer was over it. Only the header row is affected: the
pinned header cell (`data-grid-sticky`) has its own opaque background and is not a `[role="row"]`.

Construction: the hover rule selects body rows by structure (`[role="row"]:not([data-grid-header])`), so no
translucent hover can reach the cover whatever the source order or specificity. `playwright/virtual-grid.spec.ts`
asserts the header row and the pinned header cell compute the same opaque background hovered and at rest while the
grid is scrolled, and that a body row still takes the hover fill.

Other renderers checked: `Table` paints `.dx-table-row:hover` on the header row too, but its sticky header cells
(`.dx-table-header[data-sticky="true"] .dx-table-head`) carry their own opaque background, so nothing scrolls into
view through them; `VirtualList` has no row hover; the gallery and photo wall are excluded from the hover rule.
