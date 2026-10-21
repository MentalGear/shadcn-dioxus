---
id: 340
title: "DataTable: a column definition and one state hook, rendered into Table"
status: "BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation)."
size: "M (about 1.5 to 2 days)"
area: components
updated: 2026-10-21
---

# Row 340: DataTable: a column definition and one state hook, rendered into Table

## Item

Give DataTable a small data model. A column definition plus a `use_data_table` hook for sort, filter, page and selection, so a page describes its table once instead of writing every header and cell by hand.

## Status / trigger

BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation).

## Size

M (about 1.5 to 2 days)

## Detail

Evaluation: https://claude.ai/artifact/CGKtSVebCVar2PmKXNWs5T (Table, DataTable and VirtualGrid). Evidence from `main` `fd083a2d`. Order: 340, then 341, then 342; 341 and 342 only pay off if tens-of-thousands-row tables are a real use case (row 154 lists DataTable rows as a planned engine consumer).

- **Today:**
  - `data_table/state.rs` has `SortDirection`, `sort_rows`, `filter_rows`, `includes_string` and `paginate`. They are plain generic functions over `&[T]`.
  - There is no column or row model. The demo (`data_table/variants/main/mod.rs`) picks the sort column with a `&'static str` and writes every `TableHead` and `TableCell` by hand.
  - The "all, some, none" header checkbox and the row selection logic are written twice, in `table/variants/rich` and in the DataTable demo.
- **Target:**
  - `Column<T>`: an `id`, a header, a cell renderer `Fn(&T) -> Element`, an optional sort key, an optional width and alignment.
  - `use_data_table(rows, columns)`: owns sort, filter, page and selection. It returns the derived rows plus handlers, and it is built on the existing pure helpers, which stay and keep their unit tests.
  - Render into the existing `Table`, `DataTableColumnHeader`, `DataTableToolbar` and `DataTablePagination`. Rewrite the demo on it and share the selection tri-state between the two demos.
  - Keep it in the copied component directory, the same way `state.rs` is now, so `shadcn-dioxus add data_table` still works without extra wiring.
- **Done when:** `data_table.spec.ts` and the tier-1 APG sortable-table oracle pass unchanged against the rewritten demo, and the hook has unit tests for sort, filter, page and selection transitions.
