---
id: 244
title: "VirtualGrid: x/y virtualization that windows rows AND columns (a data grid, an image grid)"
status: "BACKLOG (owner question 2026-10-18)"
size: "L"
area: virtual
updated: 2026-10-18
---

# Row 244: VirtualGrid: x/y virtualization that windows rows AND columns (a data grid, an image grid)

## Item

`VirtualGrid`: x/y virtualization that windows rows AND columns (a data grid, an image grid)

## Status / trigger

**BACKLOG (owner question 2026-10-18)**

## Size

L

## Detail

Not on the backlog before: the 2026-10-04 survey (`research/virtualization-survey-2026-10-04.md` Q2) found no grid or horizontal list virtualization; `VirtualList` is vertical, one column, and the carousel windows one axis only. Build on the shared engine of row 154 (`primitives/src/virtual/`): a second axis in the size index and window (fixed or estimated column widths, measured on render), a fixed-column-count "grid" layout mode (cells wrap into rows, one axis windowed) and a true 2-D mode (rows x columns windowed, sticky header row/column optional), keyboard grid navigation (`role=grid`, arrow keys, Home/End, PageUp/Down) with the focused cell kept rendered, RTL; the survey's measured first answer for modest sizes is `content-visibility: auto` on cells (it skips on both axes, 100x100 probe), with windowing for very large grids. Demos: an image gallery grid, a 10k-row x 50-column data grid.
