---
id: 309
title: "VirtualList and VirtualGrid share one engine: DOM bridge, pinned focus, fade (two components, one engine)"
status: "LANDED. All parts are in the tree: shared DOM bridge, pinned focus and fade (2026-10-20, lane…"
size: "M"
area: virtual
updated: 2026-10-24
---

# Row 309: VirtualList and VirtualGrid share one engine: DOM bridge, pinned focus, fade (two components, one engine)

## Item

Owner decision 2026-10-20: keep `VirtualList` and `VirtualGrid` as two components (list vs grid ARIA semantics: a grid has 2-D arrow keys and one tab stop; a list reads top to bottom with variable row heights), but share as many parts of the engine between them as possible and sensible, taking the best of each. This is the second step of [row 154](./154-shared-virtualization-engine-one-engine-in.md)'s three-layer outline (core / strategy / DOM bridge) after [row 244](./244-virtualgrid-x-y-virtualization-that-windows.md) made a grid two copies of the one-axis `Engine`. Related: [row 153](../153-native-find-in-page-for-windowed-virtual-lists.md) (find-in-page placeholders), [row 186](./186-shadcn-utilities-scroll-fade-and-shimmer-are.md) (the `fade` prop), [row 308](./308-virtualgrid-scroll-trap-at-the-edges.md) (VirtualGrid scroll trap at the edges, `overscroll-behavior`).

## Status / trigger

**LANDED.** All parts are in the tree: shared DOM bridge, pinned focus and `fade` (2026-10-20, lane virtual-engine), `VirtualJustifiedGrid` (lane vgrid-justified), and the two deferred grid parts, find-in-page placeholders (`primitives/src/virtual/find.rs`) and the content-visibility mode (`virtualize` / `auto_threshold` on `VirtualGrid` and `VirtualWrapGrid`), 7d08f034 and 8985512b, merged in batch 15 (`ddae4926`), spec `playwright/virtual-grid-modes.spec.ts`. The notes' remaining items are design decisions, not work: no engine-level overscroll policy (each container's sheet decides, row 308) and `VirtualJustifiedGrid` stays always windowed with no placeholders.

## Size

M

## Detail

The engine's design note (row 154, folded here 2026-10-23) is in [done/154](./154-shared-virtualization-engine-one-engine-in.md).

**Inventory (before this row).** Who implemented what, and the verdict.

