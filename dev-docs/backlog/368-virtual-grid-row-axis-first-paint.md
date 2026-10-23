---
id: 368
title: "VirtualGrid: the row axis plans max(rows, viewport), so a short grid differs between server and first client frame"
status: "BACKLOG (found 2026-10-23 fixing the column axis in batch 9)."
size: "S"
area: primitives
updated: 2026-10-23
---

# Row 368: VirtualGrid row-axis canvas vs first paint

## Item

The column axis used to plan its canvas as `max(total columns, viewport)`, so a grid whose columns were narrower than its box rendered 428 px on the server (viewport 0) and 666 px after hydration (`first-paint-styles` red on the `selectable` demo). Batch 9 fixed the column axis by construction: `Plan::content_size` is the columns' own extent and the fill to the box is CSS (`min-width: 100%`). The row axis still plans `body_height` as `max(total rows, viewport)`, so a grid with fewer rows than fit would differ the same way.

## Status / trigger

BACKLOG (found 2026-10-23 fixing the column axis in batch 9).

## Size

S

## Detail

`min-height: 100%` does not resolve on the body wrapper (no definite height), so the column fix does not carry over as is. Options: give the body wrapper a definite height from the grid's own box, or plan the row canvas from `content_size` and let the empty space be the scroll container's. No current demo has fewer rows than fit; add one with the fix and a `first-paint-styles` case for it.
