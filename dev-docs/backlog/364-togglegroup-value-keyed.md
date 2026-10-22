---
id: 364
title: "ToggleGroup: pressed state keyed by value, not index (optional)"
status: "BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note…"
size: "S."
area: components
updated: 2026-10-22
---

# Row 364: ToggleGroup: pressed state keyed by value, not index (optional)

## Item

Optional, shadcn parity: ToggleGroup stores pressed items as `HashSet<usize>` (position); key them by value and deprecate the index set.

## Status / trigger

BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

S.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.
