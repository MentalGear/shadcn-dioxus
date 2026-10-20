---
id: 252
title: "VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)"
status: "BACKLOG (validation 2026-10-20)"
size: "S"
area: components
updated: 2026-10-20
---

# Row 252: VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)

## Item

VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)

## Status / trigger

**BACKLOG (validation 2026-10-20)**

## Size

S

## Detail

first-paint-styles fails for `/component/virtual_grid/` (JS-off vs hydrated, and `/docs -> /component/virtual_grid/`) on main 1353a5a0 and on the round-1 tree alike: the grid's inner wrapper is `height: 175000px` in the served HTML and `175036px` once hydrated (36 px = the demo's row height), so the SSR and the client compute the total size differently (off by one row). Causes in `primitives/src/virtual_grid.rs` / `primitives/src/virtual/grid.rs` (row 244, round 4). Make SSR and the first client render use the same row-count arithmetic; first-paint-styles is the proof.
