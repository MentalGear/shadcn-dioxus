---
id: 252
title: "VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)"
status: "CLOSED 2026-10-22 as inherent (owner chose \"Close as inherent\" on the decision card). The demo…"
size: "S"
area: components
updated: 2026-10-22
---

# Row 252: VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)

## Item

VirtualGrid: the prerendered inner wrapper is one row shorter than after hydration (175000 px -> 175036 px)

## Status / trigger

**CLOSED 2026-10-22 as inherent (owner chose "Close as inherent" on the decision card).** The demo keeps measured row heights; the first-paint gap is documented, not fixed.

**OPEN, inherent (cannot be decided at render): measured canvas**

## Size

S

## Detail

first-paint-styles fails for `/component/virtual_grid/` (JS-off vs hydrated, and `/docs -> /component/virtual_grid/`) on main 1353a5a0 and on the round-1 tree alike: the grid's inner wrapper is `height: 175000px` in the served HTML and `175036px` once hydrated (36 px = the demo's row height), so the SSR and the client compute the total size differently (off by one row). Causes in `primitives/src/virtual_grid.rs` / `primitives/src/virtual/grid.rs` (row 244, round 4). Make SSR and the first client render use the same row-count arithmetic; first-paint-styles is the proof.

## Resolution (lane ssr-parity, 2026-10-21)

Re-checked: the 175000 -> 175036 px canvas is VirtualWrapGrid's gallery demo (140 px estimates on the server, the sum of rows the client measures through the `document::eval` bridge, `bridge.rs`), not VirtualGrid's data grid. Commit 55556413 already marks such canvases `data-measured="false|true"` so first-paint-styles treats them as state. A render-time fix would need the row heights before the DOM exists; the row-329 harness cannot see it either (no eval reply). Left open for owner decision: close as inherent or give the demo exact row heights.
