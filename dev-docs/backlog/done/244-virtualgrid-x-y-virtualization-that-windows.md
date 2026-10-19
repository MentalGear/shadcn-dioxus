---
id: 244
title: "VirtualGrid: x/y virtualization that windows rows AND columns (a data grid, an image grid)"
status: "LANDED 2026-10-19 (Round 4: VirtualGrid + VirtualWrapGrid on the shared engine; image gallery and…"
size: "L"
area: virtual
updated: 2026-10-19
---

# Row 244: VirtualGrid: x/y virtualization that windows rows AND columns (a data grid, an image grid)

## Item

`VirtualGrid`: x/y virtualization that windows rows AND columns (a data grid, an image grid)

## Status / trigger

**LANDED 2026-10-19 (Round 4: VirtualGrid + VirtualWrapGrid on the shared engine; image gallery and 10k x 50 data grid demos; virtual-grid.spec)**

## Size

L

## Detail

Not on the backlog before: the 2026-10-04 survey (`research/virtualization-survey-2026-10-04.md` Q2) found no grid or horizontal list virtualization; `VirtualList` is vertical, one column, and the carousel windows one axis only. Build on the shared engine of row 154 (`primitives/src/virtual/`): a second axis in the size index and window (fixed or estimated column widths, measured on render), a fixed-column-count "grid" layout mode (cells wrap into rows, one axis windowed) and a true 2-D mode (rows x columns windowed, sticky header row/column optional), keyboard grid navigation (`role=grid`, arrow keys, Home/End, PageUp/Down) with the focused cell kept rendered, RTL; the survey's measured first answer for modest sizes is `content-visibility: auto` on cells (it skips on both axes, 100x100 probe), with windowing for very large grids. Demos: an image gallery grid, a 10k-row x 50-column data grid.

## Result (2026-10-19, lane `virtual-grid`)

**Construction: a grid is two one-axis engines, not a new engine.** `primitives/src/virtual/grid.rs` adds `GridEngine` = one `Engine` (the `VirtualList` windowed engine: Fenwick `SizeIndex`, hysteresis re-centring once per `buffer / 2` items, the canvas frozen while a scroll runs) for the rows and one for the scrolling columns. The only grid-specific arithmetic is the sticky edges (the rows axis sees `height - header`, the columns axis `width - sticky_width`, and the sticky first column is not part of the columns axis), the **pinned focused cell** (`with_pinned`: the active cell is drawn on each axis even far outside the window, so a keyboard jump or a scroll away never unmounts the focused cell or the grid's only tab stop) and the grid pattern's key map (`GridShape::navigate`: arrows, Home/End, Ctrl+Home/End, PageUp/PageDown, no wrap at edges, a short last row clamps the column). `Engine` gained only `item`, `visible_len` and `forget_measurements`. 13 unit tests cover the 2-D windowing math (window = product of both axes, far scroll, sticky column offsets, header shrinking the page, pinned cell far away and not duplicated, per-axis hysteresis) and the navigation.

**Components** (`primitives/src/virtual_grid.rs`, styled in `preview/src/components/virtual_grid/`): `VirtualGrid` (true 2-D: `row_count x column_count`, `row_height` / `column_width` callbacks, optional sticky `render_header` row and `sticky_first_column`, `row_buffer` / `column_buffer`) and `VirtualWrapGrid` (fixed `columns`, items wrap into CSS-grid rows, rows measured with `estimate_row_height` as the first guess; a column-count change forgets the measurements). Both: `role="grid"` with `aria-rowcount`/`aria-colcount` of the whole grid, `aria-rowindex`/`aria-colindex` on every mounted row/cell, one roving `tabindex="0"`, one passive scroll listener + one ResizeObserver through `js_listener.rs` `listen`/`cleanup`, horizontal keys through `Direction::resolve_horizontal` (RTL: the layout is logical -- `inset-inline-start`, sticky at the inline start -- and the bridge reports `|scrollLeft|`). Rows and cells are absolutely positioned (the pinned cell sits at its true position), so `VirtualGrid` rows are not measured: a cell taller than its row overflows (documented).

**Demos:** `main` = a 10,000-row x 50-column data grid (sticky header + row-number column; about 300 of 500,000 cells mounted); `gallery` = 5,000 generated inline-SVG landscapes (no image files, nothing remote) with a 2/3/4/6 column switch.

**Evidence:** `playwright/virtual-grid.spec.ts` 9 tests (DOM stays < 1,000 cells, far corner renders on scroll, keyboard nav keeps the focused cell rendered and in view clear of the sticky header/column, focus survives scrolling away, RTL sticky side + mirrored arrows + negative `scrollLeft` windowing, gallery windowing/keyboard/re-flow, axe) 27/27 over `--repeat-each=3` on a lane-ssg debug build; `virtual_list.spec.ts`, `scroll-fade`, `component-catalog`, `docs-structure`, `docs-props-table` 52 passed, 1 skipped.

**Found on the way (a class, fixed here by construction for this file):** an rsx `for x in xs { {make_cell(x)} }` produces UNKEYED list items (the key sits on the inner element, not the iteration's root), so every window move re-created every cell and dropped focus; cell lists are now `{xs.map(|x| make_cell(x))}` iterators of keyed roots. Other components with the same `for { {expr} }` shape over a changing list would have the same re-mount; not audited here.

**Not done / open:** column widths are given, not measured; the header row is not keyboard-navigable (cells only); no `content-visibility` fallback mode for small grids (the docs point at plain CSS grid + `content-visibility: auto` for a few thousand cells).
