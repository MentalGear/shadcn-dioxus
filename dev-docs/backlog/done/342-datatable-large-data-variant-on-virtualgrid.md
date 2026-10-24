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

Evaluation: https://claude.ai/artifact/CGKtSVebCVar2PmKXNWs5T (Table, DataTable and VirtualGrid), from `main` `fd083a2d`. Order:
340, 343, 341, then this row (the owner's 2026-10-21 09:35 amendment). VirtualGrid only pays off if tens-of-thousands-row tables are
a real use case (row 154 lists DataTable rows as a planned engine consumer). The 2026-10-22 amendment that moved the shared
collection model out of `data_table/` was superseded the same day by the evaluation in
[`research/collection-model-2026-10-22.md`](../../research/collection-model-2026-10-22.md): the model moved to rows 360-363, so this
row is back to its original scope.

**Target.** A large-data demo per layout; `layout: Grid` on DataTable (rendered through VirtualGrid with row 341's hooks) with the
same columns and the same `use_data_table` hook and no pagination; docs saying when to use which (the plain Table is the default;
VirtualGrid costs find-in-page, a server render of only the active cell, and screen readers seeing only the window); one shared
table look so `table/style.css` and `virtual_grid/style.css` cannot drift. Done when a spec sorts, filters and selects in the large
variant (including a selected row scrolled out and back) and computed styles show Table and VirtualGrid rows match.

**Landed 2026-10-23 (lane biggrid).**

- **API.** `DataTable { layout: DataTableLayout::Grid, grid_height, grid_label, .. }` (default `DataTableLayout::Table`;
  `Column::grid_width(px)`); the hook `use_data_table_index` returns the derived `View` in a memo, the sibling of `use_data_table`;
  `page_sizes` and `reorderable` are ignored in Grid; the existing API is untouched. The Grid layout reads rows by position from
  `View::indices`, so there are no row copies.
- **Why a `layout` prop** and not a `virtualized` flag or a second component: the layout is explicit and never switched by row count,
  because the grid changes the keyboard and accessibility model (one tab stop and arrow keys against Tab through every control; the
  window against the whole table). One `DataTable<T>` keeps one `Column<T>` and one state; the two layouts are two private components
  behind it (`DataTableTable`, `grid.rs`) because hooks cannot be conditional.
- **Row 341's hooks:** `column_sort` (aria-sort), `row_key`, `row_selected` keyed by it (`DataTableState::is_selected`), a checkbox
  column and sort buttons as controls in cells; selection survives scrolling a row out of the window and a sort.
- **One table look:** the theme's `--dx-table-*` tokens (`preview/assets/dx-components-theme.css`: font size, cell padding, row
  border, head height and weight, hover and selected fill) are read by both `table/style.css` and `virtual_grid/style.css`; a data
  grid's rows now take the Nova row hover. `data_table-large.spec.ts` compares the computed values of a Table row and a Grid row
  (including the 37 px row height and 40 px header).
- **Found on the way:** `merge_attributes` lets a `class: None` replace the base class, so every unaligned `TableCell`/`TableHead`
  of `DataTable` had no `dx-table-cell` (1 px UA padding, no border); the DataTable now passes `""`. The class (a `None`-valued
  class wipes the base class of any themed component forwarding an `Option<&str>`) is in `primitives/src/lib.rs`, left open.
- **Spec:** `playwright/data_table-large.spec.ts` (6 tests: sort, filter, select, a selected row scrolled out and back and through a
  sort, select-all under a filter, the shared look). **Cost left:** a selection toggle re-runs `derive` over all 50,000 rows (the
  memo depends on the whole `ViewState`); a candidate for splitting the memo into (sort, filter, order) and selection counts.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/342-datatable-large-data-variant-on-virtualgrid.md`.*
