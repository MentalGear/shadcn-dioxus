---
id: 369
title: "VirtualGrid and TableRows: carry the exact measured size so a short body with fractional rows cannot overflow by 1px"
status: "Open. Found in batch 10 (scroll-fade 199 vs 198)."
size: "S"
area: primitives
updated: 2026-10-23
---

# Row 369: fractional row heights in the grid and table renderers

## Item

Row 368 made every canvas the items' own extent instead of `max(items, viewport)`. That exposed a rounding gap: `measured_px` rounds each measured row to the nearest whole px, so three 66.1875 px rows were a 198 px canvas for 198.56 px of content and the `fade-windowed` VirtualList scrolled by 1 px (`scroll-fade.spec.ts:238`). Batch 10 fixed `VirtualList` by construction: `Bridge::measure_exact` hands the engine the exact size, offsets stay on the rounded whole px, and `Engine::resize_item_exact` keeps the remainders so the canvas is `total_size() + ceil(sum of remainders)`.

## Status / trigger

Open. Found in batch 10 (scroll-fade 199 vs 198).

## Size

S

## Detail

`virtual_grid.rs` and `table_rows.rs` still call `measure(measured_px(..))`, so a short grid or table body whose rows have fractional heights can overflow its box by a pixel the same way. No spec shows it yet. Move both onto `measure_exact`, then delete the plain `measure` path if nothing else needs it, so no renderer can drop the remainder again; add a short-body case with a fractional row height to `first-paint-styles` or the grid specs.
