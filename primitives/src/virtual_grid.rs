//! Defines [`VirtualGrid`] (rows AND columns windowed) and [`VirtualWrapGrid`] (items flowing
//! into a fixed number of columns, rows windowed): grids too large to mount whole.
//!
//! # Two layouts
//!
//! - [`VirtualGrid`] is a data grid: `row_count x column_count` cells, every row
//!   `row_height(row)` px tall and every column `column_width(column)` px wide (both known up
//!   front, as in a spreadsheet). Only the cells in view plus `row_buffer` rows and
//!   `column_buffer` columns each side are mounted, so a 10,000 x 50 grid keeps a few hundred
//!   cells in the DOM. An optional header row (`render_header`) sticks to the top and an
//!   optional first column (`sticky_first_column`) sticks to the inline start; both stay out of
//!   the windowing of their axis.
//! - [`VirtualWrapGrid`] is a gallery: `count` items in rows of `columns` equal tracks (CSS grid),
//!   only the rows axis windowed. Rows are measured as they render (an image tile's height
//!   follows its width), with `estimate_row_height` as the first guess.
//!
//! Both are built on the shared engine (`primitives/src/virtual/`), and share its DOM bridge with
//! `VirtualList` (one passive scroll listener, one ResizeObserver, a render only when the window
//! moves) and its pinning of the focused item: each axis is the same
//! [`Engine`](crate::r#virtual) `VirtualList` uses (a Fenwick tree of item positions, a mounted
//! range re-centred only once the viewport comes within `buffer / 2` items of its edge, the
//! canvas size frozen while a scroll runs), so a scroll re-renders Rust about once per few rows
//! or columns, not once per scroll event.
//!
//! # Keyboard and focus
//!
//! Both follow the WAI-ARIA grid pattern: the container is `role="grid"` (with `aria-rowcount`
//! and `aria-colcount` of the WHOLE grid, and every mounted row and cell carries its
//! `aria-rowindex` / `aria-colindex`), exactly one cell is in the tab order (`tabindex="0"`,
//! the "active" cell), and the arrow keys, `Home`/`End` (row start/end), `Ctrl+Home`/`Ctrl+End`
//! (first/last cell) and `PageUp`/`PageDown` (a viewport of rows) move it. Navigation stops at
//! the edges. The horizontal arrows follow the reading direction (`dir`, else the nearest
//! [`DirectionProvider`](crate::direction::DirectionProvider)): in RTL `ArrowLeft` moves to the
//! inline-end neighbour. **The active cell is always rendered**, even when it is far outside
//! the window (after `Ctrl+End`, or when the reader scrolls away from it), so focus is never
//! lost to an unmount and Tab always finds the grid's stop; moving it scrolls it into view
//! past the sticky header and column.
//!
//! # What windowing costs
//!
//! Cells outside the window do not exist: find-in-page cannot reach them, the accessibility
//! tree holds only the window (the row and column indices say where it is), and the server
//! renders only the active cell. For a grid of a few thousand cells, a plain CSS grid with
//! `content-visibility: auto` on its rows is the cheaper answer (survey
//! `dev-docs/research/virtualization-survey-2026-10-04.md`).

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::direction::{use_direction, Direction, HorizontalNav};
use crate::r#virtual::{
    measured_px, use_virtual_bridge, with_pinned, Bridge, Engine, GridEngine, GridKey, GridShape,
    ScrollReport, SizeEstimates,
};
pub use crate::scroll_area::ScrollFade;
use crate::{merge_attributes, use_unique_id};

/// The row height [`VirtualGrid`] uses without `row_height`, in px.
pub const DEFAULT_ROW_HEIGHT: u32 = 36;
/// The column width [`VirtualGrid`] uses without `column_width`, in px.
pub const DEFAULT_COLUMN_WIDTH: u32 = 120;

