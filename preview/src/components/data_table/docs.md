Data Table is a composition pattern, not a new primitive — the same architecture as shadcn/ui's Data Table over TanStack Table. You describe the table once, as typed `Column<T>` definitions, and `DataTable` renders it into [`Table`](/component/table/) plus this crate's `Select`, `Checkbox`, `Button` and `Input`: sortable headers, a filter input, pagination and row selection with an all/some/none header checkbox.

- **`Column<T>`** — an `id`, a header, a cell renderer `Fn(&T) -> Element`, and builders: `.sortable(|row| key)` (a typed key, `K: Ord`, so a sort can never name a column or type that does not exist), `.filterable(|row| text)`, `.width("8rem")`, `.align(ColumnAlign::End)`.
- **`DataTable<T>`** — the component. It takes `rows`, `columns` and a `row_key` (a stable identity, so selection survives sorting, filtering and paging), and opt-in features: `selectable`, `filter_placeholder`, `page_sizes`, `default_sort`. Header and cell counts cannot drift apart, and `aria-sort` sits on the `th` (per the [APG Table pattern](https://www.w3.org/WAI/ARIA/apg/patterns/table/)) and `data-state="selected"` on the row without the page writing either.
- **`use_data_table(rows, &columns, &row_key, options)`** — the state underneath, public for custom layouts. It owns sort, filter, page and selection and returns a `DataTableModel`: the derived `view` (page rows, keys, page count, counts, the header checkbox state) and `state`, a `Copy` handle of mutators (`toggle_sort`, `set_filter`, `set_page_size`, `next_page`, `set_selected`, `set_many_selected`) for event handlers.

Sorting, filtering or changing the page size returns to page 1. A header activation sorts the column ascending, then flips direction; exactly one column carries the sort, as in the APG reference. The filter matches like shadcn's `includesString`: a case-insensitive substring match anywhere in the text of any `filterable` column, not just at the start. The header checkbox covers the current page; the footer reads "selected rows that pass the filter, across every page, of the filtered rows" (shadcn's count, so it never reads "3 of 1"); the selection itself also keeps rows the filter hides.

The state transitions are a pure, unit-tested state machine (`ViewState` and `derive` in `dioxus_primitives::data_view`, the one collection model that `model.rs` and `state.rs` re-export as `TableState`, `derive`, `sort_rows_by`, `filter_rows` and `paginate`; `derive` returns indices and counts, and `use_data_table` memoises it and clones only the page's rows), and the three pieces below stay available for tables the component does not fit.

- `DataTableColumnHeader` — a sortable `TableHead`: a ghost `Button` toggles the sort and an icon shows the direction.
- `DataTableToolbar` — a text filter `Input` with an accessible name.
- `DataTablePagination` — a "x of y row(s) selected" summary, a rows-per-page `Select`, a "Page n of m" readout, and Previous/Next `Button`s (disabled at either end).

`DataTable` renders every row in the DOM, one page at a time. A virtualized layout for tens of thousands of rows is planned (backlog rows 343, 341, 342); it will take the same `Column<T>`.

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
