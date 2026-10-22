---
id: 363
title: "DataTable: reorderable rows (drag handle) and a Columns menu (visibility, keyboard column order)"
status: "LANDED on claude/dtreorder (merge pending): reorderable handle column, ColumnLayout + Columns menu…"
size: "M."
area: components
updated: 2026-10-22
---

# Row 363: DataTable: reorderable rows (drag handle) and a Columns menu (visibility, keyboard column order)

## Item

Migration step 5: `reorderable` adds a drag-handle column (dashboard-01 parity) driving the manual order; `ColumnLayout` with a Columns menu for visibility and keyboard column moves; the Table rich demo renders from the model. Needs rows 360-362.

## Status / trigger

LANDED on claude/dtreorder (merge pending): reorderable handle column, ColumnLayout + Columns menu, rich Table demo from the model (see "Landed" below). Design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md).

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.

## Landed (claude/dtreorder)

- `DataTable { reorderable: true }` adds a leading drag-handle column. The handle is a button (`data-dx-reorder-handle`, `aria-roledescription="drag handle"`, `aria-label="Reorder row <key>"`) that uses `dioxus_primitives::drag_and_drop_list::reorder` (pointer drag, Space/Enter lift, arrows, Escape, live-region announcements); the move writes `ViewState`'s manual `KeyOrder`. No second gesture or ordering implementation. With nothing lifted, arrows/Home/End move focus between handles.
- `ColumnLayout` (`data_table/layout.rs`: order = `KeyOrder`, hidden set, `visible`/`arranged`/`move_listed`, unit-tested) and `DataTable { column_menu: true }`: a Columns `DropdownMenu` of checkbox items (`Column::hideable(false)` keeps one out); Alt+Up / Alt+Down on an item moves the column and announces it. `use_data_table` exposes `layout`.
- Table "rich" demo renders from `use_data_table` (rows, keys, selection, header tri-state).
- Demo `data_table` variant `reorder`; spec `playwright/data_table_reorder.spec.ts` (7 tests).
- Primitives: small additions to `data_view/state.rs` (reorder/selection helpers).
