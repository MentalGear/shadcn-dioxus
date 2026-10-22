---
id: 360
title: "Shared collection model: data_view in primitives (identity, ViewState with manual order, fields, index-returning derive) and the selection count fix"
status: "BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note…"
size: "M."
area: primitives
updated: 2026-10-22
---

# Row 360: Shared collection model: data_view in primitives (identity, ViewState with manual order, fields, index-returning derive) and the selection count fix

## Item

Migration steps 1 and 2 of the note: lift row 340's `TableState`/`derive` into `primitives/src/data_view/` with no behaviour change (data_table re-exports, current names keep compiling), one `contains_query` for the two substring filters, memoised `derive` returning indices; then fix the footer count (`selected_in_view of filtered_count`, shadcn parity; today it can read "3 of 1"). Law tests and `check-collection-derive.sh`.

## Status / trigger

BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.
