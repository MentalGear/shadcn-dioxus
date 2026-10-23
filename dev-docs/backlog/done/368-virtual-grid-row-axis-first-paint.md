---
id: 368
title: "VirtualGrid: the row axis plans max(rows, viewport), so a short grid differs between server and first client frame"
status: "LANDED 2026-10-23 (lane biggrid): the engine's canvas is the items' extent on both axes; short-grid demo and first-paint case."
size: "S"
area: primitives
updated: 2026-10-23
---

# Row 368: VirtualGrid row-axis canvas vs first paint

## Item

The column axis used to plan its canvas as `max(total columns, viewport)`, so a grid whose columns were narrower than its box rendered 428 px on the server (viewport 0) and 666 px after hydration (`first-paint-styles` red on the `selectable` demo). Batch 9 fixed the column axis by construction: `Plan::content_size` is the columns' own extent and the fill to the box is CSS (`min-width: 100%`). The row axis still plans `body_height` as `max(total rows, viewport)`, so a grid with fewer rows than fit would differ the same way.

## Status / trigger

LANDED 2026-10-23 (lane biggrid). Found fixing the column axis in batch 9.

## Size

S

## Detail

`min-height: 100%` does not resolve on the body wrapper (no definite height), so the column fix does not carry over as is. Options: give the body wrapper a definite height from the grid's own box, or plan the row canvas from `content_size` and let the empty space be the scroll container's. No current demo has fewer rows than fit; add one with the fix and a `first-paint-styles` case for it.

## Landed

- **Construction:** `Plan::canvas_height` in `primitives/src/virtual/virtualizer.rs` is `total_size()`, never `max(total, viewport)`; the separate `content_size` field of 4b55f1a1 is gone (it was the same number). Every consumer of the engine reads it: `VirtualGrid` (`body_height`, and `pane_width` for the columns axis), `VirtualWrapGrid`, `VirtualList` and the `TableRows` bottom spacer. The viewport is `0` on the server and measured on the client, so nothing that reaches the DOM may depend on it; what fills the box is CSS (`min-width: 100%` for columns) or the scroll container itself (the space under the last row). This is the option "plan the row canvas from the content size and let the empty space be the scroll container's"; the other option (a definite body height from the grid's box) needs a measured box, which the server does not have.
- **Subsumes:** the grid's row axis (this row), `VirtualList` and `VirtualWrapGrid` with fewer items than fit (same `max`), and a blank spacer row under a short windowed `Table`. Does not subsume: a grid inside an auto-height box (it has no viewport to fill, and never had one).
- **Red first:** a `short` demo (5 rows in a 24rem box) and `playwright/oracle/tier2-html/first-paint-styles.spec.ts` "5b" (the demo's body canvas, JavaScript off against hydrated) plus the existing sweeps 2 and 3 on `/component/virtual_grid/`. On a build of the demo alone: 5b `canvas height: server 180px, hydrated 342px`; sweep 2 and 3 `div.dx-virtual-grid#1/div#1 { height: 220px -> 382px }`, `div#2 { height: 180px -> 342px }`. After the fix all three are green. Cargo unit: `a_short_list_has_one_canvas_before_and_after_the_viewport_is_known`.
