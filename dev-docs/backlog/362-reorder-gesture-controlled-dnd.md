---
id: 362
title: "Reorder gesture extracted, KeyOrder, DragAndDropList controlled mode with on_reorder"
status: "BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note…"
size: "M."
area: primitives
updated: 2026-10-22
---

# Row 362: Reorder gesture extracted, KeyOrder, DragAndDropList controlled mode with on_reorder

## Item

Migration step 4: one keyboard-accessible reorder gesture out of `drag_and_drop_list.rs`, a `KeyOrder` type (ordered keys with move operations) shared by row and column order, a controlled DragAndDropList (`order` + `on_reorder`; today it reads `items` once and never reports the new order), drop the deprecated `aria-grabbed`, one key derivation. Parallel with row 361.

## Status / trigger

BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.
