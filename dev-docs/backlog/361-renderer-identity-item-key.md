---
id: 361
title: "Renderer identity: item_key on VirtualList, TableRows and VirtualGrid, engine sizes keyed by item"
status: "BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note…"
size: "M."
area: primitives
updated: 2026-10-22
---

# Row 361: Renderer identity: item_key on VirtualList, TableRows and VirtualGrid, engine sizes keyed by item

## Item

Migration step 3: every windowed renderer tracks items by position today (DOM keys, pinned focus row, grid active cell, measured-height cache), so after a sort or move focus and heights stick to the slot. Optional `item_key` (default the index) on VirtualList, TableRows, VirtualGrid/VirtualWrapGrid; `check-render-keys.sh`; across-sort specs. Parallel with row 362.

## Status / trigger

BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.
