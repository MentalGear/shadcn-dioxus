---
id: 342
title: "DataTable: a large-data variant rendered through VirtualGrid"
status: "LANDED 2026-10-23 (lane biggrid). Was: owner 2026-10-21; amended 09:35; scope widened 2026-10-22…"
size: "S (about 1 day)"
area: components
updated: 2026-10-22
---

# Row 342: DataTable: a large-data variant rendered through VirtualGrid

## Item

A DataTable demo that renders the row 340 model through VirtualGrid (for example 50,000 rows, sortable, filterable and selectable, scrolling in place of pages), and one table look shared by Table and VirtualGrid.

## Status / trigger

LANDED 2026-10-23 (lane biggrid). Was: owner 2026-10-21; amended 09:35; scope widened 2026-10-22; needed rows 340, 343 and 341.

**Amendment 2026-10-22, superseded the same day by the evaluation in [`research/collection-model-2026-10-22.md`](../../research/collection-model-2026-10-22.md): the collection model moved to rows 360-363, so this row is back to its original scope (VirtualGrid layout, large-data demo, shared table look) on top of them.** Original amendment, kept for the record: one collection model for every view. Before the VirtualGrid variant, move row 340's `TableState` + `derive` out of `data_table/` into one shared collection model that Table/`TableRows`, VirtualList, VirtualGrid and the drag-and-drop list all render from:
- **Manual order:** the state gains an ordered list of row keys; with no active sort the view follows it, and a drag or keyboard "move up/down" rewrites it (reorderable lists and table rows from one model; the drag-and-drop list becomes a renderer of it).
- **Fields, not columns, at the model level:** a field says how a row sorts and filters; a list is the model with one or a few fields. Table `Column<T>` = field + header + cell, plus a table-only column order and visibility state (reorderable columns).
- **Renderers stay separate** so the semantics stay right (`list`/`listbox` with free-form items vs `table`/`grid` with cells, as shadcn keeps them); a list is not rendered as a one-column table.
- `TableState` must keep its public transitions (or a deprecated alias) so row 340's `DataTable` and `use_data_table` callers keep compiling.

## Size

S (about 1 day)

## Detail

Evaluation: https://claude.ai/artifact/CGKtSVebCVar2PmKXNWs5T (Table, DataTable and VirtualGrid). Evidence from `main` `fd083a2d`. Order: 340, then 341, then 342; 341 and 342 only pay off if tens-of-thousands-row tables are a real use case (row 154 lists DataTable rows as a planned engine consumer).

- **Target:**
  - A `virtualized` (or `large`) variant on the DataTable page. Same columns, same `use_data_table` hook, with `VirtualGrid` in place of `Table` and no pagination.
  - The docs say when to use which. The plain Table is the default. VirtualGrid is for tens of thousands of rows, and the docs list its costs (no find-in-page, server render shows only the active cell, screen readers see only the window), as `virtual_grid.rs`'s module docs already do.
  - Shared look: `virtual_grid/style.css` re-creates the table look (text size, row border, header weight) by hand. Move those values into shared tokens, or one shared rule set, that both `table/style.css` and `virtual_grid/style.css` use, so the two cannot drift.
- **Done when:** a spec sorts, filters and selects in the large variant (including a selected row scrolled out and back), and visual or computed-style checks show Table and VirtualGrid rows match.

## Amendment (owner 2026-10-21, 09:35)

New order (owner 2026-10-21, 09:35): 340, then 343, then 341, then 342. Design: row 340's amendment (one `VirtualizeMode` for every consumer, one data-driven `DataTable<T>`, explicit layout with an automatic mode).

**This row:** Add `layout: Grid` to DataTable (rendered through VirtualGrid with row 341's hooks), a large-data demo for each layout, and the shared table look. Comes after 340, 343 and 341.

## Landed (lane biggrid, 2026-10-23)

- **API:** `DataTable { layout: DataTableLayout::Grid, grid_height, grid_label, .. }` (default `DataTableLayout::Table`; `Column::grid_width(px)`; the hook `use_data_table_index` returns the derived `View` in a memo, the sibling of `use_data_table`). `page_sizes` and `reorderable` are ignored in Grid. The existing `DataTable` API is untouched.
- **Why a `layout` prop and not a `virtualized` flag or a second component:** the design note (migration step 6: "`layout: Grid` renders the same `View`") and row 340's amendment 4 say the layout is explicit and never switched by the row count, because the grid changes the keyboard and accessibility model (one tab stop and arrow keys against Tab through every control; the window against the whole table). One `DataTable<T>` keeps one `Column<T>` and one state; a separate component would copy the toolbar, the Columns menu and the selection summary. The two layouts are two private components behind it (`DataTableTable`, `grid.rs`), because hooks cannot be conditional.
- **No row copies:** the Grid layout reads rows by position from `View::indices` (`use_data_table_index`); `use_data_table` stays the cloning one for the page's rows.
- **Hooks used (row 341):** `column_sort` (aria-sort on the header), `row_key` (the view's keys) and `row_selected` keyed by it (`DataTableState::is_selected`), a checkbox column and sort buttons as controls in cells. Selection survives scrolling a row out of the window and a sort.
- **One table look:** the theme's `--dx-table-*` tokens (`preview/assets/dx-components-theme.css`: font size, cell padding, row border, head height and weight, hover and selected fill); `table/style.css` and `virtual_grid/style.css` both read them, and a data grid's rows now take the Nova row hover. `data_table-large.spec.ts` compares the computed values of a Table row and a Grid row (including the 37px row height and 40px header).
- **Found on the way (fixed in the DataTable, class left open):** `merge_attributes` lets a `class: None` replace the base class, so every unaligned `TableCell`/`TableHead` of `DataTable` had no `dx-table-cell` (1px UA padding, no border). The DataTable now passes `""`; the class (a `None`-valued class attribute wipes the base class of any themed component that forwards an `Option<&str>`) is in `primitives/src/lib.rs`, not touched here.
- **Spec:** `playwright/data_table-large.spec.ts` (6 tests: sort, filter, select, a selected row scrolled out and back and through a sort, select-all under a filter, the shared look).
- **Costs left:** a selection toggle re-runs `derive` over all 50,000 rows (the memo depends on the whole `ViewState`); fine at this size, a candidate for splitting the memo into (sort, filter, order) and selection counts.