| Part | VirtualList (windowed) | VirtualGrid / VirtualWrapGrid | Verdict |
|---|---|---|---|
| Scroll listener + resize bridge | own `SCROLL_BRIDGE_JS`: raw `addEventListener` scroll + window resize, removed after a recv sentinel (a `check-js-listeners.sh` `PAIRED_BY_HAND` entry) | own `SCROLL_BRIDGE_JS` (x2 consumers): `listen` scroll + one ResizeObserver, scope-owned | **shared**: `virtual/bridge.rs` `use_virtual_bridge` (passive `listen` scroll, window resize, one ResizeObserver, `cleanup`; one `ScrollReport` with `\|scrollLeft\|` for RTL) |
| Range-change signal write | hand-written `publish` (engine + `rev`) | the same `publish`, written twice | **shared**: `Bridge::publish` / `Bridge::track` over a `Windowing` trait (`Engine`, `GridEngine`) |
| Scroll correction (anchoring) write-back | `sync_container_scroll` (async eval) + `(offset as i32 + delta).max(0)` | `set_scroll_top` + the same cast, written twice | **shared**: `Bridge::scroll_axis`, `Bridge::measure`, saturating `corrected()` |
| Measuring | content-box `onresize` -> `resize_item` | border-box `onresize` (wrap grid) | **shared** flow `Bridge::measure` + `measured_px`; the box (content vs border) stays per component (the wrap grid's gaps are part of its row) |
| `scroll_to(index)` | `beforematch` -> `scroll_target` -> write | none (focus move scrolls by DOM rects past the sticky edges) | **shared** `Bridge::reveal`; the grid's sticky-aware focus scroll stays per component |
| Keep the focused item rendered | none: a focused row that left the window was unmounted (focus fell to `<body>`) | `with_pinned` (active cell) | **shared** `virtual/pin.rs`: `with_pinned` for absolute layouts, `pin_in_flow` for the list's flow layout (detached row drawn at its own position, same parent and key, so it rejoins the flow without remounting); **VirtualList gained it** |
| ScrollFade `fade` prop | yes | no | **VirtualGrid and VirtualWrapGrid gained it** (+ `dx-effects.css` in `globalAssets`, a `fade` demo) |
| Content-visibility strategy | `cv_chunks` mode + `Auto` threshold | none | **shared** (2026-10-24): `VirtualGrid` and `VirtualWrapGrid` take `virtualize` / `auto_threshold` (counted in cells) and render every cell in flow rows through `cv_chunks`; the keyboard model is the same, nothing needs pinning |
| Find-in-page placeholders | opt-in `text_value` (row 153) | none | **shared** (2026-10-24): `virtual/find.rs` (`FindLayer`, `use_find_in_page`), one layer for the list's rows, the data grid's cells (`"row:column"`, placed exactly) and the wrapping grid's items |
| Overscroll / scroll chaining | none (chains to the page) | `overscroll-behavior: contain` in its sheet (row 308 changes it) | no engine policy: it is CSS on each component's container, reconciled with [row 308](./308-virtualgrid-scroll-trap-at-the-edges.md) |
| RTL | n/a (vertical) | `use_direction` key map, `\|scrollLeft\|` | the report's RTL-neutral `left` is shared; the key map stays grid-only |
| Keyboard model, ARIA, sticky edges | list semantics | grid pattern, sticky header/column | per component (the reason there are two) |

**Ideas borrowed (read-only design reference).** A list is a one-column grid (the wrap grid already windows rows with the list's `Engine`); saturating `u32` arithmetic for offsets (`corrected`); the core computes a scroll compensation and the presentation layer decides when to apply it (the engine returns a delta, the bridge writes it).

**Landed 2026-10-20 (lane vgrid-justified): the Justified mode, as a sibling component `VirtualJustifiedGrid`** rather than a `layout` prop, so each mode's props stay honest (a justified wall has no `column_count`, column widths, sticky header/column or per-axis buffers; it has `aspects`, `target_row_height`, `gap`, `min_tile_width`, `canvas_width: Option<f64>` and a px `overscan`). Same page, same `style.css`, same bridge (`use_virtual_bridge`, with a third `Windowing` engine, `virtual/justified_engine.rs`), same pinning (`pin.rs`'s new `pin_sorted`, applied to the active tile's row and then the tile). Fluid width from the bridge's measured viewport (vertical scroll, re-layout on resize keeping the top row's first tile in place), or a fixed `canvas_width` for a two-axis wall windowed by row (`y`) and by tile within each row (`x`). Keyboard: reading-order Left/Right across rows, spatial Up/Down/PageUp/PageDown onto the nearest tile by horizontal centre, Home/End, Ctrl+Home/End; `aria-colcount` omitted (rows differ in length). A two-way `active: Option<Signal<usize>>` lets a caller jump to a tile (reveal + focus). Demos: `justified` (20,000 photos, row-height slider, "Wall (both axes)" switch) and a Grid / Justified toggle on `photo_wall`. Spec `playwright/virtual-grid-justified.spec.ts`. The original plan, for the record:

**Plan: a VirtualGrid `layout` mode**, integrated after this row lands. `layout: Grid` is today's shared rows x columns: the grid's structure (cells aligned in rows and columns) never depends on content; non-uniform content is a presentation concern inside the cell (`object-fit`), so both axes stay independent 1-D windows. `layout: Justified` gives each row its own tile positions, from the pure layout layer a separate lane adds under `primitives/src/virtual/layout/` (justified rows and 2-D tile windowing), with spatial keyboard navigation (Up/Down pick the tile in the next row nearest the current tile's centre). The grid internals stay open to it: rows are already the windowing unit of one `Engine`, the 2-D window is the row window times a cross-axis query, so Justified swaps the shared columns `Engine` for a per-row cross-axis index; the bridge, `with_pinned` and `GridShape`'s row/column addressing carry over.

## Notes

**Built (lane virtual-engine, 2026-10-20).** `primitives/src/virtual/bridge.rs` (`use_virtual_bridge`, `Bridge`, the `Windowing` trait, `corrected`, `measured_px`), `primitives/src/virtual/pin.rs` (`with_pinned` moved from `grid.rs`, new `pin_in_flow`), `ScrollReport` in `types.rs` (replaces `GridScroll`). `virtual_list.rs` and `virtual_grid.rs` lost their own bridge JS, `publish`, correction and measure code. New props: `VirtualGrid::fade`, `VirtualWrapGrid::fade`; new DOM hook `data-virtual-pinned` on a windowed list's detached focused row; new `fade` demo on the VirtualGrid page.

**Evidence.** `cargo test -p dioxus-primitives --lib virtual`: 90 passed (new: 5 bridge, 4 pin tests). `cargo clippy -p dioxus-primitives --features web --all-targets -- -D warnings`: clean. Fast gates (`run-gates.sh --skip` the cargo ones): 52/52 PASS. Debug lane-ssg build containing the change: `playwright/virtual-engine.spec.ts` 3/3 (wrap-grid and data-grid fade, list focused row kept as the same node and rejoining the flow); `virtual-grid`, `virtual_list`, `virtual-list-find`, `scroll-fade`: 43 passed, 1 skipped (CDP timing, debug). `registry-install-smoke.sh --only virtual_grid,virtual_list`: OK, 2/2.

**Open.** (1) ~~`scripts/check-js-listeners.sh`'s `PAIRED_BY_HAND` entry for `primitives/src/virtual_list.rs` is now stale~~ deleted by lane vgrid-justified. (2) ~~A content-visibility mode for the grids and find-in-page placeholders for grids were not built~~ built 2026-10-24, see below. (3) No engine-level overscroll policy: each container's sheet decides ([row 308](./308-virtualgrid-scroll-trap-at-the-edges.md)); the list has none and chains to the page. (4) `VirtualJustifiedGrid` stays always windowed and has no placeholders (a tile's place depends on the layout of every row above it, so a placeholder layer would have to re-run the justified layout).

**Built 2026-10-24 (branch `claude/row-309-grid-find-cv`): the two deferred grid parts.**

- **One find layer.** `primitives/src/virtual/find.rs` holds what `virtual_list.rs` had privately: the `beforematch` bridge (`use_find_in_page`, which now sends the placeholder's `data-find-index` as a string), the chunked `hidden="until-found"` layer (`FindLayer`, chunks of 100 rows, each a `content-visibility: auto` box, the `::search-text` rule, the re-hide in the next frame) and a `FindLayout`: `Rows` (the list, one element per row as before, so its node count and spec are unchanged), `Columns` (a data grid: one placeholder per non-empty cell at the column's exact inline offset, named `"row:column"`) and `Tracks` (a wrapping grid: the item index in rows of equal tracks). `text_value` on `VirtualGrid` is `Callback<(usize, usize), String>`, on `VirtualWrapGrid` `Callback<usize, String>`; one shared prop doc (`check-virtual-docs-parity.sh`).
- **What a match does.** The data grid's placeholders sit exactly where the cells do (row heights and column widths are known), so the browser's own scroll mounts the cell; the grid marks it `data-find-match` and runs the focus scroll without focusing (`reveal_cell`: clear of the sticky header and column, or first to the cell's computed place when the reveal did not scroll there, e.g. a dispatched event). The wrapping grid's rows are measured, so its placeholders sit at the estimates; a match centres the item's row through `Bridge::reveal`, as the list does.
- **Content-visibility mode.** `VirtualGrid` and `VirtualWrapGrid` split into a mode body each, like `VirtualList` (`VirtualGridWindowed` / `VirtualGridContentVisibility`, same for the wrap grid), chosen by `virtualize.resolve(cells, auto_threshold)` with cells = `row_count x column_count` or `count`. The content-visibility bodies render every row through `cv_chunks` (chunks of 20 skipped by the browser), columns in flow (the sticky header and first column are `position: sticky` in flow), no engine and no listener; the active cell, `row_key` resolution, `on_active_change` and the key map are one `GridFocus` / `WrapFocus` value both bodies use, and PageUp/PageDown count the viewport's rows from the container's measured height. Default `Auto`, so a grid of up to 1,000 cells is no longer windowed: the `short` demo (row 368's windowed first paint) and `DataTable`'s grid (a filter would re-mount it mid-typing) pin `Windowed`.
- **Demos and spec.** `virtual_grid` variants `find_in_page` (a 2,000 x 8 data grid and a 3,000-item gallery, both windowed with `text_value`) and `content_visibility` (60 x 12 and 240 tiles under `Auto`); `playwright/virtual-grid-modes.spec.ts`.
