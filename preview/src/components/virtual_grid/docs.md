The VirtualGrid component keeps a very large grid cheap by mounting only the cells in view. It comes in three layouts: `VirtualGrid`, a data grid whose rows AND columns are windowed (with an optional sticky header row and sticky first column), `VirtualWrapGrid`, a gallery whose items wrap into rows of a fixed column count (only the rows are windowed), and `VirtualJustifiedGrid`, a photo wall of justified rows (every photo uncropped at its own aspect ratio, rows filling the width edge to edge).

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

## Grid or Justified

Two models for showing many photos of mixed shapes:

- **Grid** (`VirtualGrid`, `VirtualWrapGrid`; the default): cells aligned in rows and columns, and the structure never depends on the content. A photo of any aspect is a presentation concern inside a uniform cell: `object-fit: contain` shows it whole on a mat, `cover` crops it to fill. Because rows and columns are independent, both axes window independently, `aria-colcount` is exact and the arrow keys move along straight columns. Pick it for anything tabular, for thumbnails where alignment and predictable scanning matter, and whenever items have no meaningful aspect ratio.
- **Justified** (`VirtualJustifiedGrid`): the gapless, uncropped photo-wall look. Each row takes as many photos as fit at `target_row_height` and then shrinks to fill the width exactly, so a row's composition depends on the photos' aspect ratios and every tile IS its photo's aspect. There are no columns: `ArrowUp`/`ArrowDown` move to the tile of the next row whose horizontal centre is closest. Pick it when the photos themselves are the content and cropping or letterboxing would hurt.

```rust
// 20,000 photos; the aspect ratios (width / height) are all the layout needs.
VirtualJustifiedGrid {
    aspects,                       // ReadSignal<Vec<f64>>
    target_row_height: 150.0,      // px a full row is packed toward (default 150)
    gap: 4.0,                      // px between tiles and rows (default 4)
    // Optional: a canvas wider than the grid makes a wall that scrolls both ways.
    canvas_width: Some(4800.0),
    render_tile: |(index, rect): (usize, TileRect)| rsx! {
        img { src: photo_url(index), alt: "Photo {index}", style: "width: 100%; height: 100%;" }
    },
    "aria-label": "Photos",
    style: "height: 28rem;",
}
```

Without `canvas_width` the rows fill the grid's own width, the grid scrolls vertically only and lays the rows out again when it is resized (keeping the row at the top of the viewport in place). With a `canvas_width` larger than the grid, the rows fill that canvas: the grid scrolls both ways and mounts only the rows that meet the viewport and, within each, only the tiles that meet it horizontally. The full row of a few dozen tiles is never mounted whole. `min_tile_width` keeps a narrow container from squeezing a portrait photo to a sliver (the row gives up photos to the next one and grows taller). Each tile is placed with `inset-inline-start`, so in a right-to-left page the wall mirrors (rows start at the right) and `ArrowLeft` moves to the next photo. The example is the `justified` variant: a target-row-height slider and a "Wall (both axes)" switch that sets `canvas_width` to six times the demo's width.

Keyboard: one tab stop; `ArrowLeft`/`ArrowRight` move in reading order and continue into the previous/next row, `ArrowUp`/`ArrowDown` move to the nearest tile by horizontal centre, `Home`/`End` go to the row's ends, `Ctrl+Home`/`Ctrl+End` to the first/last photo, `PageUp`/`PageDown` a viewport up/down. The active tile stays rendered wherever you scroll. The container is `role="grid"` with the whole wall's `aria-rowcount`; every row carries `aria-rowindex` and every tile its position in its row as `aria-colindex`. `aria-colcount` is omitted: rows hold different numbers of tiles, so no column count would be true.

## Layouts

| | `VirtualGrid` | `VirtualWrapGrid` |
|---|---|---|
| Shape | `row_count x column_count` cells | `count` items in rows of `columns` |
| Windowed axes | rows and columns | rows |
| Sizes | given: `row_height(row)`, `column_width(column)` | measured per row; `estimate_row_height` is the first guess |
| Sticky edges | optional header row and first column | none |
| Cell renderer | `render_cell((row, column))` | `render_item(index)` |

