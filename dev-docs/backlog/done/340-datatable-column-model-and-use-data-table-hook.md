---
id: 340
title: "DataTable: a column definition and one state hook, rendered into Table"
status: "BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation; reframed 09:35 as one…"
size: "M+ (about 2 to 2.5 days; was M before…"
area: components
updated: 2026-10-22
---

# Row 340: DataTable: a column definition and one state hook, rendered into Table

## Item

Give DataTable a small data model. A column definition plus a `use_data_table` hook for sort, filter, page and selection, so a page describes its table once instead of writing every header and cell by hand.

## Status / trigger

BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation; reframed 09:35 as one DataTable<T> component). First in the order 340, 343, 341, 342.

## Size

M+ (about 2 to 2.5 days; was M before the 09:35 amendment)

## Detail

Evaluation: https://claude.ai/artifact/CGKtSVebCVar2PmKXNWs5T (Table, DataTable and VirtualGrid), from `main` `fd083a2d`. Owner order
2026-10-21 09:35: 340, then 343, then 341, then 342. Before this row, `data_table/state.rs` had only pure generic helpers
(`SortDirection`, `sort_rows`, `filter_rows`, `includes_string`, `paginate`), the demo picked the sort column with a `&'static str`
and wrote every `TableHead` and `TableCell` by hand, and the all/some/none header checkbox was written twice.

**The design (owner amendment 2026-10-21 09:35, "solved by construction"), and what each part rules out:**

1. *One mode type, in the engine:* `VirtualizeMode { Auto, ContentVisibility, Windowed }` and `resolve` move to
   `primitives/src/virtual/`, so VirtualList, the Table data body and VirtualGrid share `virtualize` and `auto_threshold` (rules out a
   component with its own threshold logic or without Auto). Built by row 343.
2. *Hand-written rows and data rows are different types:* `TableBody { children }` stays (content-visibility class only); a data body
   `TableRows { count, render_row, virtualize, auto_threshold }` renders inside a real `<table>` with spacer `<tr>`s in Windowed mode
   (rules out asking for windowing on rows that cannot be windowed). Row 343.
3. *`DataTable<T>` is one data-driven component:* `rows`, `columns: Vec<Column<T>>`, opt-in sort, filter, selection, pagination;
   `use_data_table` stays public underneath for custom layouts; `Column<T>` carries a typed sort key (`Fn(&T) -> K`, `K: Ord`), not a
   string (rules out header and cell counts drifting, a sort key naming a missing column, `aria-sort` or the selected state on the
   wrong element, since the component emits them).
4. *The layout is explicit, the mode automatic:* `layout: Table` (default) or `layout: Grid` (VirtualGrid, row 342); Auto picks only
   between content visibility and windowing within the chosen layout and never turns a read-only table into a keyboard grid, because
   that would change the accessibility model when the row count crosses a threshold.

**This row's deliverable:** the `DataTable<T>` component with `Column<T>` and `use_data_table`, `layout: Table` only (size M to M+).

**Landed 2026-10-22 (lane `datatable`).** `data_table/model.rs` (pure `TableState` plus `derive`, 12 unit tests), `columns.rs`
(`Column<T>` with typed `.sortable(key)`, `.filterable`, `.width`, `.align`; `RowFn`; `use_data_table` returning `DataTableModel {
state (Copy mutators), view, ... }`), and `DataTable<T>` in `component.rs` (toolbar, `Table`, sort headers with `aria-sort`, selection,
pagination; `selectable`, `filter_placeholder`, `page_sizes`, `default_sort`). `state.rs` keeps its helpers and gains `sort_rows_by` and
`tri_state`; the demo is one `DataTable::<Payment>` and the Table `rich` demo's header checkbox uses the shared `tri_state`. Evidence:
`cargo test -p preview data_table` 34 passed; `data_table.spec.ts` (5 new: sort and `aria-sort`, page reset, selection across
pages/sort/filter, keyboard sort plus select-all plus Tab order, keyboard filter and paging) 17 x3 green; the tier-1 APG sortable-table
oracle unchanged and green; `test:changed` 44 passed. `Column<T>` is layout-agnostic and `use_data_table` yields `view` plus `state`,
so a `TableRows` body (343) or a VirtualGrid renderer (342) consumes it; the collection model later moved to rows 360-363.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/340-datatable-column-model-and-use-data-table-hook.md`.*
