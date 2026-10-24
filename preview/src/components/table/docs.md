The Table component displays tabular data using plain semantic HTML (`<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>`, `<caption>`) — the same architecture as shadcn/ui's `Table`: no ARIA widget role, no primitive underneath.

## Component Structure

```rust
Table {
    TableCaption { "A list of your recent invoices." }
    TableHeader {
        TableRow {
            TableHead { "Invoice" }
            TableHead { "Status" }
            TableHead { "Amount" }
        }
    }
    TableBody {
        TableRow {
            TableCell { "INV001" }
            TableCell { "Paid" }
            TableCell { "$250.00" }
        }
    }
    TableFooter {
        TableRow {
            TableCell { "Total" }
            TableCell {}
            TableCell { "$250.00" }
        }
    }
}
```

`TableHead` defaults to `scope="col"`; pass `scope: "row"` to use it as a row header instead. `TableRow` accepts an optional `selected: ReadSignal<bool>` prop that sets `data-state="selected"` for row-selection styling — used by the [Data Table](/component/data_table/) composition pattern, which builds sorting, filtering, pagination, and row selection on top of this component.

## Long tables: `TableRows`

`TableBody` is for rows you write by hand. For a table of data, `TableRows` is the body: it takes a row count and a renderer, owns each `<tr>`, and keeps the table cheap with the same `virtualize` and `auto_threshold` props as `VirtualList`.

```rust
Table {
    // The container scrolls: give it a height, and `TableHeader` can stay put above the rows.
    // A scroll container must be keyboard-reachable: `tabindex`, a role and a name.
    container_attributes: vec![
        Attribute::new("style", "max-height: 24rem;", None, false),
        Attribute::new("tabindex", "0", None, false),
        Attribute::new("role", "region", None, false),
        Attribute::new("aria-label", "Invoices", None, false),
    ],
    TableHeader { sticky: true,
        TableRow { TableHead { "Invoice" } TableHead { "Status" } TableHead { "Amount" } }
    }
    TableRows {
        count: invoices.len(),
        columns: 3usize,
        // The cells of row `idx`; the body supplies the `<tr>`.
        render_row: move |idx: usize| rsx! {
            TableCell { "{invoices[idx].id}" }
            TableCell { "{invoices[idx].status}" }
            TableCell { "{invoices[idx].amount}" }
        },
        // Optional: `data-state="selected"` on the row.
        row_selected: move |idx: usize| selected().contains(&idx),
        // Optional: `Auto` (the default), `ContentVisibility` or `Windowed`.
        virtualize: VirtualizeMode::Auto,
    }
}
```

- **Windowed** mounts only the rows in view (plus `buffer`, default 8, each side) between two `aria-hidden` spacer rows, so the table stays a real `<table>`: `<thead>`, `<tr>`, `<td>`. Every row carries `aria-rowindex` (the header counts as row 1; `row_offset` says how many rows come before the data) and the `<table>` carries `aria-rowcount` of the whole table, so a screen reader announces `row 4,001 of 20,001`. Rows are measured as they mount; `estimate_size` (default 37 px) is the first guess. A row that holds focus is not kept mounted when the window leaves it.
- **`ContentVisibility`** keeps every row in the DOM and in the server's HTML. Unlike a list, a table cannot let the browser skip the off-screen rows: browsers do not apply `content-visibility` to table rows or row groups (measured in Chromium: a `<tbody>` or `<tr>` with `content-visibility: auto` keeps its full height). So here the mode is a plain table, and its cost grows with the row count, which is why `Auto` leaves it at 1,000 rows.
- **`item_key`** (optional, `Callback<usize, String>`) says who is at row `idx`. Without it a row is its slot, so after a sort the `<tr>` (and focus, or a checkbox the row owns) stays on the slot while the data moves on; with it each `<tr>` is keyed by its item and the measured row heights follow the item. The `reorder` demo shows both.
- Give a fixed column layout (`table-layout: fixed` on the `Table`) when the first paint matters: the server renders a windowed table without rows, so with automatic layout the columns resize once the window mounts.
- `TableBody` stays as it is, for hand-written rows; it cannot be windowed, because its rows are children, not data.

<!-- shared:virtual-engine:modes -->
## The three modes and what they cost

One mode type, `VirtualizeMode`, lives in the shared engine and is the `virtualize` prop of every component that chooses between keeping rows in the DOM and windowing them: `VirtualList`, `VirtualGrid`, `VirtualWrapGrid` and `TableRows` (the data body of a `Table`). `auto_threshold` (default 1,000) is the one switch point of `Auto`, counted in items: a list's rows, a grid's cells (`row_count x column_count`, or a wrapping grid's `count`). `VirtualJustifiedGrid` is always windowed.

| | `ContentVisibility` | `Windowed` | `Auto` (the default) |
|---|---|---|---|
| In the DOM | every row | the rows in view plus an overscan | `ContentVisibility` while `count <= auto_threshold`, `Windowed` above |
| Server-rendered HTML | every row | none | follows the mode |
| Find-in-page (Ctrl+F), selection | reach every row (skipped content: not Safari 18-25) | only the mounted window | follows the mode |
| Screen readers | every row | only the window, each row announcing its place (`3 of 100000`) | follows the mode |
| Cost grows with the row count | yes: DOM and memory are linear | no: flat to millions of rows | follows the mode |
| How off-screen rows are kept cheap | a list or grid: the browser skips them in chunks of 20 rows (`content-visibility: auto`); a table: not at all, browsers do not skip table rows | not mounted | follows the mode |

The mode follows the live `count`, so a component whose count crosses the threshold re-mounts its rows in the other mode (its scroll position resets); choose an explicit mode when the count hovers around the threshold.
<!-- /shared:virtual-engine:modes -->
