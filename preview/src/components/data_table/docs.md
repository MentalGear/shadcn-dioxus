Data Table is a composition pattern, not a new primitive — the same architecture as shadcn/ui's Data Table over TanStack Table. You describe the table once, as typed `Column<T>` definitions, and `DataTable` renders it into [`Table`](/component/table/) plus this crate's `Select`, `Checkbox`, `Button` and `Input`: sortable headers, a filter input, pagination and row selection with an all/some/none header checkbox.

- **`Column<T>`** — an `id`, a header, a cell renderer `Fn(&T) -> Element`, and builders: `.sortable(|row| key)` (a typed key, `K: Ord`, so a sort can never name a column or type that does not exist), `.filterable(|row| text)`, `.width("8rem")`, `.align(ColumnAlign::End)`.
- **`DataTable<T>`** — the component. It takes `rows`, `columns` and a `row_key` (a stable identity, so selection survives sorting, filtering and paging), and opt-in features: `selectable`, `filter_placeholder`, `page_sizes`, `default_sort`, and the change callbacks `on_sort_change`, `on_page_change` and `on_selection_change` (TanStack's `onSortingChange`, `onPaginationChange` and `onRowSelectionChange`, reporting the new value: the new sort or `None`, the 0-based page, every selected key; never on the first render). Header and cell counts cannot drift apart, and `aria-sort` sits on the `th` (per the [APG Table pattern](https://www.w3.org/WAI/ARIA/apg/patterns/table/)) and `data-state="selected"` on the row without the page writing either.
- **`use_data_table_index(rows, &columns, &row_key, options)`** — the same state with the derived `View` (indices and keys) in a memo instead of copied rows; what the Grid layout renders from.
- **`use_data_table(rows, &columns, &row_key, options)`** — the state underneath, public for custom layouts. It owns sort, filter, page and selection and returns a `DataTableModel`: the derived `view` (page rows, keys, page count, counts, the header checkbox state) and `state`, a `Copy` handle of mutators (`toggle_sort`, `clear_sort`, `set_filter`, `set_page_size`, `set_page`, `next_page`, `set_selected`, `set_many_selected`) for event handlers. The same three callbacks go in its `DataTableOptions`; they are read on every render, the initial values only on the first.

Sorting, filtering or changing the page size returns to page 1. A header activation sorts the column ascending, then descending, then clears the sort (TanStack's default cycle), which restores the source order, or the manual order of a `reorderable` table; exactly one column carries the sort, as in the APG reference. The filter matches like shadcn's `includesString`: a case-insensitive substring match anywhere in the text of any `filterable` column, not just at the start. The header checkbox covers the current page; the footer reads "selected rows that pass the filter, across every page, of the filtered rows" (shadcn's count, so it never reads "3 of 1"); the selection itself also keeps rows the filter hides.

The state transitions are a pure, unit-tested state machine (`ViewState` and `derive` in `dioxus_primitives::data_view`, the one collection model that `model.rs` and `state.rs` re-export as `TableState`, `derive`, `sort_rows_by`, `filter_rows` and `paginate`; `derive` returns indices and counts, and `use_data_table` memoises it and clones only the page's rows), and the three pieces below stay available for tables the component does not fit.

- `DataTableColumnHeader` — a sortable `TableHead`: a ghost `Button` toggles the sort and an icon shows the direction.
- `DataTableToolbar` — a text filter `Input` with an accessible name.
- `DataTablePagination` — a "x of y row(s) selected" summary, a rows-per-page `Select`, a "Page n of m" readout, and Previous/Next `Button`s (disabled at either end).

## Table or Grid layout

`DataTable` draws its rows one of two ways, chosen with `layout`, never switched for you by the row count (the two have different keyboard and accessibility models):

- **`DataTableLayout::Table`** (the default) is a real `<table>` one page at a time. Every row of the page is in the DOM, Tab walks the controls, find-in-page and the server's HTML reach every row on the page. Use it unless you have tens of thousands of rows.
- **`DataTableLayout::Grid`** draws the same columns and the same state through [`VirtualGrid`](/component/virtual_grid/) with no pagination: the table scrolls, and only the rows in view (plus a few) exist. Use it for tens of thousands of rows. Sorting, filtering, selection (kept by `row_key`, so a selected row survives scrolling away and back, and a sort) and the Columns menu work the same; the header carries `aria-sort`, and the checkbox column is a control in a cell. Give each column a pixel width with `.grid_width(px)` (the Table layout takes the CSS `.width`), and the box a height with `grid_height`.

The Grid layout's costs are the virtual grid's: no find-in-page across the rows, the server renders only the header (the rows appear when the page hydrates), screen readers see only the window (rows announce their position of the total), the arrow keys move between cells with one tab stop instead of Tab walking every control, and `page_sizes` and `reorderable` are ignored. The Grid layout reads rows by position from the derived indices (`use_data_table_index`), so it never copies the data; its header checkbox covers every row that passes the filter.

Both layouts read one table look from the theme's `--dx-table-*` tokens (text size, cell padding, row border, header weight and height, hover and selected fill), so a row of either looks the same.

## Component Structure

```rust
let columns = vec![
    Column::new("email", "Email", |p: &Payment| rsx! { "{p.email}" })
        .sortable(|p| p.email)
        .filterable(|p| p.email.to_string()),
    Column::new("amount", "Amount", |p: &Payment| rsx! { "{format_amount(p.amount_cents)}" })
        .sortable(|p| p.amount_cents),
];

DataTable::<Payment> {
    rows: payments,
    columns,
    row_key: RowFn::new(|p: &Payment| p.id.to_string()),
    selectable: true,
    filter_placeholder: "Filter emails...".to_string(),
    page_sizes: vec![5, 10, 20],
    default_sort: Some(("email".to_string(), SortDirection::Ascending)),
}
```