/// The props for the [`VirtualGrid`] component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualGridProps {
    /// The number of data rows (the header row, if any, is not counted).
    pub row_count: ReadSignal<usize>,
    /// The number of columns.
    pub column_count: ReadSignal<usize>,
    /// The height of a data row by index, in px. Defaults to [`DEFAULT_ROW_HEIGHT`] (36) for
    /// every row. Rows are not measured: a cell taller than its row overflows it.
    pub row_height: Option<Callback<usize, u32>>,
    /// The width of a column by index, in px. Defaults to [`DEFAULT_COLUMN_WIDTH`] (120) for
    /// every column.
    pub column_width: Option<Callback<usize, u32>>,
    /// Renders the content of the cell at `(row, column)`.
    pub render_cell: Callback<(usize, usize), Element>,
    /// Renders the header of a column. When given, a header row of `role="columnheader"` cells
    /// sticks to the top of the grid while it scrolls.
    pub render_header: Option<Callback<usize, Element>>,
    /// The height of the header row, in px. Ignored without `render_header`.
    #[props(default = 40)]
    pub header_height: u32,
    /// Keep the first column at the inline start while the grid scrolls horizontally. Its
    /// cells are `role="rowheader"`.
    #[props(default)]
    pub sticky_first_column: ReadSignal<bool>,
    /// How many rows to mount beyond each edge of the viewport (the window is re-centred once
    /// fewer than half of them remain).
    #[props(default = ReadSignal::new(Signal::new(6)))]
    pub row_buffer: ReadSignal<usize>,
    /// How many columns to mount beyond each edge of the viewport.
    #[props(default = ReadSignal::new(Signal::new(4)))]
    pub column_buffer: ReadSignal<usize>,
    /// The reading direction for the horizontal arrow keys. Defaults to the nearest
    /// `DirectionProvider`, else left-to-right. Set the matching `dir` attribute (or inherit
    /// one) so the layout flips with it.
    pub dir: Option<Direction>,
    /// Called with `(row, column)` when the active cell changes (by keyboard, click or focus).
    pub on_active_change: Option<Callback<(usize, usize)>>,
    /// A scroll-edge fade on the container, which is the element that scrolls (the
    /// `dx-scroll-fade-*` classes of `dx-effects.css`; see [`ScrollFade`]). Defaults to
    /// [`ScrollFade::None`]. It follows the container's own scroll offset, whatever cells are
    /// mounted. The fade is a mask, so it also fades a sticky header (prefer
    /// [`ScrollFade::Bottom`] / [`ScrollFade::End`] with `render_header`) and a border or
    /// background on the container: put those on a wrapper.
    #[props(default)]
    pub fade: ScrollFade,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # VirtualGrid
