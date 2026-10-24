The VirtualGrid component keeps a very large grid cheap by mounting only the cells in view. It comes in three layouts: `VirtualGrid`, a data grid whose rows AND columns are windowed (with an optional header row and pinned columns at both inline edges), `VirtualWrapGrid`, a gallery whose items wrap into rows of a fixed column count (only the rows are windowed), and `VirtualJustifiedGrid`, a photo wall of justified rows (every photo uncropped at its own aspect ratio, rows filling the width edge to edge).

## Component Structure

```rust
// A data grid: 10,000 rows x 50 columns, a few hundred cells in the DOM.
VirtualGrid {
    row_count: 10_000usize,
    column_count: 50usize,
    // Optional: px per row / column (default 36 / 120).
    row_height: |_row: usize| 36,
    column_width: |column: usize| if column == 0 { 80 } else { 112 },
    // Optional: a header row, pinned to the top (`pin_header: false` lets it scroll away), and
    // columns pinned at the inline start and end (see "Pinning rows and columns").
    render_header: |column: usize| rsx! { "Column {column}" },
    pinned_columns_start: 1usize,
    pinned_columns_end: 1usize,
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

## Pinning rows and columns

A data grid pins the header row and columns at both inline edges while the rest of the body scrolls under them. In TanStack Table's terms: `pin_header` is the pinned header row, `pinned_columns_start` is `columnPinning.left` (the inline start, so the right in RTL) and `pinned_columns_end` is `columnPinning.right`.

```rust
VirtualGrid {
    row_count: 10_000usize,
    column_count: 50usize,
    column_width: |column: usize| if column < 2 { 80 } else { 112 },
    render_header: |column: usize| rsx! { "Column {column}" },
    render_cell: |(row, column): (usize, usize)| rsx! { "{row}, {column}" },
    // The header stays at the top (the default), two columns stay at the start, one at the end.
    pin_header: true,
    pinned_columns_start: 2usize,
    pinned_columns_end: 1usize,
    "aria-label": "Readings",
    style: "height: 24rem;",
}
```

| Prop | Default | Pins |
|---|---|---|
| `pin_header: bool` | `true` | The header row (`render_header`) at the top. `false` lets it scroll away with the body. |
| `pinned_columns_start: usize` | `0` | The first N columns at the inline start. They stack in column order, each at the cumulative width of the pinned columns before it. |
| `pinned_columns_end: usize` | `0` | The last N columns at the inline end. They stack from the end edge inward. |

What pinning means:

- **Windowing.** Pinned columns are always mounted and are not part of the column window: the windowed body renders the scrolling columns between them, and a pinned cell is never rendered twice. The two counts are clamped to `column_count`, the start side first.
- **Appearance.** A pinned cell is opaque, so the cells scrolling under it never show through, and takes the row's hover and selected fill over its own background. The pinned cell next to the scrolling columns (the last start column, the first end column) has the separating edge.
- **Keyboard.** Every move scrolls the active cell into view clear of the pinned header and of the pinned columns on both sides, so `End`, `Home`, the arrows and `PageUp`/`PageDown` never leave it covered. A move to a pinned cell scrolls only vertically.
- **Right to left.** The layout uses logical properties: the start columns sit at the right, the end columns at the left, and the arrows mirror as described under "Keyboard and focus".
- **ARIA.** With `pinned_columns_start >= 1` the first column's cells are `role="rowheader"`; no other pinned column is a row header, and with no start column pinned the first column stays `gridcell`. Pinning changes no `aria-colindex`: every cell keeps its place in the whole grid.
- **Styling.** Pinned cells carry `data-grid-pinned="start|end"` and the innermost one of each side `data-grid-pin-edge="true"`.

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
        // Empty alt (a loading photo paints its alt text in the tile), the name on a wrapper.
        div { role: "img", "aria-label": "Photo {index}", style: "width: 100%; height: 100%;",
            img { src: photo_url(index), alt: "", style: "width: 100%; height: 100%;" }
        }
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
| Pinned edges | optional header row (`pin_header`), `pinned_columns_start` and `pinned_columns_end` columns | none |
| Cell renderer | `render_cell((row, column))` | `render_item(index)` |

Both use the shared engine of `VirtualList`'s windowed mode, one per axis (see "The shared windowing engine" below); `VirtualWrapGrid` measures its rows as they render.

In the 10,000 x 50 demo about 300 of the 500,000 cells are in the DOM at any time, wherever you scroll.

## Keyboard and focus

The grid follows the WAI-ARIA grid pattern. Exactly one cell is in the tab order (the active cell, `tabindex="0"`); click a cell or Tab into the grid, then:

| Key | Moves to |
|---|---|
| Arrow keys | the neighbouring cell (stops at the edges) |
| `Home` / `End` | the first / last cell of the row |
| `Ctrl+Home` / `Ctrl+End` | the first / last cell of the grid |
| `PageUp` / `PageDown` | a viewport of rows up / down, same column |

**The active cell is always rendered**, even when it is far outside the window (after `Ctrl+End`, or when you scroll away from it): a windowed grid that unmounted it would lose focus and its only tab stop (see "Focus stays mounted" below). Each move scrolls the cell into view clear of the pinned header and the pinned columns on both sides. `on_active_change` reports the active cell.

In right-to-left layouts the grid mirrors: the start-pinned columns sit at the right, the end-pinned ones at the left, and `ArrowLeft` moves to the next cell (inline end). The direction comes from `dir`, else the nearest `DirectionProvider`; the layout itself follows the inherited `dir` attribute through logical CSS properties.

## Accessibility

The container is `role="grid"` with `aria-rowcount` and `aria-colcount` of the whole grid, and every mounted row and cell carries `aria-rowindex` / `aria-colindex`, so a screen reader announces where a cell is even though only a window exists. Header cells are `role="columnheader"`; with at least one start-pinned column (`pinned_columns_start >= 1`) the first column's cells are `role="rowheader"` (no other pinned column is).

## Sorting, selection and controls in cells

Three hooks let the grid host a data table. All are optional.

- `column_sort: Callback<usize, Option<SortDirection>>` sets `aria-sort` on a header: return `Some(Ascending|Descending)` for the sorted column and `None` for the rest (the APG sortable-table pattern puts it on that one header only). The grid owns the `columnheader`, so this is the only way to set it.
- `row_selected: Callback<String, bool>` is asked, for every mounted row, whether the row with that `row_key` (the index as a string without one) is selected. Selected rows get `aria-selected="true"` and `data-state="selected"` (painted `--dx-muted` by the sheet, pinned columns included); the others `aria-selected="false"`. It is keyed by identity, so the selection follows the row through a sort and a window move.
- **A cell may hold one control** (checkbox, switch, button, link): the APG grid's "focus the widget" model. The control, not its cell, is then the grid's single tab stop, the arrows still move between cells (focus lands on the next cell's control, or on the cell when it has none), and `Space` / `Enter` act on the control natively. A cell with a text field, a select or several controls needs the APG's `Enter`/`F2` interaction mode, which the grid does not provide: keep those out of the grid, or edit in a dialog.

## What windowing costs

A `VirtualGrid` cell taller than its row overflows it (rows are not measured). A grid of up to `auto_threshold` cells (1,000) is not windowed at all under the default `Auto`: every cell stays in the DOM, columns in flow, and the browser skips the rows that are off-screen in chunks of 20 (`data-virtualize="content-visibility"`, the `content_visibility` variant); the keyboard model, the pinned header and columns, sorting, selection and controls in cells are the same in both modes. Pass `virtualize: VirtualizeMode::Windowed` (or `ContentVisibility`) to choose. Find-in-page, the accessibility tree and the server are covered under "Find-in-page and the windowed DOM" below.

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

<!-- shared:virtual-engine:engine -->
## The shared windowing engine

`VirtualList` (windowed mode), `VirtualGrid`, `VirtualWrapGrid` and `VirtualJustifiedGrid` mount only what is in view, on one engine and one DOM bridge (`primitives/src/virtual/`).

- **Overscan.** Items beyond each edge of the viewport are mounted too, so a scroll never shows a blank edge: `buffer` rows for `VirtualList` (default 8) and `VirtualWrapGrid` (default 4), `row_buffer` (default 6) and `column_buffer` (default 4) for `VirtualGrid`, and `overscan` px (default 320) for `VirtualJustifiedGrid`. The window is re-centred only once fewer than half of that margin remains, so between half and all of it is always ready beyond each edge, and a steady scroll re-renders about once per half margin rather than once per scroll event.
- **Positions.** Sizes are kept in a Fenwick tree: measuring an item and finding the item at a pixel are `O(log N)`, and nothing is copied per measurement. Where items are measured (a list's rows, a wrapping grid's rows) an estimate is used until then; an item that resizes above the viewport adjusts the scroll position so content does not jump, and the scroll canvas keeps its size while a scroll is running.
- **Listeners.** One passive `scroll` listener and one `ResizeObserver` on the container, plus a `resize` listener on the window, all removed when the component unmounts. Scrolling costs Rust nothing unless the window moves.
<!-- /shared:virtual-engine:engine -->

<!-- shared:virtual-engine:focus -->
## Focus stays mounted

The item that last received focus (a link or button inside a row, or the active cell or tile of a grid) stays mounted when you scroll away from it: it is drawn at its own position outside the window (`VirtualList` marks it `data-virtual-pinned`), so focus never falls to the page and Tab or the arrow keys continue from where they were. Moving focus elsewhere releases it, and when a scroll brings the window back to it the same DOM node rejoins the flow without remounting. (`VirtualList` in content-visibility mode keeps every row in the DOM, so there is nothing to pin.)
<!-- /shared:virtual-engine:focus -->

<!-- shared:virtual-engine:identity -->
## Identity: `item_key`

A renderer is a count and a function of the index, so on its own it cannot tell who is at an index. After a sort, a filter or a reorder the DOM node, the pinned focus row, the grid's active cell and the measured height of a row would all stay on the slot while the item moved on: focus on a different item, a measured 130 px applied to a 34 px header. The optional `item_key` prop (`VirtualGrid` calls it `row_key`: rows are what it identifies) says who is at an index: a string that is unique and stays with the item. Everything that must survive a change of the view is then keyed by it: the row's DOM node and the state inside it, the row that stays mounted while it holds focus, the grid's active cell, and the engine's measured sizes (a wrapping grid keys a row by the items it holds). Without it the key is the index (a row is its slot). The key is read while rendering, so signals it reads re-render the component; the browser still drops focus from a node it has to move, so a reorder that keeps the focused node in place (the common case when one row moves) keeps focus. Concretely, in a task list with a checkbox in every row: without `item_key` the checkbox state belongs to the slot, so ticking row 3 and then reversing the list leaves row 3 ticked while a different task now sits there (focus stays on the slot the same way); with `item_key` the tick, the focus and the measured height follow the task to its new place.
<!-- /shared:virtual-engine:identity -->

<!-- shared:virtual-engine:scroll-chaining -->
## Scroll chaining

None of the virtual components traps the page's scroll: at the top or bottom edge of the container a wheel or touch scroll continues on the page (the block axis keeps `overscroll-behavior: auto`). A container that also scrolls sideways contains only the inline axis (`overscroll-behavior-x: contain`), so a swipe at its edge does not trigger the browser's back and forward navigation.
<!-- /shared:virtual-engine:scroll-chaining -->

<!-- shared:virtual-engine:find-in-page -->
## Find-in-page and the windowed DOM

Items outside the window do not exist: the browser's find-in-page and `#:~:text=` links cannot reach them, the accessibility tree holds only what is mounted (each mounted row or cell announces its position in the whole, with `aria-posinset` or `aria-rowindex`), and the server renders none of them or only the active one. The opt-in `text_value` of a windowed `VirtualList`, `VirtualGrid` or `VirtualWrapGrid` keeps every row's, cell's or item's plain text in the DOM as a `hidden="until-found"` placeholder (one shared layer, `primitives/src/virtual/find.rs`), so find-in-page can match items that are not mounted; the component brings the matched one into view and marks it `data-find-match` (the `find_in_page` variants). `VirtualJustifiedGrid` has no placeholders yet. Where findability matters more than a flat cost, a plain list or CSS grid of a few thousand items with `content-visibility: auto` keeps everything findable while the browser skips the off-screen work (`VirtualList`'s content-visibility mode is exactly that).
<!-- /shared:virtual-engine:find-in-page -->

