The VirtualGrid component keeps a very large grid cheap by mounting only the cells in view. It comes in two layouts: `VirtualGrid`, a data grid whose rows AND columns are windowed (with an optional sticky header row and sticky first column), and `VirtualWrapGrid`, a gallery whose items wrap into rows of a fixed column count (only the rows are windowed).

## Component Structure

```rust
// A data grid: 10,000 rows x 50 columns, a few hundred cells in the DOM.
VirtualGrid {
    row_count: 10_000usize,
    column_count: 50usize,
    // Optional: px per row / column (default 36 / 120).
    row_height: |_row: usize| 36,
    column_width: |column: usize| if column == 0 { 80 } else { 112 },
    // Optional: a header row that sticks to the top, and a first column that sticks to the inline start.
    render_header: |column: usize| rsx! { "Column {column}" },
    sticky_first_column: true,
    render_cell: |(row, column): (usize, usize)| rsx! { "{row}, {column}" },
    "aria-label": "Readings",
    style: "height: 24rem;",
}

// A gallery: 5,000 items in rows of 4, rows measured as they render.
VirtualWrapGrid {
    count: 5_000usize,
    columns: 4usize,
    estimate_row_height: |_row: usize| 140,
    render_item: |index: usize| rsx! { figure { "Photo {index}" } },
    "aria-label": "Photos",
    style: "height: 28rem;",
}
```

Give the grid a height (or `max-height`): it is the scroll container. The grid owns its `id` and `role="grid"`; every other attribute is spread onto it. Name it with `aria-label` (or `aria-labelledby`).

## Layouts

| | `VirtualGrid` | `VirtualWrapGrid` |
|---|---|---|
| Shape | `row_count x column_count` cells | `count` items in rows of `columns` |
| Windowed axes | rows and columns | rows |
| Sizes | given: `row_height(row)`, `column_width(column)` | measured per row; `estimate_row_height` is the first guess |
| Sticky edges | optional header row and first column | none |
| Cell renderer | `render_cell((row, column))` | `render_item(index)` |

Both use the shared engine of `VirtualList`'s windowed mode, one per axis: positions are kept in a Fenwick tree, the mounted range is re-centred only once the viewport comes within half a buffer of its edge (so a scroll re-renders about once every few rows or columns, not once per scroll event), and the scroll canvas keeps its size while a scroll is running. `row_buffer` (default 6) and `column_buffer` (default 4) set how many rows and columns are mounted beyond each edge; `VirtualWrapGrid` has `buffer` (default 4 rows).

In the 10,000 x 50 demo about 300 of the 500,000 cells are in the DOM at any time, wherever you scroll.

## Keyboard and focus

The grid follows the WAI-ARIA grid pattern. Exactly one cell is in the tab order (the active cell, `tabindex="0"`); click a cell or Tab into the grid, then:

| Key | Moves to |
|---|---|
| Arrow keys | the neighbouring cell (stops at the edges) |
| `Home` / `End` | the first / last cell of the row |
| `Ctrl+Home` / `Ctrl+End` | the first / last cell of the grid |
| `PageUp` / `PageDown` | a viewport of rows up / down, same column |

**The active cell is always rendered**, even when it is far outside the window (after `Ctrl+End`, or when you scroll away from it): a windowed grid that unmounted it would lose focus and its only tab stop. Each move scrolls the cell into view just past the sticky header and column. `on_active_change` reports the active cell.

In right-to-left layouts the grid mirrors: the sticky column sits at the right, and `ArrowLeft` moves to the next cell (inline end). The direction comes from `dir`, else the nearest `DirectionProvider`; the layout itself follows the inherited `dir` attribute through logical CSS properties.

## Accessibility

The container is `role="grid"` with `aria-rowcount` and `aria-colcount` of the whole grid, and every mounted row and cell carries `aria-rowindex` / `aria-colindex`, so a screen reader announces where a cell is even though only a window exists. Header cells are `role="columnheader"`; with `sticky_first_column` the first column's cells are `role="rowheader"`.

## What windowing costs

Cells outside the window do not exist: the browser's find-in-page cannot reach them, the accessibility tree holds only the window, and the server renders only the active cell. A `VirtualGrid` cell taller than its row overflows it (rows are not measured). For a grid of a few thousand cells, a plain CSS grid with `content-visibility: auto` on each row may be the simpler answer: everything stays in the DOM and the browser skips the rows that are off-screen.

## Styling

Rows are `[role=row]`; cells carry `data-grid-cell="row:column"` (the header's is `"header:column"`), and the sticky header and column carry `data-grid-header` / `data-grid-sticky`. Positions and sizes are inline styles; this component's `style.css` paints borders, the header, the sticky column's background and the focus ring. In `VirtualWrapGrid` each row is a CSS grid of equal tracks: set `column-gap` and `padding-block-end` on `[role=row]` for the gaps (they are part of the measured row).
