---
id: 343
title: "One virtualize mode in the engine, and a data body for Table"
status: "LANDED 2026-10-22 (batch 6, lane virtualize). VirtualizeMode { Auto, ContentVisibility, Windowed }…"
size: "M (about 2 to 2.5 days)"
area: primitives
updated: 2026-10-22
---

# Row 343: One virtualize mode in the engine, and a data body for Table

## Item

Move `VirtualListMode` into the shared engine as `VirtualizeMode`, and give Table a data-driven body (`TableRows`) with the same `virtualize: Auto | ContentVisibility | Windowed` and `auto_threshold` API, windowing rows inside a real `<table>`.

## Status / trigger

**LANDED 2026-10-22 (batch 6, lane virtualize).** `VirtualizeMode { Auto, ContentVisibility, Windowed }` and `DEFAULT_AUTO_THRESHOLD` live in `primitives/src/virtual/mode.rs` (`VirtualListMode` is a deprecated alias); new `TableRows` primitive windows rows inside a real `<table>` with top and bottom spacer rows, `aria-rowindex`, and `aria-rowcount` on the table via `TableScroller`; themed `Table { container_attributes }`, `TableHeader { sticky }`, `TableRows { row_selected }`, demo `table/variants/virtualized`, `playwright/table-rows.spec.ts` (4/4). Measured: `content-visibility` does not apply to table rows or row groups (spec excludes internal table elements from size containment), so ContentVisibility mode keeps every row in the DOM. The table route joined first-paint-styles `CLIENT_MEASURED` like virtual_list. Rows 341 and 342 can now build on it.

## Size

M (about 2 to 2.5 days)

## Detail

Owner order 2026-10-21 09:35: 340, then this row, then 341, then 342 (design in row 340's amendment). Today the mode type and
`resolve` lived in `primitives/src/virtual_list.rs` (default threshold 1000) and Table had no virtualization. **Landed 2026-10-22
(batch 6, lane virtualize, branch `claude/virtualize`).**

- **Shared mode.** `primitives/src/virtual/mode.rs`: `VirtualizeMode { Auto, ContentVisibility, Windowed }`, `resolve`, `as_str`,
  `DEFAULT_AUTO_THRESHOLD` (4 unit tests moved with it), public through `virtual_list::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD}`
  and `table_rows`; `VirtualListMode` is a `#[deprecated]` alias and the VirtualList demos and docs use the new name.
- **`TableRows`.** `primitives/src/table_rows.rs`: a `<tbody>` with rows from `count` plus `render_row` (returns the cells),
  `row_attributes`, `row_offset`, `buffer`, `estimate_size`, `virtualize`, `auto_threshold`, built on the one-axis `Engine` and
  `use_virtual_bridge`. Windowed draws a top and a bottom spacer `<tr>` (one `aria-hidden` cell, `colspan = columns`) and puts
  `aria-rowindex` on every real row; the `TableScroller` context (`use_table_scroller`) names the scrolling container and carries
  `aria-rowcount` up to the `<table>` (ARIA does not allow `aria-rowcount` on a row group).
- **Themed layer.** `Table` provides the scroller, owns the container `id`, takes `container_attributes` (give it a `max-height`);
  `TableHeader { sticky }` (`data-sticky`); themed `TableRows { row_selected }`; demo `table/variants/virtualized` (20,000 / 1,001 /
  1,000 rows); `playwright/table-rows.spec.ts` (4/4). One shared docs block `modes` (the three modes and their costs on
  find-in-page, server render and screen readers) in `virtual_list/docs.md`, `virtual_grid/docs.md` and `table/docs.md`, held
  byte-identical by `check-virtual-docs-parity.sh`.
- **Finding that changed the plan.** `content-visibility` does not apply to table rows or row groups (measured in Chromium 1194: a
  `<tbody>` or `<tr>` with `content-visibility: auto` keeps its real height and skips nothing; the specification excludes internal
  table elements from size containment). So `TableRows` in `ContentVisibility` mode is every row in the DOM and nothing more
  (find-in-page works, cost is linear), and `TableBody` gets no content-visibility class. The docs say so. Left out on purpose: no
  pinned focus row in a windowed table (a spacer layout has nowhere to draw a detached row).
- **For rows 341 and 342.** `VirtualGrid` takes `VirtualizeMode` from the same module; `DataTable<T>` can render `TableRows` inside
  `Table { container_attributes }` with `TableHeader { sticky: true }`; `Column<T>` and `use_data_table` are untouched.
- **Verification** (lane-ssg of `4428b41f`): `table-rows.spec.ts` 4/4; the `table`, `virtual_list`, `virtual-grid` and `table-rows`
  specs 59 passed; the first broad `test:changed` run (1147 passed) had 3 axe reds (scroller not focusable, fixed in the demo) and 2
  first-paint reds: a windowed body renders no rows on the server, so its bottom spacer is 740000 px there and 739334 px once rows
  mount, so `/component/table/` joined first-paint-styles `CLIENT_MEASURED` with `virtual_list` and `data_table`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/343-one-virtualize-mode-in-the-engine-and-a-data.md`.*
