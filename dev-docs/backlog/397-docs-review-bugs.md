---
id: 397
title: "Bugs the docs review found: ToggleGroup arrows, dead ScrollArea prop, DataTable sort, Sidebar Ctrl+B"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "S"
area: components
updated: 2026-10-24
---

# Row 397: Bugs the docs review found: ToggleGroup arrows, dead ScrollArea prop, DataTable sort, Sidebar Ctrl+B

## Item

- ToggleGroup `roving_loop: false` stops arrow navigation entirely, not only the wrap (`primitives/src/toggle_group.rs` focus_next/focus_prev return early).
- ScrollArea `always_show_scrollbars` toggles a class no stylesheet defines.
- DataTable: a sort can never be cleared (`data_view/state.rs` toggle_sort), while the reorder handle says 'Clear the sort to reorder rows'.
- Sidebar: the global Ctrl/Cmd+B handler cannot be turned off and swallows the shortcut in editors on the page.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

S

## Detail

Each item gets a spec that fails first. The fixes are independent and can run as parallel lanes.
