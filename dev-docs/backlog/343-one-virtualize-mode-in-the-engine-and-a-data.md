---
id: 343
title: "One virtualize mode in the engine, and a data body for Table"
status: "BACKLOG (owner 2026-10-21). Comes after 340 and before 341 and 342."
size: "M (about 2 to 2.5 days)"
area: primitives
updated: 2026-10-22
---

# Row 343: One virtualize mode in the engine, and a data body for Table

## Item

Move `VirtualListMode` into the shared engine as `VirtualizeMode`, and give Table a data-driven body (`TableRows`) with the same `virtualize: Auto | ContentVisibility | Windowed` and `auto_threshold` API, windowing rows inside a real `<table>`.

## Status / trigger

BACKLOG (owner 2026-10-21). Comes after 340 and before 341 and 342.

## Size

M (about 2 to 2.5 days)

## Detail

New order (owner 2026-10-21, 09:35): 340, then 343, then 341, then 342. The shared design is in row 340's "Amendment" section.

- **Today:** the mode type and `resolve` live in `primitives/src/virtual_list.rs:66-117` (default threshold 1000, cv chunk size at `:74`). Only VirtualList has it. Table renders whatever children it gets and has no virtualization.
- **Target:**
  - `VirtualizeMode` and `resolve` in `primitives/src/virtual/`, with `VirtualListMode` kept as a deprecated alias for one release, so existing callers keep compiling.
  - `TableRows` (themed in `table/component.rs`, with the logic in a primitive): content-visibility chunks using `cv_chunks.rs` as VirtualList does, and Windowed using the one-axis `Engine` and `use_virtual_bridge`, with a top and bottom spacer `<tr>` (one cell spanning all columns, `aria-hidden`) and `aria-rowcount` and `aria-rowindex` on the real rows.
  - Hand-written `TableBody` gets an opt-in content-visibility class only.
  - The docs give one table of the three modes and their costs (find-in-page, server render, screen readers), shared by VirtualList, Table and VirtualGrid.
- **Done when:** specs cover each mode on `TableRows` (row count in the DOM, a sticky `<thead>` while scrolling, Auto switching at the threshold, find-in-page working in content-visibility mode), and the VirtualList specs pass unchanged after the move.

## Landed on branch `claude/virtualize` (2026-10-22, lane `virtualize`); browser run still to do

- `primitives/src/virtual/mode.rs`: `VirtualizeMode { Auto, ContentVisibility, Windowed }`, `resolve`, `as_str`, `DEFAULT_AUTO_THRESHOLD` (4 unit tests moved with it), public through `virtual_list::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD}` and `table_rows`. `VirtualListMode` is a `#[deprecated]` type alias (variants still resolve through it); the VirtualList demos and docs use the new name.
- `primitives/src/table_rows.rs`: `TableRows` (a `<tbody>`, rows from `count` + `render_row` that returns the cells, `row_attributes`, `row_offset`, `buffer`, `estimate_size`, `virtualize`, `auto_threshold`) on the one-axis `Engine` and `use_virtual_bridge`; Windowed draws a top and a bottom spacer `<tr>` (one `aria-hidden` cell, `colspan = columns`) and puts `aria-rowindex` on every real row; `TableScroller` context (`use_table_scroller`) names the scrolling container and carries `aria-rowcount` up to the `<table>` (an ARIA `aria-rowcount` is not valid on a row group, so it is on the table, not on the rows as the row text said).
- `table/component.rs`: `Table` provides the scroller, owns the container `id`, takes `container_attributes` (give it a `max-height`), `TableHeader { sticky }` (`data-sticky`, CSS in `style.css`), themed `TableRows` with `row_selected`. New demo `table/variants/virtualized` (20,000 / 1,001 / 1,000 rows), docs section and `playwright/table-rows.spec.ts`.
- Docs: one shared block `modes` (three modes and their costs on find-in-page, server render, screen readers) in `virtual_list/docs.md` and `virtual_grid/docs.md`; `table/docs.md` carries the same block, and `check-virtual-docs-parity.sh` now also holds the table copy to the list's text.
- **Finding that changes the row's plan:** `content-visibility` does not apply to table rows or row groups. Measured in Chromium 1194: a `<tbody>` or `<tr>` with `content-visibility: auto; contain-intrinsic-block-size: auto 500px` keeps its real height (800 px) and skips nothing; the specification excludes internal table elements from size containment. So `TableRows` in `ContentVisibility` mode is every row in the DOM and nothing more (find-in-page works, cost is linear), and `TableBody` gets no content-visibility class (it would do nothing). Auto at 1,000 keeps that cheap. The docs say so.
- Left out on purpose: no pinned focus row in a windowed table (a spacer layout has nowhere to draw a detached row), documented.
- For 341/342: `VirtualGrid` takes `VirtualizeMode` from the same module (`crate::r#virtual::VirtualizeMode`); `DataTable<T>` can render `TableRows { count: view.len(), columns, render_row, row_selected }` inside `Table { container_attributes }` with `TableHeader { sticky: true }`; `Column<T>` and `use_data_table` are untouched.
- Evidence so far: `cargo check`, clippy `-D warnings` (primitives + preview, `web`) clean, `cargo test -p dioxus-primitives --lib virtual` 188 passed, fast gates green except `check-slow-spec-budget` (needs the new spec's duration entry from a real run).