<!-- shared:virtual-engine:fade -->
## Edge fade

`fade: ScrollFade` (`None` by default; `Y`, `X`, `Both`, `Top`, `Bottom`, `Start`, `End`) puts the matching `dx-scroll-fade-*` class on the container, which is the element that scrolls, exactly as on `ScrollArea`. The fade is a CSS scroll-driven mask driven by the container's own scroll offset, so it follows the scrollbar whichever rows or cells are mounted (a windowed list of 20,000 rows or a grid of 500,000 cells fades exactly like a short one), and a container whose content fits has no fade. `Both` masks all four edges of a container that scrolls on both axes (a `VirtualGrid`, or a justified wall with a `canvas_width`), each edge only while there is content past it.

```rust
VirtualList { count: 20_000usize, fade: ScrollFade::Y, render_item: .. }
VirtualGrid { row_count: 1_000usize, column_count: 30usize, fade: ScrollFade::Both, render_cell: .. }
```

- It needs `dx-effects.css` linked once at the app root next to the theme (the `virtual_list` and `virtual_grid` `component.json` list it in `globalAssets`). Without it the prop is inert. See the `dx-scroll-fade` utility in the ScrollArea docs for the variables that tune depth and reveal distance.
- The mask covers everything inside the container: a border or background on it fades with the content (put those on a wrapper and set `border: none` on the container), and so does a `VirtualGrid`'s pinned header, which a top fade dims once you scroll (prefer `ScrollFade::Bottom` or `End` with `render_header`).
- Examples: the `fade` variants (a windowed list and a content-visibility list; a wrapping gallery and a two-axis data grid).
<!-- /shared:virtual-engine:fade -->