Both use the shared engine of `VirtualList`'s windowed mode, one per axis, and the same DOM bridge (one passive scroll listener and one ResizeObserver on the container, removed on unmount): positions are kept in a Fenwick tree, the mounted range is re-centred only once the viewport comes within half a buffer of its edge (so a scroll re-renders about once every few rows or columns, not once per scroll event), and the scroll canvas keeps its size while a scroll is running. `row_buffer` (default 6) and `column_buffer` (default 4) set how many rows and columns are mounted beyond each edge; `VirtualWrapGrid` has `buffer` (default 4 rows).

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

## Edge fade

`fade: ScrollFade` (`None` by default; `Y`, `X`, `Top`, `Bottom`, `Start`, `End`) puts the matching `dx-scroll-fade-*` class on the grid container, which is the element that scrolls, exactly as on `VirtualList` and `ScrollArea`. The fade is a CSS scroll-driven mask driven by the container's own scroll offset, so it follows the scrollbar whichever cells are mounted.

```rust
VirtualWrapGrid { count: 5_000usize, columns: 4usize, fade: ScrollFade::Y, render_item: .. }
```

- The mask covers everything inside the container: with `render_header` a top fade dims the sticky header once you scroll, so prefer `ScrollFade::Bottom` (or `End` for the inline axis). The grid's own border and background fade too: put them on a wrapper and set `border: none` on the grid.
- It needs `dx-effects.css` linked once at the app root next to the theme (`virtual_grid`'s `component.json` lists it in `globalAssets`). Without it the prop is inert.
- Example: the `fade` variant.

## Styling

Rows are `[role=row]`; cells carry `data-grid-cell="row:column"` (the header's is `"header:column"`), and the sticky header and column carry `data-grid-header` / `data-grid-sticky`. Positions and sizes are inline styles; this component's `style.css` paints borders, the header, the sticky column's background and the focus ring. In `VirtualWrapGrid` each row is a CSS grid of equal tracks: set `column-gap` and `padding-block-end` on `[role=row]` for the gaps (they are part of the measured row).

## Photo wall demo

The `photo_wall` demo is a 300 x 300 wall of 90,000 photos on the two-axis `VirtualGrid`, scrolling on both axes, with a few hundred cells in the DOM wherever you are (the counter under the wall, "N of 90,000 photos rendered", shows how many).

The photos have mixed aspect ratios (portrait 2:3 and 3:4, square, landscape 4:3, 3:2 and 16:9, and 3:1 panoramas), but the grid does not care: its structure, cells aligned in rows and columns of a fixed size, never depends on content. That is why both axes window independently and the keyboard model stays the plain grid one. The shape of a photo is a presentation concern inside its cell: each photo is an `<img>` filling a cell-sized mat, and CSS `object-fit` decides the rest. By default (`contain`) the whole photo is visible, letterboxed on the mat; the switch above the wall changes it to `cover` (a tight wall, photos cropped). Each photo is a single `<img src="data:image/svg+xml,...">` generated from its index, one DOM node per cell instead of the dozen of an inline `<svg>`, and nothing is fetched.

- **Pan** by scrolling or a trackpad on both axes, or by dragging the wall with a mouse or pen (touch keeps its native scroll). A press that does not move stays a click, which opens the photo.
- **Minimap**: the small square shows the whole wall and an outlined box for what the grid shows. Press or drag it to move the viewport there (it sets the grid container's scroll position, which the grid already follows).
- **Jump to random photo** scrolls a random cell to the middle of the wall and focuses it.
- **Preview**: clicking a cell, or `Enter`/`Space` on the focused one, opens the photo in a `Dialog` at its real aspect ratio. The arrow keys, `Home`/`End` and `PageUp`/`PageDown` are the grid's own.

One script installed with `JsListeners` reads the scroll position and the mounted cell count (passive scroll listener, MutationObserver, ResizeObserver, all removed with the demo).