///
/// A data grid that mounts only the cells in view: rows AND columns are windowed. See the
/// module documentation for the layouts, the keyboard model and the trade-offs.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::virtual_grid::VirtualGrid;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         VirtualGrid {
///             row_count: 10_000usize,
///             column_count: 50usize,
///             column_width: |column: usize| if column == 0 { 80 } else { 120 },
///             render_header: |column: usize| rsx! { "Column {column}" },
///             render_cell: |(row, column): (usize, usize)| rsx! { "R{row} C{column}" },
///             sticky_first_column: true,
///             style: "height: 24rem; overflow: auto;",
///             "aria-label": "Measurements",
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The container is a `div` the caller makes a scroll container: give it a height (or
/// `max-height`) and `overflow: auto`. It owns `id` and `role`; every other attribute is
/// spread onto it. Rows are `[role=row]`, cells `[role=gridcell]` / `[role=rowheader]` /
/// `[role=columnheader]`, each with `data-grid-cell="row:column"` (the header row's is
/// `"header:column"`); the sticky ones carry `data-grid-sticky`. Positions and sizes are
/// inline styles; give the sticky header and column a background.
#[component]
pub fn VirtualGrid(props: VirtualGridProps) -> Element {
    let VirtualGridProps {
        row_count,
        column_count,
        row_height,
        column_width,
        render_cell,
        render_header,
        header_height,
        sticky_first_column,
        row_buffer,
        column_buffer,
        dir,
        on_active_change,
        fade,
        attributes,
    } = props;
    let attributes = merge_attributes(vec![attributes, fade.attributes()]);

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    // The shared DOM bridge (`primitives/src/virtual/bridge.rs`); both axes read each report.
    let bridge = use_virtual_bridge(
        container_id,
        GridEngine::new,
        |bridge: Bridge<GridEngine>, report: ScrollReport| {
            let mut engine = bridge.engine();
            engine.with_mut(|engine| engine.scroll(report));
            bridge.publish();
        },
    );
    let engine = bridge.engine();
    let active = use_signal(|| (0usize, 0usize));
    let focus_request = use_signal(|| 0u64);

    let row_estimates = use_memo(move || {
        let height = move |row: usize| row_height.map_or(DEFAULT_ROW_HEIGHT, |f| f(row));
        SizeEstimates::evaluate(row_count(), Some(&height))
    });
    // The sticky column is not part of the scrolling axis: the columns engine indexes the
    // columns after it.
    let column_geometry = use_memo(move || {
        let width = move |column: usize| column_width.map_or(DEFAULT_COLUMN_WIDTH, |f| f(column));
        let columns = column_count();
        let sticky = usize::from(sticky_first_column() && columns > 0);
        let sticky_width = if sticky == 1 { width(0) } else { 0 };
        let scrolling = move |i: usize| width(i + sticky);
        (
            SizeEstimates::evaluate(columns - sticky, Some(&scrolling)),
            sticky,
            sticky_width,
        )
    });
    let header = if render_header.is_some() {
        header_height
    } else {
        0
    };

    let (sticky_width, sticky) = {
        let geometry = column_geometry.read();
        (geometry.2, geometry.1)
    };
    use_focus_active_cell(container_id, active, focus_request, header, sticky_width);

    let shape = GridShape::full(row_count(), column_count());
    let tab_stop = shape.clamp(active());
    let set_active = move |cell: (usize, usize)| {
        let mut active = active;
        if *active.peek() != cell {
            active.set(cell);
            if let Some(callback) = on_active_change {
                callback.call(cell);
            }
        }
    };

    let onkeydown = move |event: KeyboardEvent| {
        let Some(key) = grid_key(&event.data(), direction) else {
            return;
        };
        event.prevent_default();
        let page = engine.read().page_rows();
        let from = *active.peek();
        set_active(shape.navigate(from, key, page));
        let mut focus_request = focus_request;
        focus_request += 1;
    };

    // Subscribe: this render runs again whenever the bridge publishes a new window.
    bridge.track();
    let plan = {
        let rows = row_estimates.read();
        let geometry = column_geometry.read();
        let mut engine = engine;
        engine.with_mut(|engine| {
            engine.sync(&rows, &geometry.0, geometry.1, geometry.2, header);
            engine.plan(row_buffer(), column_buffer(), tab_stop)
        })
    };

    let header_rows = usize::from(render_header.is_some());
    let width = sticky_width + plan.pane_width;
    let columns = plan.columns;

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows() + header_rows,
        "aria-colcount": column_count(),
        "data-virtual-grid": "grid",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    // A cell: the sticky one sits in flow at the row's inline start, the others are placed
    // absolutely in the scrolling pane.
    let cell = move |row: usize, column: usize, start: u32, size: u32| {
        let is_sticky = column < sticky;
        let style = if is_sticky {
            format!("position: sticky; inset-inline-start: 0; z-index: 1; flex: none; box-sizing: border-box; width: {size}px; height: 100%;")
        } else {
            format!("position: absolute; inset-block: 0; inset-inline-start: {start}px; box-sizing: border-box; width: {size}px;")
        };
        let focusable = tab_stop == Some((row, column));
        rsx! {
            div {
                key: "{column}",
                role: if is_sticky { "rowheader" } else { "gridcell" },
                "aria-colindex": column + 1,
                "data-grid-cell": "{row}:{column}",
                "data-grid-sticky": is_sticky.then_some("true"),
                tabindex: if focusable { "0" } else { "-1" },
                style,
                onfocus: move |_| set_active((row, column)),
                {render_cell((row, column))}
            }
        }
    };
    let sticky_column_width = sticky_width;
    let pane_width = plan.pane_width;
    let header_cells = columns.clone();

    rsx! {
        div { onkeydown, ..merged,
            div { style: "position: relative; width: {width}px;",
                if let Some(render_header) = render_header {
                    div {
                        role: "row",
                        "aria-rowindex": 1,
                        "data-grid-header": "true",
                        style: "position: sticky; top: 0; z-index: 2; display: flex; width: {width}px; height: {header}px;",
                        if sticky == 1 {
                            div {
                                role: "columnheader",
                                "aria-colindex": 1,
                                "data-grid-cell": "header:0",
                                "data-grid-sticky": "true",
                                style: "position: sticky; inset-inline-start: 0; z-index: 1; flex: none; box-sizing: border-box; width: {sticky_column_width}px; height: 100%;",
                                {render_header(0)}
                            }
                        }
                        div { style: "position: relative; flex: none; width: {pane_width}px;",
                            for column in header_cells {
                                div {
                                    key: "{column.index()}",
                                    role: "columnheader",
                                    "aria-colindex": column.index() + 1,
                                    "data-grid-cell": "header:{column.index()}",
                                    style: "position: absolute; inset-block: 0; inset-inline-start: {column.start()}px; box-sizing: border-box; width: {column.size()}px;",
                                    {render_header(column.index())}
                                }
                            }
                        }
                    }
                }
                div { style: "position: relative; width: {width}px; height: {plan.body_height}px;",
                    for row in plan.rows {
                        div {
                            key: "{row.index()}",
                            role: "row",
                            "aria-rowindex": row.index() + 1 + header_rows,
                            style: "position: absolute; inset-inline-start: 0; top: {row.start()}px; display: flex; width: {width}px; height: {row.size()}px;",
                            if sticky == 1 {
                                {cell(row.index(), 0, 0, sticky_column_width)}
                            }
                            div { style: "position: relative; flex: none; width: {pane_width}px;",
                                // An iterator of keyed elements, not a `for` around an
                                // expression: the key must be on each list item's root, or a
                                // window move re-creates every cell (and drops focus).
                                {columns.iter().map(|column| cell(row.index(), column.index(), column.start(), column.size()))}
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The props for the [`VirtualWrapGrid`] component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualWrapGridProps {
    /// The number of items.
    pub count: ReadSignal<usize>,
    /// How many items each row holds (equal-width tracks). At least 1. Changing it forgets the
    /// measured row heights (row `i` then holds other items).
    pub columns: ReadSignal<usize>,
    /// Estimates the height of a row by row index, in px, until it is measured. Without it the
    /// grid estimates from the average of the rows measured so far (100px before the first).
    pub estimate_row_height: Option<Callback<usize, u32>>,
    /// Renders the item at an absolute index.
    pub render_item: Callback<usize, Element>,
    /// How many rows to mount beyond each edge of the viewport (the window is re-centred once
    /// fewer than half of them remain).
    #[props(default = ReadSignal::new(Signal::new(4)))]
    pub buffer: ReadSignal<usize>,
    /// The reading direction for the horizontal arrow keys. Defaults to the nearest
    /// `DirectionProvider`, else left-to-right.
    pub dir: Option<Direction>,
    /// Called with the item index when the active item changes (by keyboard, click or focus).
    pub on_active_change: Option<Callback<usize>>,
    /// A scroll-edge fade on the container (the `dx-scroll-fade-*` classes of
    /// `dx-effects.css`; see [`ScrollFade`]). Defaults to [`ScrollFade::None`]. It follows the
    /// container's own scroll offset, whatever rows are mounted; a border or background on the
    /// container fades with it, so put those on a wrapper.
    #[props(default)]
    pub fade: ScrollFade,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # VirtualWrapGrid
///
/// A gallery grid: `count` items flow into rows of `columns`, and only the rows in view (plus a
/// buffer) are mounted. Rows are measured as they render. Keyboard and focus follow the grid
/// pattern, as for [`VirtualGrid`] (see the module documentation).
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::virtual_grid::VirtualWrapGrid;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         VirtualWrapGrid {
///             count: 5_000usize,
///             columns: 4usize,
///             estimate_row_height: |_row: usize| 160,
///             render_item: |index: usize| rsx! { figure { "Photo {index}" } },
///             style: "height: 30rem; overflow-y: auto;",
///             "aria-label": "Photos",
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The container is the scroll container (give it a height and `overflow-y: auto`); it owns
/// `id` and `role`. Each `[role=row]` is a CSS grid of `columns` equal tracks; set
/// `column-gap` and a `padding-block-end` on it for the gaps (they are part of the measured
/// row). Items are `[role=gridcell]` with `data-grid-cell="row:column"` and `data-grid-index`.
#[component]
pub fn VirtualWrapGrid(props: VirtualWrapGridProps) -> Element {
    let VirtualWrapGridProps {
        count,
        columns,
        estimate_row_height,
        render_item,
        buffer,
        dir,
        on_active_change,
        fade,
        attributes,
    } = props;
    let attributes = merge_attributes(vec![attributes, fade.attributes()]);

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    // The shared DOM bridge with the one-axis flow `VirtualList` uses: the rows axis, with the
    // anchoring correction for rows measured above the viewport.
    let bridge = use_virtual_bridge(container_id, Engine::new, Bridge::scroll_axis);
    let engine = bridge.engine();
    let laid_out_for = use_hook(|| CopyValue::new(0usize));
    // The active item, as an index: it keeps its item when the column count changes.
    let active = use_signal(|| 0usize);
    let focus_request = use_signal(|| 0u64);

    let estimates = use_memo(move || {
        let shape = GridShape::wrapping(count(), columns());
        let estimate = estimate_row_height.map(|f| move |row: usize| f(row));
        SizeEstimates::evaluate(
            shape.rows(),
            estimate.as_ref().map(|f| f as &dyn Fn(usize) -> u32),
        )
    });

    let active_cell = use_active_cell(active, columns);
    use_focus_active_cell(container_id, active_cell, focus_request, 0, 0);

    let columns_now = columns().max(1);
    let shape = GridShape::wrapping(count(), columns_now);
    let tab_stop = shape.clamp(index_to_cell(active(), columns_now));
    let set_active = move |index: usize| {
        let mut active = active;
        if *active.peek() != index {
            active.set(index);
            if let Some(callback) = on_active_change {
                callback.call(index);
            }
        }
    };

    let onkeydown = move |event: KeyboardEvent| {
        let Some(key) = grid_key(&event.data(), direction) else {
            return;
        };
        event.prevent_default();
        let page = engine.read().visible_len();
        let from = index_to_cell(*active.peek(), columns_now);
        let (row, column) = shape.navigate(from, key, page);
        set_active(row * columns_now + column);
        let mut focus_request = focus_request;
        focus_request += 1;
    };

    let onresize = move |row: usize| {
        move |event: Event<ResizeData>| {
            let size = event.data().get_border_box_size().unwrap_or_default();
            bridge.measure(row, measured_px(size.height));
        }
    };

    bridge.track();
    let rows = {
        let estimates = estimates.read();
        let (mut engine, mut laid_out_for) = (engine, laid_out_for);
        engine.with_mut(|engine| {
            if *laid_out_for.peek() != columns_now {
                laid_out_for.set(columns_now);
                engine.forget_measurements();
            }
            engine.sync(&estimates);
            let plan = engine.plan(buffer());
            let pinned = tab_stop.and_then(|(row, _)| engine.item(row));
            (with_pinned(plan.items, pinned), plan.canvas_height)
        })
    };
    let (rows, height) = rows;

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows(),
        "aria-colcount": columns_now,
        "data-virtual-grid": "wrap",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    rsx! {
        div { onkeydown, ..merged,
            div { style: "position: relative; height: {height}px;",
                for row in rows {
                    div {
                        key: "{row.index()}",
                        role: "row",
                        "aria-rowindex": row.index() + 1,
                        style: "position: absolute; inset-inline: 0; top: {row.start()}px; box-sizing: border-box; display: grid; grid-template-columns: repeat({columns_now}, minmax(0, 1fr));",
                        onresize: onresize(row.index()),
                        {(0..shape.row_len(row.index())).map(|column| {
                            let index = row.index() * columns_now + column;
                            let focusable = tab_stop == Some((row.index(), column));
                            rsx! {
                                div {
                                    key: "{column}",
                                    role: "gridcell",
                                    "aria-colindex": column + 1,
                                    "data-grid-cell": "{row.index()}:{column}",
                                    "data-grid-index": index,
                                    tabindex: if focusable { "0" } else { "-1" },
                                    style: "min-width: 0;",
                                    onfocus: move |_| set_active(index),
                                    {render_item(index)}
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}

/// `index` as `(row, column)` in rows of `columns`.
fn index_to_cell(index: usize, columns: usize) -> (usize, usize) {
    let columns = columns.max(1);
    (index / columns, index % columns)
}

/// The active item of a wrapping grid as a `(row, column)` signal, for the shared focus hook.
fn use_active_cell(active: Signal<usize>, columns: ReadSignal<usize>) -> Memo<(usize, usize)> {
    use_memo(move || index_to_cell(active(), columns()))
}

/// Maps a key press to a grid move, or `None` for a key the grid does not handle (which then
/// keeps its default action). The horizontal arrows go through `direction.rs`, so they mirror in
/// RTL.
fn grid_key(event: &KeyboardData, direction: Direction) -> Option<GridKey> {
    let key = event.key();
    let ctrl = event.modifiers().ctrl() || event.modifiers().meta();
    if let Some(horizontal) = direction.resolve_horizontal(&key) {
        return Some(match horizontal {
            HorizontalNav::Prev => GridKey::Prev,
            HorizontalNav::Next => GridKey::Next,
        });
    }
    Some(match key {
        Key::ArrowUp => GridKey::Up,
        Key::ArrowDown => GridKey::Down,
        Key::Home if ctrl => GridKey::First,
        Key::End if ctrl => GridKey::Last,
        Key::Home => GridKey::RowStart,
        Key::End => GridKey::RowEnd,
        Key::PageUp => GridKey::PageUp,
        Key::PageDown => GridKey::PageDown,
        _ => return None,
    })
}

/// After a keyboard move has rendered, focus the active cell and scroll it into view, clear of
/// the sticky header (`header` px) and sticky column (`sticky_width` px).
fn use_focus_active_cell(
    container_id: Signal<String>,
    active: impl Readable<Target = (usize, usize)> + Copy + 'static,
    focus_request: Signal<u64>,
    header: u32,
    sticky_width: u32,
) {
    use_effect(move || {
        if focus_request() == 0 {
            return;
        }
        let (row, column) = *active.peek();
        let eval = document::eval(FOCUS_CELL_JS);
        let _ = eval.send((
            container_id.peek().clone(),
            format!("{row}:{column}"),
            header,
            sticky_width,
        ));
    });
}

/// Focuses the active cell once it is in the DOM (it is pinned, so the render that made it
/// active mounts it) and scrolls the container just enough to show it beside the sticky edges.
const FOCUS_CELL_JS: &str = r#"
    const [id, key, header, stickyWidth] = await dioxus.recv();
    let tries = 0;
    const go = () => {
        const grid = document.getElementById(id);
        if (!grid) return;
        const cell = grid.querySelector('[data-grid-cell="' + key + '"]');
        if (!cell) {
            if (tries++ < 10) requestAnimationFrame(go);
            return;
        }
        // focus-ok: the grid pattern's roving tab stop; the key press already holds focus inside the grid
        cell.focus({ preventScroll: true });
        const box = grid.getBoundingClientRect();
        const rect = cell.getBoundingClientRect();
        const top = box.top + grid.clientTop + header;
        const bottom = box.top + grid.clientTop + grid.clientHeight;
        if (rect.top < top) grid.scrollTop -= top - rect.top;
        else if (rect.bottom > bottom) grid.scrollTop += rect.bottom - bottom;
        if (cell.hasAttribute("data-grid-sticky")) return;
        const rtl = getComputedStyle(grid).direction === "rtl";
        const left = box.left + grid.clientLeft + (rtl ? 0 : stickyWidth);
        const right = box.left + grid.clientLeft + grid.clientWidth - (rtl ? stickyWidth : 0);
        if (rect.left < left) grid.scrollLeft -= left - rect.left;
        else if (rect.right > right) grid.scrollLeft += rect.right - right;
    };
    go();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_cell_round_trip() {
        assert_eq!(index_to_cell(0, 4), (0, 0));
        assert_eq!(index_to_cell(9, 4), (2, 1));
        assert_eq!(index_to_cell(9, 0), (9, 0));
    }
}
