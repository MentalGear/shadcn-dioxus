---
id: 342
title: "DataTable: a large-data variant rendered through VirtualGrid"
status: "BACKLOG (owner 2026-10-21; amended 09:35; scope widened 2026-10-22). Needs rows 340, 343 and 341.…"
size: "S (about 1 day)"
area: components
updated: 2026-10-22
---

# Row 342: DataTable: a large-data variant rendered through VirtualGrid

## Item

A DataTable demo that renders the row 340 model through VirtualGrid (for example 50,000 rows, sortable, filterable and selectable, scrolling in place of pages), and one table look shared by Table and VirtualGrid.

## Status / trigger

BACKLOG (owner 2026-10-21; amended 09:35; scope widened 2026-10-22). Needs rows 340, 343 and 341.

**Amendment 2026-10-22, superseded the same day by the evaluation in [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md): the collection model moved to rows 360-363, so this row is back to its original scope (VirtualGrid layout, large-data demo, shared table look) on top of them.** Original amendment, kept for the record: one collection model for every view. Before the VirtualGrid variant, move row 340's `TableState` + `derive` out of `data_table/` into one shared collection model that Table/`TableRows`, VirtualList, VirtualGrid and the drag-and-drop list all render from:
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
