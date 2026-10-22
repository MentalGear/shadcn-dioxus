---
id: 341
title: "VirtualGrid: header sort state, row selection, and controls inside cells"
status: "2026-10-22: hooks keyed by row identity (row_key, row 361); comes after rows 360-362…"
size: "M (about 1.5 to 2 days)"
area: primitives
updated: 2026-10-22
---

# Row 341: VirtualGrid: header sort state, row selection, and controls inside cells

## Item

Add the three hooks VirtualGrid needs before it can host a DataTable: `aria-sort` on header cells, a selected state on rows, and keyboard handling for a control (such as a checkbox) inside a cell.

## Status / trigger

**2026-10-22:** hooks keyed by row identity (`row_key`, row 361); comes after rows 360-362 ([`research/collection-model-2026-10-22.md`](../../research/collection-model-2026-10-22.md)).

BACKLOG (owner 2026-10-21; amended 09:35). Comes after row 343 (needs its VirtualizeMode); row 342 needs it.

## Size

M (about 1.5 to 2 days)

## Detail

Evaluation: https://claude.ai/artifact/CGKtSVebCVar2PmKXNWs5T (Table, DataTable and VirtualGrid). Evidence from `main` `fd083a2d`. Order: 340, then 341, then 342; 341 and 342 only pay off if tens-of-thousands-row tables are a real use case (row 154 lists DataTable rows as a planned engine consumer).

- **Today** (`primitives/src/virtual_grid.rs`):
  - The `columnheader` element belongs to VirtualGrid, and `render_header` only supplies its content, so a caller cannot set `aria-sort` on it.
  - Rows (`role="row"`) take no caller attributes, so there is no `aria-selected` or `data-state="selected"` and no way to style a selected row.
  - The active cell is the grid's one tab stop. A checkbox inside a cell adds a second tab stop, and the container's `onkeydown` (`grid_key`, around `:924`) moves focus away from it on the arrow keys.
  - Row heights are fixed (`row_height`). That is fine for data tables, so this row leaves it as it is.
- **Target:**
  - A header attributes hook, for example `column_sort: Callback<usize, Option<SortDirection>>` or a general `header_attributes: Callback<usize, Vec<Attribute>>`.
  - A row hook, `row_selected: Callback<usize, bool>` (emits `aria-selected` and `data-state`), with a matching rule in `virtual_grid/style.css`.
  - Controls inside cells, following the APG grid pattern: when the active cell holds one focusable control, focus goes to that control, and it keeps `tabindex=-1` otherwise. Space or Enter act on it, and the arrow keys still move between cells. Settle the exact model (one control per cell, or Enter/F2 for an "interaction mode") in the row before building it.
- **Done when:** a spec covers each hook (aria-sort on the header, aria-selected on rows that survive a window move, and Tab, Space and the arrow keys with a checkbox cell), and the existing VirtualGrid specs and oracle stay green.

## Amendment (owner 2026-10-21, 09:35)

New order (owner 2026-10-21, 09:35): 340, then 343, then 341, then 342. Design: row 340's amendment (one `VirtualizeMode` for every consumer, one data-driven `DataTable<T>`, explicit layout with an automatic mode).

**This row:** Besides the three hooks, VirtualGrid adopts the shared `VirtualizeMode` from row 343 (content-visibility row chunks while `rows <= auto_threshold`, windowed above), about +0.5 day. It needs row 343's mode type, so it comes after 343.

## Landed (2026-10-22, lane `gridhooks`)

API (all optional props of `VirtualGrid`, `primitives/src/virtual_grid.rs`):
- `column_sort: Callback<usize, Option<SortDirection>>`: `aria-sort` on that column's header (only the sorted one carries it). `SortDirection` is re-exported from `data_view`.
- `row_selected: Callback<String, bool>`: asked per mounted row with `render_key(row_key, index)`, so selection is keyed by row identity (row 361). Emits `aria-selected="true|false"` and `data-state="selected"` on selected rows; `style.css` paints it `--dx-muted` (sticky column too).
- Controls in cells: no prop; the grid detects a cell's control.

**Control-model decision: one control per cell, "focus the widget" (APG), no Enter/F2 interaction mode.** Why: it keeps exactly one tab stop with no mode state, `Space`/`Enter` work natively on a checkbox/switch/button/link, and arrows keep moving between cells because the focused control does not consume them. The interaction mode is needed only for text fields, selects and multi-control cells (which swallow arrows); a data table's row checkbox and header sort button, the case this row exists for, need none. Cells with such content are out of scope and documented as such. Mechanism: a MutationObserver (coalesced per frame, `listen`-free, stops when the grid leaves the page) sets the cell `tabindex=-1` and the control `0` in the active cell, `-1` elsewhere; the active-cell focus script focuses the control when the cell has one; the cell's `onfocusin` (not `onfocus`) makes the cell active when a control inside takes focus. The wrap and justified grids are unchanged.

Not done: the amendment's `VirtualizeMode` adoption (row 343's type is not in this tree); left to row 343/342.

Specs: `playwright/virtual-grid-hooks.spec.ts` on the new `selectable` demo (10,000 orders, checkbox column, sortable headers).
