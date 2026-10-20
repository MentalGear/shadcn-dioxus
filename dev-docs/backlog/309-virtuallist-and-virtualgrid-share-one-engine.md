---
id: 309
title: "VirtualList and VirtualGrid share one engine: DOM bridge, pinned focus, fade (two components, one engine)"
status: "IN PROGRESS 2026-10-20 (owner go; lane virtual-engine)"
size: "M"
area: virtual
updated: 2026-10-20
---

# Row 309: VirtualList and VirtualGrid share one engine: DOM bridge, pinned focus, fade (two components, one engine)

## Item

Owner decision 2026-10-20: keep `VirtualList` and `VirtualGrid` as two components (list vs grid ARIA semantics: a grid has 2-D arrow keys and one tab stop; a list reads top to bottom with variable row heights), but share as many parts of the engine between them as possible and sensible, taking the best of each. This is the second step of [row 154](./154-shared-virtualization-engine-one-engine-in.md)'s three-layer outline (core / strategy / DOM bridge) after [row 244](./done/244-virtualgrid-x-y-virtualization-that-windows.md) made a grid two copies of the one-axis `Engine`. Related: [row 153](./153-native-find-in-page-for-windowed-virtual-lists.md) (find-in-page placeholders), [row 186](./186-shadcn-utilities-scroll-fade-and-shimmer-are.md) (the `fade` prop), [row 308](./308-virtualgrid-scroll-trap-at-the-edges.md) (VirtualGrid scroll trap at the edges, `overscroll-behavior`).

## Status / trigger

**IN PROGRESS 2026-10-20 (owner go; lane virtual-engine)**

## Size

M

## Detail

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
| Content-visibility strategy | `cv_chunks` mode + `Auto` threshold | none | stays list-only for now: a grid CV mode needs its own keyboard/pinning story over flow rows (open item below) |
| Find-in-page placeholders | opt-in `text_value` (row 153) | none | stays list-only (a grid's cell text would need 2-D placeholders); the reveal half is shared (`Bridge::reveal`) |
| Overscroll / scroll chaining | none (chains to the page) | `overscroll-behavior: contain` in its sheet (row 308 changes it) | no engine policy: it is CSS on each component's container, reconciled with [row 308](./308-virtualgrid-scroll-trap-at-the-edges.md) |
| RTL | n/a (vertical) | `use_direction` key map, `\|scrollLeft\|` | the report's RTL-neutral `left` is shared; the key map stays grid-only |
| Keyboard model, ARIA, sticky edges | list semantics | grid pattern, sticky header/column | per component (the reason there are two) |

**Ideas borrowed (read-only design reference).** A list is a one-column grid (the wrap grid already windows rows with the list's `Engine`); saturating `u32` arithmetic for offsets (`corrected`); the core computes a scroll compensation and the presentation layer decides when to apply it (the engine returns a delta, the bridge writes it).

**Plan, not built here: a VirtualGrid `layout` mode**, integrated after this row lands. `layout: Grid` is today's shared rows x columns: the grid's structure (cells aligned in rows and columns) never depends on content; non-uniform content is a presentation concern inside the cell (`object-fit`), so both axes stay independent 1-D windows. `layout: Justified` gives each row its own tile positions, from the pure layout layer a separate lane adds under `primitives/src/virtual/layout/` (justified rows and 2-D tile windowing), with spatial keyboard navigation (Up/Down pick the tile in the next row nearest the current tile's centre). The grid internals stay open to it: rows are already the windowing unit of one `Engine`, the 2-D window is the row window times a cross-axis query, so Justified swaps the shared columns `Engine` for a per-row cross-axis index; the bridge, `with_pinned` and `GridShape`'s row/column addressing carry over.

## Notes

Evidence and the lane report are recorded when the lane lands.
