---
id: 340
title: "DataTable: a column definition and one state hook, rendered into Table"
status: "BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation; reframed 09:35 as one…"
size: "M+ (about 2 to 2.5 days; was M before…"
area: components
updated: 2026-10-21
---

# Row 340: DataTable: a column definition and one state hook, rendered into Table

## Item

Give DataTable a small data model. A column definition plus a `use_data_table` hook for sort, filter, page and selection, so a page describes its table once instead of writing every header and cell by hand.

## Status / trigger

BACKLOG (owner 2026-10-21, after the Table/DataTable/VirtualGrid evaluation; reframed 09:35 as one DataTable<T> component). First in the order 340, 343, 341, 342.

## Size

M+ (about 2 to 2.5 days; was M before the 09:35 amendment)

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

## Amendment (owner 2026-10-21, 09:35)

Owner: "put it in the backlog if it's truly the most peak solved by construction". New order (owner 2026-10-21, 09:35): 340, then 343, then 341, then 342.

Tom asked for Table to get the same `virtualize` auto mode as the virtual components, and for the by-construction version of the whole design ("put it in the backlog if it's truly the most peak solved by construction"). This amends rows 340-342 as filed, and adds row D (provisional 343).

**The design, and what each part rules out by construction:**
1. **One mode type, in the engine.** `VirtualizeMode { Auto, ContentVisibility, Windowed }` and its `resolve(count, auto_threshold)` move from `virtual_list.rs` (`VirtualListMode`, `:83-115`) into `primitives/src/virtual/`. VirtualList, the Table data body and VirtualGrid all take the same `virtualize` and `auto_threshold` props. *Rules out:* a component that has its own threshold logic or its own default, or that lacks Auto.
2. **Hand-written rows and data rows are different types.** `TableBody { children }` stays as it is and may only use content visibility (a CSS chunking class, since every row is already in the DOM). A new data body, `TableRows { count, render_row, virtualize, auto_threshold }`, renders inside a real `<table>`. It supports all three modes, and in Windowed it puts spacer `<tr>`s around the window on the one-axis `Engine` and `Bridge`. *Rules out:* asking for windowing on rows that cannot be windowed.
3. **DataTable<T> is one data-driven component.** It takes `rows`, `columns: Vec<Column<T>>`, opt-in sort, filter, selection and pagination, and `virtualize`. `use_data_table` stays public underneath for custom layouts. `Column<T>` carries a typed sort key (`Fn(&T) -> K`, with `K: Ord`), not a string. *Rules out:* header and cell counts drifting apart, a sort key naming a column that doesn't exist, and `aria-sort` or the selected state on the wrong element (the component emits them, not the page).
4. **The layout is explicit, and the mode is automatic.** DataTable has `layout: Table` (the default) or `layout: Grid` (VirtualGrid, for data that is also very wide or needs cell-level keyboard navigation). Auto picks only between content visibility and windowing *within* the chosen layout. It never switches a read-only table into a keyboard grid on its own, because that would change the accessibility model and the keyboard behaviour when the row count crosses a threshold. *Rules out:* behaviour that changes with data size beyond performance.

**This row:** The deliverable is the `DataTable<T>` component (point 3 above), with `Column<T>` (typed sort key `Fn(&T) -> K`, `K: Ord`) and `use_data_table` underneath (still public for custom layouts), and `layout: Table` only. Size M becomes M+ (about 2 to 2.5 days).