## Styling

Rows are `[role=row]`; cells carry `data-grid-cell="row:column"` (the header's is `"header:column"`), and the header row carries `data-grid-header` while pinned cells carry `data-grid-pinned="start|end"` (and the one beside the scrolling columns `data-grid-pin-edge="true"`). Positions and sizes are inline styles; this component's `style.css` paints borders, the header, the pinned columns' opaque background (with the row hover and selected fill layered over it) and the focus ring. In `VirtualWrapGrid` each row is a CSS grid of equal tracks: set `column-gap` and `padding-block-end` on `[role=row]` for the gaps (they are part of the measured row).

## Photo wall demo

The `photo_wall` demo is a 300 x 300 wall of 90,000 photos on the two-axis `VirtualGrid`, scrolling on both axes, with a few hundred cells in the DOM wherever you are (the counter under the wall, "N of 90,000 photos rendered", shows how many).

The photos have mixed aspect ratios (portrait 2:3 and 3:4, square, landscape 4:3, 3:2 and 16:9, and 3:1 panoramas), but the grid does not care: its structure, cells aligned in rows and columns of a fixed size, never depends on content. That is why both axes window independently and the keyboard model stays the plain grid one. The shape of a photo is a presentation concern inside its cell: each photo is an `<img>` filling a cell-sized mat, and CSS `object-fit` decides the rest. By default (`contain`) the whole photo is visible, letterboxed on the mat; the switch above the wall changes it to `cover` (a tight wall, photos cropped). Each photo is a single `<img src="data:image/svg+xml,...">` generated from its index, one DOM node per cell instead of the dozen of an inline `<svg>`, and nothing is fetched.

- **Pan** by scrolling or a trackpad on both axes, or by dragging the wall with a mouse or pen (touch keeps its native scroll). A press that does not move stays a click, which opens the photo.
- **Minimap**: the small square shows the whole wall and an outlined box for what the grid shows. Press or drag it to move the viewport there (it sets the grid container's scroll position, which the grid already follows).
- **Jump to random photo** scrolls a random cell to the middle of the wall and focuses it.
- **Preview**: clicking a cell, or `Enter`/`Space` on the focused one, opens the photo in a `Dialog` at its real aspect ratio. The arrow keys, `Home`/`End` and `PageUp`/`PageDown` are the grid's own.

One script installed with `JsListeners` reads the scroll position and the mounted cell count (passive scroll listener, MutationObserver, ResizeObserver, all removed with the demo).
