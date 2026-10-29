//! Defines [`VirtualGrid`] (rows AND columns windowed), [`VirtualWrapGrid`] (items flowing
//! into a fixed number of columns, rows windowed) and [`VirtualJustifiedGrid`] (a photo wall of
//! justified rows): grids too large to mount whole.
//!
//! # Layouts
//!
//! - [`VirtualGrid`] is a data grid: `row_count x column_count` cells, every row
//!   `row_height(row)` px tall and every column `column_width(column)` px wide (both known up
//!   front, as in a spreadsheet). Only the cells in view plus `row_buffer` rows and
//!   `column_buffer` columns each side are mounted, so a 10,000 x 50 grid keeps a few hundred
//!   cells in the DOM. An optional header row (`render_header`, pinned to the top unless
//!   `pin_header` is false) and the first `pinned_columns_start` / last `pinned_columns_end`
//!   columns are pinned to the grid's edges while the rest scrolls; pinned columns stay out of
//!   the column windowing, and the module's "Pinning" section lists what that means.
//! - [`VirtualWrapGrid`] is a gallery: `count` items in rows of `columns` equal tracks (CSS grid),
//!   only the rows axis windowed. Rows are measured as they render (an image tile's height
//!   follows its width), with `estimate_row_height` as the first guess.
//! - [`VirtualJustifiedGrid`] is a photo wall: items of mixed aspect ratios packed into justified
//!   rows that fill the width edge to edge, each tile exactly its item's aspect (uncropped). The
//!   structure depends on the content (a row holds as many items as fit), so it has no columns
//!   and its own spatial keyboard model (`ArrowUp`/`ArrowDown` pick the nearest tile by
//!   horizontal centre). Rows fill the grid's width (vertical scrolling), or a fixed
//!   `canvas_width` wider than the grid for a wall that scrolls both ways, windowed by row and,
//!   within a row, by tile.
//!
//! The data grid's model is the default for a reason: its structure never depends on content,
//! so non-uniform content (a photo of any aspect) is a presentation concern inside a uniform cell
//! (`object-fit`), and both axes window independently. Use the justified grid when the gapless,
//! uncropped look matters more than aligned columns.
//!
//! Both are built on the shared engine (`primitives/src/virtual/`), and share its DOM bridge with
//! `VirtualList` (one passive scroll listener, one ResizeObserver, a render only when the window
//! moves) and its pinning of the focused item: each axis is the same
//! `Engine` (`crate::r#virtual`) `VirtualList` uses (a Fenwick tree of item positions, a mounted
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
//! past the pinned header and the pinned columns on both sides.
//!
//! # Pinning rows and columns (row 388)
//!
//! [`VirtualGridProps::pin_header`] (default true) keeps the header row at the top while the body
//! scrolls; false lets it scroll away with the body. [`VirtualGridProps::pinned_columns_start`]
//! and [`VirtualGridProps::pinned_columns_end`] (default 0 each) pin that many columns at the
//! inline start and the inline end; the start columns stack from the edge in column order and the
//! end columns from the opposite edge, each at the cumulative width of the pinned columns
//! between it and its edge. Both sides follow the reading direction through logical properties.
//! A pinned column is always mounted, outside the column window (the windowed body never renders
//! it twice), and is opaque so the cells that scroll under it never show through. The two counts
//! are clamped to the column count (the start side first). The first column's cells are
//! `role="rowheader"` when at least one start column is pinned; no other pinned column is a row
//! header. Pinned cells carry `data-grid-pinned="start|end"`, and the one next to the scrolling
//! pane `data-grid-pin-edge="true"`.
//!
//! # Header sort, row selection and controls in cells
//!
//! [`VirtualGridProps::column_sort`] sets `aria-sort` on a column header,
//! [`VirtualGridProps::row_selected`] marks rows `aria-selected` / `data-state="selected"` by the
//! row's identity (`row_key`), and a cell may hold **one control** (a checkbox, switch, button or
//! link): the APG grid's "focus the widget" model. The control, not its cell, is then the grid's
//! one tab stop; the arrows still move between cells (focus lands on the next cell's control),
//! and `Space` / `Enter` act on the control natively. A cell with a text field, a select or
//! several controls needs the APG's Enter/F2 interaction mode, which the grid does not provide.
//!
//! # Scrolling to an item (row 398)
//!
//! [`use_virtual_scroller`] makes a [`VirtualScroller`]; pass it as `scroller` to any of the three
//! grids and call it from an event handler or an effect. [`VirtualScroller::scroll_to_cell`]
//! (`VirtualGrid`) aligns a cell on both axes to a [`ScrollAlign`] (`Start`, `Center`, `End`, or
//! `Nearest`: nothing if it is fully in view), clear of the pinned header and columns, and a
//! pinned column moves only the rows. [`VirtualScroller::scroll_to_index`] reads the index as a
//! row for `VirtualGrid`, as an item for `VirtualWrapGrid` (its row scrolls into place) and as a
//! tile for `VirtualJustifiedGrid` (vertical only). Windowed, the target comes from the engines'
//! measured-or-estimated positions, so a cell far outside the window is reachable; in
//! content-visibility mode the browser scrolls to it and the alignment is corrected for a few
//! frames. An index past the end goes to the last item; a call before the grid has mounted is kept
//! and runs then (see [`VirtualScroller`]).
//!
//! # Modes, and what windowing costs
//!
//! [`VirtualGrid`] and [`VirtualWrapGrid`] take the same `virtualize` / `auto_threshold` props as
//! `VirtualList` ([`VirtualizeMode`], counted in cells). Windowed, cells outside the window do not
//! exist: the accessibility tree holds only the window (the row and column indices say where it
//! is), the server renders only the active cell, and find-in-page reaches them only through the
//! opt-in `text_value` placeholders (`virtual/find.rs`, shared with `VirtualList`). In
//! content-visibility mode (the `Auto` default up to 1,000 cells) every cell is mounted, columns
//! in flow, and the browser skips off-screen rows in chunks of 20 (`content-visibility: auto`):
//! no engine and no scroll listener, the same keyboard model (every cell exists, so nothing
//! needs pinning), and find-in-page, selection and the accessibility tree reach every cell.
//! [`VirtualJustifiedGrid`] is always windowed and has no placeholders.

use crate::state_attrs;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

pub use crate::data_view::SortDirection;
use crate::direction::{use_direction, Direction, HorizontalNav};
use crate::r#virtual::{
    cv_chunks, engine_row_keys, justified_navigate, parse_cell, render_key, resolve_index,
    scroll_dom_to, use_find_in_page, use_scroller_attach, use_virtual_bridge, with_pinned, Bridge,
    ChunkSkip, Engine, FindLayer, FindLayout, GridEngine, GridKey, GridShape, JustifiedEngine,
    JustifiedOptions, Pins, RowPlan, ScrollReport, ScrollRequest, SizeEstimates, DEFAULT_SIZE,
};
pub use crate::r#virtual::{
    use_virtual_scroller, ScrollAlign, VirtualScroller, VirtualizeMode, DEFAULT_AUTO_THRESHOLD,
};
use crate::scroll_area::use_scroll_fade;
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
    /// Who is at row `idx`: a string unique among the rows that stays with the row when the rows
    /// are sorted, filtered or reordered. Defaults to the index (a row is its slot). With it the
    /// row's DOM node and the active cell's row follow the row instead of the slot, so after a
    /// sort the grid's one tab stop (and focus) stays on the same row's item. Read while
    /// rendering, so signals it reads re-render the grid.
    pub row_key: Option<Callback<usize, String>>,
    /// The width of a column by index, in px. Defaults to [`DEFAULT_COLUMN_WIDTH`] (120) for
    /// every column.
    pub column_width: Option<Callback<usize, u32>>,
    /// Renders the content of the cell at `(row, column)`.
    pub render_cell: Callback<(usize, usize), Element>,
    /// Windowed layouts, opt-in (experimental, backlog rows 153 and 309): the searchable text of
    /// an item, plain text without markup (a `VirtualList` row or a `VirtualWrapGrid` item by
    /// index, a `VirtualGrid` cell by `(row, column)`). When given, every item's text is also kept
    /// in the DOM inside a `hidden="until-found"` placeholder, so the browser's find-in-page (and a
    /// `#:~:text=` link) can match items that are not mounted; when the browser reveals a match it
    /// fires `beforematch` on that placeholder, and the component scrolls the real item into view
    /// (by the item's own box, so row estimates that are off do not matter) and marks it `data-find-match="true"` (the find bar highlights the hidden placeholder, not
    /// the item, so style the attribute to show which item matched). Costs about two DOM nodes
    /// per item (an empty text gets no placeholder), mounted after hydration (not
    /// server-rendered), and a mounted item's match counts twice. Engines without `until-found`
    /// (Firefox before 139, Safari before 26.2) treat it as `hidden`: items that are not mounted
    /// stay unfindable, as without the prop. Ignored by content-visibility mode, whose items are
    /// all in the DOM already.
    pub text_value: Option<Callback<(usize, usize), String>>,
    /// Renders the header of a column. When given, a header row of `role="columnheader"` cells
    /// is drawn above the body, pinned to the top of the grid while it scrolls unless
    /// [`pin_header`](Self::pin_header) is false.
    pub render_header: Option<Callback<usize, Element>>,
    /// Which way column `column` is sorted, for its header's `aria-sort` (`ascending` or
    /// `descending`). Return `None` for a column that is not the sort column: it gets no
    /// `aria-sort`, as the APG sortable-table pattern has only the sorted header carry one. The
    /// grid owns the `columnheader` element, so this is the caller's only way to set it. Read
    /// while rendering, so signals it reads re-render the grid. Ignored without `render_header`.
    pub column_sort: Option<Callback<usize, Option<SortDirection>>>,
    /// Whether the row whose [`row_key`](Self::row_key) is `key` is selected (the key is the
    /// index as a string without `row_key`). When given, every mounted row carries
    /// `aria-selected="true|false"`, and a selected one also `data-state="selected"` (the
    /// style hook). Keyed by the row's identity, not its slot: the callback is asked about the
    /// row each DOM node holds, so a selection survives a sort and a window move. Read while
    /// rendering, so signals it reads re-render the grid.
    pub row_selected: Option<Callback<String, bool>>,
    /// The height of the header row, in px. Ignored without `render_header`.
    #[props(default = 40)]
    pub header_height: u32,
    /// Whether the header row stays at the top of the grid while the body scrolls. Defaults to
    /// true; false lets the header scroll away with the body (a keyboard move then reveals a cell
    /// against the top edge of the grid, not below the header). Ignored without `render_header`.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub pin_header: ReadSignal<bool>,
    /// How many columns, from the first, stay at the inline start while the grid scrolls
    /// horizontally. They stack in column order, are always mounted and are not part of the
    /// column window. Defaults to 0; clamped to the column count. With at least one, the first
    /// column's cells are `role="rowheader"` (the others stay `gridcell`).
    #[props(default)]
    pub pinned_columns_start: ReadSignal<usize>,
    /// How many columns, from the last, stay at the inline end while the grid scrolls
    /// horizontally. They stack from the end edge inward, are always mounted and are not part of
    /// the column window. Defaults to 0; clamped to what the start columns leave.
    #[props(default)]
    pub pinned_columns_end: ReadSignal<usize>,
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
    /// How off-screen items are kept cheap. Defaults to [`VirtualizeMode::Auto`].
    #[props(default = ReadSignal::new(Signal::new(VirtualizeMode::Auto)))]
    pub virtualize: ReadSignal<VirtualizeMode>,
    /// With [`VirtualizeMode::Auto`]: the largest number of items that keeps every item in the
    /// DOM (a list counts its rows, a grid its cells: `count` for `VirtualWrapGrid`,
    /// `row_count x column_count` for `VirtualGrid`). Defaults to [`DEFAULT_AUTO_THRESHOLD`] (1000).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_AUTO_THRESHOLD)))]
    pub auto_threshold: ReadSignal<usize>,
    /// A scroll-edge fade on the container, which is the element that scrolls (the `dx-scroll-fade-*`
    /// classes of `dx-effects.css`; see [`ScrollFade`]). Defaults to [`ScrollFade::None`]. It follows
    /// the container's own scroll offset, whatever rows or cells are mounted, and a container whose
    /// content fits has no fade. The fade is a mask, so it also fades a border or background on the
    /// container (put those on a wrapper) and a `VirtualGrid`'s pinned header (prefer
    /// [`ScrollFade::Bottom`] / [`ScrollFade::End`] with `render_header`); [`ScrollFade::Both`] masks
    /// all four edges of a two-axis scroller.
    #[props(default)]
    pub fade: ScrollFade,
    /// A handle from [`use_virtual_scroller`] that scrolls to an item by index
    /// (`VirtualGrid` also by cell), in either mode: see [`VirtualScroller`].
    pub scroller: Option<VirtualScroller>,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # VirtualGrid
///
/// A data grid that, windowed, mounts only the cells in view: rows AND columns are windowed (a
/// grid of up to `auto_threshold` cells keeps every cell instead, see [`VirtualizeMode`]). See
/// the module documentation for the layouts, the keyboard model and the trade-offs.
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
///             pinned_columns_start: 1usize,
///             pinned_columns_end: 1usize,
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
/// `"header:column"`); the pinned ones carry `data-grid-pinned="start|end"` (and the one beside
/// the scrolling columns `data-grid-pin-edge="true"`). Positions and sizes are inline styles;
/// give the pinned header and columns an opaque background.
#[component]
pub fn VirtualGrid(props: VirtualGridProps) -> Element {
    let mut props = props;
    props.attributes = merge_attributes(vec![
        std::mem::take(&mut props.attributes),
        use_scroll_fade(props.fade),
    ]);
    let (row_count, column_count) = (props.row_count, props.column_count);
    let (virtualize, auto_threshold) = (props.virtualize, props.auto_threshold);

    // The mode body is a separate component, as in `VirtualList`: switching modes drops the old
    // body's scope (the windowed engine and its listeners) with it, and only a change of mode
    // re-renders this component.
    let windowed = use_memo(move || {
        let cells = row_count().saturating_mul(column_count());
        virtualize().resolve(cells, auto_threshold()) == VirtualizeMode::Windowed
    });

    if windowed() {
        rsx! {
            VirtualGridWindowed { grid: props }
        }
    } else {
        rsx! {
            VirtualGridContentVisibility { grid: props }
        }
    }
}

/// What both mode bodies of [`VirtualGrid`] keep the same way: the active cell (the one tab stop),
/// which follows its row through a reorder, and the focus request a keyboard move makes.
#[derive(Clone, Copy)]
struct GridFocus {
    active: Signal<(usize, usize)>,
    active_key: CopyValue<Option<String>>,
    focus_request: Signal<u64>,
    row_key: Option<Callback<usize, String>>,
    on_active_change: Option<Callback<(usize, usize)>>,
}

impl GridFocus {
    /// The hook: the active cell starts at `(0, 0)`, and when the rows are reordered it moves to
    /// where its row (by `row_key`) is now.
    fn use_new(
        row_key: Option<Callback<usize, String>>,
        row_count: ReadSignal<usize>,
        on_active_change: Option<Callback<(usize, usize)>>,
    ) -> Self {
        let active = use_signal(|| (0usize, 0usize));
        let active_key = use_hook(|| CopyValue::new(None::<String>));
        let focus_request = use_signal(|| 0u64);
        {
            let (row, column) = active();
            let key = active_key.peek().clone();
            let now = resolve_index(row_key, key.as_deref(), row, row_count());
            if let Some(now) = now.filter(|now| *now != row) {
                let mut active = active;
                active.set((now, column));
            }
        }
        Self {
            active,
            active_key,
            focus_request,
            row_key,
            on_active_change,
        }
    }

    /// Make `cell` the active one (a click, a focus, or a key press).
    fn set(self, cell: (usize, usize)) {
        let (mut active, mut active_key) = (self.active, self.active_key);
        active_key.set(self.row_key.map(|_| render_key(self.row_key, cell.0)));
        if *active.peek() != cell {
            active.set(cell);
            if let Some(callback) = self.on_active_change {
                callback.call(cell);
            }
        }
    }

    /// The grid pattern's key map: move the active cell for `event` (a page is `page` rows) and
    /// ask for it to be focused. Keys the grid does not handle keep their default action.
    fn on_key(self, event: &KeyboardEvent, direction: Direction, shape: GridShape, page: usize) {
        let Some(key) = grid_key(&event.data(), direction) else {
            return;
        };
        event.prevent_default();
        let from = *self.active.peek();
        self.set(shape.navigate(from, key, page));
        let mut focus_request = self.focus_request;
        focus_request += 1;
    }
}

/// Which edge of the grid a pinned column sits at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PinSide {
    /// Pinned at the inline start (the left in LTR, the right in RTL).
    Start,
    /// Pinned at the inline end.
    End,
}

impl PinSide {
    /// The `data-grid-pinned` value.
    fn token(self) -> &'static str {
        match self {
            PinSide::Start => "start",
            PinSide::End => "end",
        }
    }
}

/// One pinned column: its side, the distance from its edge, its width, and whether it is the
/// one next to the scrolling columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PinnedColumn {
    side: PinSide,
    /// The px between the grid's edge and this column: the widths of the pinned columns of the
    /// same side that lie between them.
    offset: u32,
    width: u32,
    /// The innermost pinned column of its side (the one that gets the separating edge).
    edge: bool,
}

/// How a [`VirtualGrid`]'s columns divide into the pinned start columns, the scrolling columns
/// and the pinned end columns, with the widths of the pinned ones (row 388).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pinning {
    /// The number of data columns.
    columns: usize,
    /// The widths of the pinned start columns, in column order.
    start: Vec<u32>,
    /// The widths of the pinned end columns, in column order.
    end: Vec<u32>,
}

impl Pinning {
    /// The division of `columns` columns into `start` leading and `end` trailing pinned columns:
    /// the start side is clamped to the column count first, the end side to what is left.
    /// `width` is a column's width in px.
    fn new(columns: usize, start: usize, end: usize, width: impl Fn(usize) -> u32) -> Self {
        let start = start.min(columns);
        let end = end.min(columns - start);
        Self {
            columns,
            start: (0..start).map(&width).collect(),
            end: (columns - end..columns).map(&width).collect(),
        }
    }

    /// How many leading columns are pinned.
    fn start_count(&self) -> usize {
        self.start.len()
    }

    /// How many trailing columns are pinned.
    fn end_count(&self) -> usize {
        self.end.len()
    }

    /// How many columns scroll (those between the two pinned sides).
    fn scrolling_count(&self) -> usize {
        self.columns - self.start.len() - self.end.len()
    }

    /// The px the pinned start columns take at the inline start.
    fn start_width(&self) -> u32 {
        self.start.iter().fold(0, |sum, w| sum.saturating_add(*w))
    }

    /// The px the pinned end columns take at the inline end.
    fn end_width(&self) -> u32 {
        self.end.iter().fold(0, |sum, w| sum.saturating_add(*w))
    }

    /// The pinned column `column` is, or `None` for a scrolling one.
    fn at(&self, column: usize) -> Option<PinnedColumn> {
        let end_from = self.columns - self.end.len();
        if column < self.start.len() {
            let offset = self.start[..column].iter().sum();
            Some(PinnedColumn {
                side: PinSide::Start,
                offset,
                width: self.start[column],
                edge: column + 1 == self.start.len(),
            })
        } else if column >= end_from && column < self.columns {
            let index = column - end_from;
            let offset = self.end[index + 1..].iter().sum();
            Some(PinnedColumn {
                side: PinSide::End,
                offset,
                width: self.end[index],
                edge: index == 0,
            })
        } else {
            None
        }
    }
}

/// One data or pinned cell of a [`VirtualGrid`], positioned by `style`. `row_header` makes it a
/// `role="rowheader"` (the first column, when a start column is pinned).
#[expect(
    clippy::too_many_arguments,
    reason = "one argument per input of a cell's layout; a struct would only rename the list"
)]
fn grid_cell(
    cell: (usize, usize),
    pin: Option<PinnedColumn>,
    row_header: bool,
    focusable: bool,
    find_match: bool,
    style: String,
    render_cell: Callback<(usize, usize), Element>,
    focus: GridFocus,
) -> Element {
    let (row, column) = cell;
    rsx! {
        div {
            // key-ok: a column is a position of the grid, not an item (`row_key` names rows)
            key: "{column}",
            role: if row_header { "rowheader" } else { "gridcell" },
            "aria-colindex": column + 1,
            "data-grid-cell": "{row}:{column}",
            "data-grid-pinned": pin.map(|pin| pin.side.token()),
            "data-grid-pin-edge": pin.filter(|pin| pin.edge).map(|_| "true"),
            "data-grid-active": focusable.then_some("true"),
            "data-find-match": find_match.then_some("true"),
            tabindex: if focusable { "0" } else { "-1" },
            style,
            // `focusin`, not `focus`: it also fires for a control inside the cell, so a click
            // or Tab that lands on one makes its cell the active one.
            onfocusin: move |_| focus.set(cell),
            {render_cell(cell)}
        }
    }
}

/// One column header of a [`VirtualGrid`], positioned by `style`.
fn header_cell(
    column: usize,
    pin: Option<PinnedColumn>,
    style: String,
    render_header: Callback<usize, Element>,
    column_sort: Option<Callback<usize, Option<SortDirection>>>,
) -> Element {
    rsx! {
        div {
            // key-ok: a column header is a position of the grid
            key: "{column}",
            role: "columnheader",
            "aria-colindex": column + 1,
            "data-grid-cell": "header:{column}",
            "aria-sort": sort_token(column_sort, column),
            "data-grid-pinned": pin.map(|pin| pin.side.token()),
            "data-grid-pin-edge": pin.filter(|pin| pin.edge).map(|_| "true"),
            style,
            {render_header(column)}
        }
    }
}

/// The style of a pinned cell: in flow, sticky at `offset` px from its edge. The innermost end
/// column also takes the free space before it, so the end columns reach the inline end of a grid
/// whose columns are narrower than its container.
fn pinned_cell_style(pin: PinnedColumn) -> String {
    let PinnedColumn {
        side,
        offset,
        width,
        edge,
    } = pin;
    let place = match side {
        PinSide::Start => format!("inset-inline-start: {offset}px;"),
        PinSide::End if edge => format!("inset-inline-end: {offset}px; margin-inline-start: auto;"),
        PinSide::End => format!("inset-inline-end: {offset}px;"),
    };
    format!("position: sticky; {place} z-index: 1; flex: none; box-sizing: border-box; width: {width}px; height: 100%;")
}

/// The style of the header row: pinned to the top, or scrolling with the body.
fn header_row_style(pinned: bool, width: u32, header: u32) -> String {
    let position = if pinned {
        "position: sticky; top: 0;"
    } else {
        "position: relative;"
    };
    format!("{position} z-index: 2; display: flex; width: {width}px; min-width: 100%; height: {header}px;")
}

/// The column widths of a [`VirtualGrid`], in px.
fn column_widths(column_count: usize, column_width: Option<Callback<usize, u32>>) -> Vec<u32> {
    (0..column_count)
        .map(|column| column_width.map_or(DEFAULT_COLUMN_WIDTH, |f| f(column)))
        .collect()
}

/// The px the grid's pinned edges cover, for scrolling a cell into view clear of them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct Cover {
    /// The header row's height (the body starts below it, pinned or not).
    header: u32,
    /// The px the pinned header covers at the top: `header`, or 0 when it scrolls away.
    top: u32,
    /// The px the pinned start columns cover at the inline start.
    start: u32,
    /// The px the pinned end columns cover at the inline end.
    end: u32,
}

impl Cover {
    fn new(header: u32, pin_header: bool, pinning: &Pinning) -> Self {
        Self {
            header,
            top: if pin_header { header } else { 0 },
            start: pinning.start_width(),
            end: pinning.end_width(),
        }
    }

    /// What the windowing engine needs of it.
    fn pins(self, pinning: &Pinning) -> Pins {
        Pins {
            start_columns: pinning.start_count(),
            top: self.top,
            lead: self.header - self.top,
            inline: self.start.saturating_add(self.end),
        }
    }
}

/// Every cell mounted, every column in flow, chunks of rows skipped by the browser while
/// off-screen: no engine, no scroll listener, find-in-page and the accessibility tree reach every
/// cell.
#[component]
fn VirtualGridContentVisibility(
    /// The grid's props, the fade already merged into `attributes`.
    grid: VirtualGridProps,
) -> Element {
    let VirtualGridProps {
        row_count,
        column_count,
        row_height,
        row_key,
        column_width,
        render_cell,
        render_header,
        column_sort,
        row_selected,
        header_height,
        pin_header,
        pinned_columns_start,
        pinned_columns_end,
        dir,
        on_active_change,
        scroller,
        attributes,
        ..
    } = grid;

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    use_cell_controls(container_id);
    let focus = GridFocus::use_new(row_key, row_count, on_active_change);
    let header = if render_header.is_some() {
        header_height
    } else {
        0
    };
    let geometry = use_memo(move || {
        let widths = column_widths(column_count(), column_width);
        let pinning = Pinning::new(
            widths.len(),
            pinned_columns_start(),
            pinned_columns_end(),
            |column| widths[column],
        );
        (widths, pinning)
    });
    let cover = Cover::new(header, pin_header(), &geometry.read().1);
    // Read when the focus effect runs, so it sees this render's cover, not the first one's.
    let mut cover_cell = use_hook(|| CopyValue::new(Cover::default()));
    cover_cell.set(cover);
    use_focus_active_cell(
        container_id,
        focus.active,
        focus.focus_request,
        true,
        cover_cell,
    );
    // Every cell is in the DOM: the browser scrolls to it, clear of the pinned edges.
    let on_scroll = use_callback(move |request: ScrollRequest| {
        let (Some(last_row), Some(last_column)) = (
            row_count.peek().checked_sub(1),
            column_count.peek().checked_sub(1),
        ) else {
            return;
        };
        let (row, column) = (
            request.index.min(last_row),
            request.column.map_or(0, |column| column.min(last_column)),
        );
        let cover = *cover_cell.read();
        scroll_dom_to(
            container_id.peek().clone(),
            format!(r#"[data-grid-cell="{row}:{column}"]"#),
            request.align,
            (cover.top, cover.start, cover.end),
            request.column.is_some(),
        );
    });
    use_scroller_attach(scroller, on_scroll);
    // The viewport's height, for PageUp/PageDown (a viewport of rows).
    let mut viewport = use_signal(|| 0u32);

    let shape = GridShape::full(row_count(), column_count());
    let tab_stop = shape.clamp((focus.active)());
    let onkeydown = move |event: KeyboardEvent| {
        let row = row_height
            .map_or(DEFAULT_ROW_HEIGHT, |f| f(focus.active.peek().0))
            .max(1);
        let page = (viewport.peek().saturating_sub(header) / row) as usize;
        focus.on_key(&event, direction, shape, page);
    };

    let header_rows = usize::from(render_header.is_some());
    let geometry = geometry.read();
    let (widths, pinning) = (&geometry.0, &geometry.1);
    let row_header = pinning.start_count() > 0;
    let width: u32 = widths.iter().copied().fold(0, u32::saturating_add);

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows() + header_rows,
        "aria-colcount": column_count(),
        "data-virtual-grid": "grid",
        "data-virtualize": "content-visibility",
    });
    let merged = merge_attributes(vec![attributes, owned]);
    let skip = ChunkSkip::Inline {
        estimate: row_height,
        fallback: DEFAULT_ROW_HEIGHT,
    };
    let in_flow = |width: u32| format!("flex: none; box-sizing: border-box; width: {width}px;");

    rsx! {
        div {
            onkeydown,
            onresize: move |event: Event<ResizeData>| {
                let size = event.data().get_border_box_size().unwrap_or_default();
                viewport.set(size.height.max(0.0) as u32);
            },
            ..merged,
            div { style: "position: relative; width: {width}px; min-width: 100%;",
                if let Some(render_header) = render_header {
                    div {
                        role: "row",
                        "aria-rowindex": 1,
                        "data-grid-header": "true",
                        style: header_row_style(pin_header(), width, header),
                        {widths.iter().enumerate().map(|(column, &size)| {
                            let pin = pinning.at(column);
                            let style = pin.map_or_else(|| in_flow(size), pinned_cell_style);
                            header_cell(column, pin, style, render_header, column_sort)
                        })}
                    }
                }
                {cv_chunks(shape.rows(), 0, skip, &[], |row| {
                    let height = row_height.map_or(DEFAULT_ROW_HEIGHT, |f| f(row));
                    rsx! {
                        div {
                            key: "{render_key(row_key, row)}",
                            ..row_attributes(
                                row_selected,
                                row_key,
                                row,
                                row + 1 + header_rows,
                                format!("display: flex; width: {width}px; min-width: 100%; height: {height}px;"),
                            ),
                            {widths.iter().enumerate().map(|(column, &size)| {
                                let pin = pinning.at(column);
                                let style = pin.map_or_else(|| in_flow(size), pinned_cell_style);
                                let focusable = tab_stop == Some((row, column));
                                grid_cell((row, column), pin, row_header && column == 0, focusable, false, style, render_cell, focus)
                            })}
                        }
                    }
                })}
            }
        }
    }
}

/// Only the cells in view (plus a buffer of rows and columns) mounted over a full-size canvas.
#[component]
fn VirtualGridWindowed(
    /// The grid's props, the fade already merged into `attributes`.
    grid: VirtualGridProps,
) -> Element {
    let VirtualGridProps {
        row_count,
        column_count,
        row_height,
        row_key,
        column_width,
        render_cell,
        text_value,
        render_header,
        column_sort,
        row_selected,
        header_height,
        pin_header,
        pinned_columns_start,
        pinned_columns_end,
        row_buffer,
        column_buffer,
        dir,
        on_active_change,
        scroller,
        attributes,
        ..
    } = grid;

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    use_cell_controls(container_id);
    // The shared DOM bridge (`primitives/src/virtual/bridge.rs`); both axes read each report.
    let bridge = use_virtual_bridge(
        container_id,
        GridEngine::new,
        |bridge: Bridge<GridEngine>, report: ScrollReport| {
            let mut engine = bridge.engine();
            engine.with_mut(|engine| engine.scroll(report));
            bridge.publish();
            bridge.run_deferred();
        },
    );
    let engine = bridge.engine();
    // The public scroll-to-cell: both axes aim past the pinned edges (`GridEngine::align_cell`).
    let on_scroll = use_callback(move |request: ScrollRequest| bridge.scroll_to(request));
    use_scroller_attach(scroller, on_scroll);
    let focus = GridFocus::use_new(row_key, row_count, on_active_change);

    let row_estimates = use_memo(move || {
        let height = move |row: usize| row_height.map_or(DEFAULT_ROW_HEIGHT, |f| f(row));
        SizeEstimates::evaluate(row_count(), Some(&height))
    });
    // The pinned columns are not part of the scrolling axis: the columns engine indexes the
    // columns between them (item `i` is data column `i + pinned start columns`).
    let column_geometry = use_memo(move || {
        let width = move |column: usize| column_width.map_or(DEFAULT_COLUMN_WIDTH, |f| f(column));
        let pinning = Pinning::new(
            column_count(),
            pinned_columns_start(),
            pinned_columns_end(),
            width,
        );
        let start = pinning.start_count();
        let scrolling = move |i: usize| width(i + start);
        (
            SizeEstimates::evaluate(pinning.scrolling_count(), Some(&scrolling)),
            pinning,
        )
    });
    let header = if render_header.is_some() {
        header_height
    } else {
        0
    };

    let cover = Cover::new(header, pin_header(), &column_geometry.read().1);
    // Read when the focus and reveal effects run, so they see this render's cover.
    let mut cover_cell = use_hook(|| CopyValue::new(Cover::default()));
    cover_cell.set(cover);
    use_focus_active_cell(
        container_id,
        focus.active,
        focus.focus_request,
        true,
        cover_cell,
    );

    // Find-in-page placeholders (opt-in, `virtual/find.rs`): a placeholder sits exactly where its
    // cell would (rows and columns have known sizes), so the browser's own scroll to a match
    // mounts the real cell; the grid marks it and scrolls it clear of the pinned edges (or to
    // its place first, when a reveal did not scroll there).
    // Every column's `(start, width)` on the canvas, the pinned ones included.
    let find_columns = use_memo(move || {
        let widths = column_widths(column_count(), column_width);
        let mut start = 0u32;
        widths
            .into_iter()
            .map(|width| {
                let cell = (start, width);
                start = start.saturating_add(width);
                cell
            })
            .collect::<Vec<_>>()
    });
    let mut find_match = use_signal(|| None::<(usize, usize)>);
    let hydrated = use_find_in_page(
        container_id,
        text_value.is_some(),
        None,
        move |index: String| {
            if let Some(cell) = parse_cell(&index) {
                find_match.set(Some(cell));
            }
        },
    );
    use_effect(move || {
        if let Some((row, column)) = find_match() {
            let top = (0..row)
                .map(|row| row_height.map_or(DEFAULT_ROW_HEIGHT, |f| f(row)))
                .fold(0u32, u32::saturating_add);
            let start = find_columns
                .peek()
                .get(column)
                .map_or(0, |(start, _)| *start);
            reveal_cell(
                container_id.peek().clone(),
                format!(r#"[data-grid-cell="{row}:{column}"]"#),
                *cover_cell.read(),
                (top, start),
            );
        }
    });

    let shape = GridShape::full(row_count(), column_count());
    let tab_stop = shape.clamp((focus.active)());
    let onkeydown = move |event: KeyboardEvent| {
        let page = engine.read().page_rows();
        focus.on_key(&event, direction, shape, page);
    };

    // Subscribe: this render runs again whenever the bridge publishes a new window.
    bridge.track();
    let plan = {
        let rows = row_estimates.read();
        let geometry = column_geometry.read();
        let mut engine = engine;
        engine.with_mut(|engine| {
            engine.sync(&rows, &geometry.0, cover.pins(&geometry.1));
            engine.plan(row_buffer(), column_buffer(), tab_stop)
        })
    };

    let header_rows = usize::from(render_header.is_some());
    let pinning = column_geometry.read().1.clone();
    let (start_count, end_count) = (pinning.start_count(), pinning.end_count());
    let width = cover
        .start
        .saturating_add(plan.pane_width)
        .saturating_add(cover.end);
    let columns = plan.columns;
    let find_match = find_match();

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows() + header_rows,
        "aria-colcount": column_count(),
        "data-virtual-grid": "grid",
        "data-virtualize": "windowed",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    // A cell: a pinned one sits in flow beside the pane (sticky at its edge), the others are
    // placed absolutely in the scrolling pane.
    let cell_pinning = pinning.clone();
    let cell = move |row: usize, column: usize, start: u32, size: u32| {
        let pin = cell_pinning.at(column);
        let style = match pin {
            Some(pin) => pinned_cell_style(pin),
            None => format!("position: absolute; inset-block: 0; inset-inline-start: {start}px; box-sizing: border-box; width: {size}px;"),
        };
        let focusable = tab_stop == Some((row, column));
        let matched = find_match == Some((row, column));
        let row_header = column == 0 && cell_pinning.start_count() > 0;
        grid_cell(
            (row, column),
            pin,
            row_header,
            focusable,
            matched,
            style,
            render_cell,
            focus,
        )
    };
    let pinned_header = |render_header: Callback<usize, Element>, column: usize| {
        let pin = pinning.at(column);
        header_cell(
            column,
            pin,
            pin.map(pinned_cell_style).unwrap_or_default(),
            render_header,
            column_sort,
        )
    };
    let total_columns = column_count();
    let pane_width = plan.pane_width;
    let header_cells = columns.clone();
    let body_rows = shape.rows();

    rsx! {
        div { onkeydown, ..merged,
            div { style: "position: relative; width: {width}px; min-width: 100%;",
                if let Some(render_header) = render_header {
                    div {
                        role: "row",
                        "aria-rowindex": 1,
                        "data-grid-header": "true",
                        style: header_row_style(pin_header(), width, header),
                        {(0..start_count).map(|column| pinned_header(render_header, column))}
                        div { style: "position: relative; flex: none; width: {pane_width}px;",
                            {header_cells.iter().map(|column| header_cell(
                                column.index(),
                                None,
                                format!("position: absolute; inset-block: 0; inset-inline-start: {}px; box-sizing: border-box; width: {}px;", column.start(), column.size()),
                                render_header,
                                column_sort,
                            ))}
                        }
                        {(total_columns - end_count..total_columns).map(|column| pinned_header(render_header, column))}
                    }
                }
                div { style: "position: relative; width: {width}px; min-width: 100%; height: {plan.body_height}px;",
                    if let Some(text_value) = text_value.filter(|_| hydrated()) {
                        FindLayer {
                            rows: body_rows,
                            layout: FindLayout::Columns(find_columns()),
                            row_height: move |row| row_height.map_or(DEFAULT_ROW_HEIGHT, |f| f(row)),
                            text: text_value,
                        }
                    }
                    for row in plan.rows {
                        div {
                            key: "{render_key(row_key, row.index())}",
                            ..row_attributes(
                                row_selected,
                                row_key,
                                row.index(),
                                row.index() + 1 + header_rows,
                                format!("position: absolute; inset-inline-start: 0; top: {}px; display: flex; width: {width}px; min-width: 100%; height: {}px;", row.start(), row.size()),
                            ),
                            {(0..start_count).map(|column| cell(row.index(), column, 0, 0))}
                            div { style: "position: relative; flex: none; width: {pane_width}px;",
                                // An iterator of keyed elements, not a `for` around an
                                // expression: the key must be on each list item's root, or a
                                // window move re-creates every cell (and drops focus).
                                {columns.iter().map(|column| cell(row.index(), column.index(), column.start(), column.size()))}
                            }
                            {(total_columns - end_count..total_columns).map(|column| cell(row.index(), column, 0, 0))}
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
    /// The total number of items.
    pub count: ReadSignal<usize>,
    /// How many items each row holds (equal-width tracks). At least 1. Changing it forgets the
    /// measured row heights (row `i` then holds other items).
    pub columns: ReadSignal<usize>,
    /// Estimates the height of a row by row index, in px, until it is measured. Without it the
    /// grid estimates from the average of the rows measured so far (100px before the first).
    pub estimate_row_height: Option<Callback<usize, u32>>,
    /// Renders the item at an absolute index.
    pub render_item: Callback<usize, Element>,
    /// Who is at index `idx`: a string unique among the items that stays with the item when the
    /// items are sorted, filtered or reordered. Defaults to the index (an item is its slot). With
    /// it the item's DOM node (and so focus and any state inside it), the item that stays mounted
    /// while it holds focus and the measured sizes follow the item instead of the slot. Read
    /// while rendering, so signals it reads re-render the component.
    pub item_key: Option<Callback<usize, String>>,
    /// Windowed layouts, opt-in (experimental, backlog rows 153 and 309): the searchable text of
    /// an item, plain text without markup (a `VirtualList` row or a `VirtualWrapGrid` item by
    /// index, a `VirtualGrid` cell by `(row, column)`). When given, every item's text is also kept
    /// in the DOM inside a `hidden="until-found"` placeholder, so the browser's find-in-page (and a
    /// `#:~:text=` link) can match items that are not mounted; when the browser reveals a match it
    /// fires `beforematch` on that placeholder, and the component scrolls the real item into view
    /// (by the item's own box, so row estimates that are off do not matter) and marks it `data-find-match="true"` (the find bar highlights the hidden placeholder, not
    /// the item, so style the attribute to show which item matched). Costs about two DOM nodes
    /// per item (an empty text gets no placeholder), mounted after hydration (not
    /// server-rendered), and a mounted item's match counts twice. Engines without `until-found`
    /// (Firefox before 139, Safari before 26.2) treat it as `hidden`: items that are not mounted
    /// stay unfindable, as without the prop. Ignored by content-visibility mode, whose items are
    /// all in the DOM already.
    pub text_value: Option<Callback<usize, String>>,
    /// How many rows to mount beyond each edge of the viewport, so a scroll never shows a blank edge
    /// (the window is re-centred once fewer than half of them remain, so between `buffer / 2` and
    /// `buffer` rows are always mounted beyond each edge). Only a windowed layout reads it:
    /// `VirtualList` in content-visibility mode ignores it. Defaults to 8 rows for `VirtualList` and 4
    /// for `VirtualWrapGrid`.
    #[props(default = ReadSignal::new(Signal::new(4)))]
    pub buffer: ReadSignal<usize>,
    /// The reading direction for the horizontal arrow keys. Defaults to the nearest
    /// `DirectionProvider`, else left-to-right.
    pub dir: Option<Direction>,
    /// Called with the item index when the active item changes (by keyboard, click or focus).
    pub on_active_change: Option<Callback<usize>>,
    /// How off-screen items are kept cheap. Defaults to [`VirtualizeMode::Auto`].
    #[props(default = ReadSignal::new(Signal::new(VirtualizeMode::Auto)))]
    pub virtualize: ReadSignal<VirtualizeMode>,
    /// With [`VirtualizeMode::Auto`]: the largest number of items that keeps every item in the
    /// DOM (a list counts its rows, a grid its cells: `count` for `VirtualWrapGrid`,
    /// `row_count x column_count` for `VirtualGrid`). Defaults to [`DEFAULT_AUTO_THRESHOLD`] (1000).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_AUTO_THRESHOLD)))]
    pub auto_threshold: ReadSignal<usize>,
    /// A scroll-edge fade on the container, which is the element that scrolls (the `dx-scroll-fade-*`
    /// classes of `dx-effects.css`; see [`ScrollFade`]). Defaults to [`ScrollFade::None`]. It follows
    /// the container's own scroll offset, whatever rows or cells are mounted, and a container whose
    /// content fits has no fade. The fade is a mask, so it also fades a border or background on the
    /// container (put those on a wrapper) and a `VirtualGrid`'s pinned header (prefer
    /// [`ScrollFade::Bottom`] / [`ScrollFade::End`] with `render_header`); [`ScrollFade::Both`] masks
    /// all four edges of a two-axis scroller.
    #[props(default)]
    pub fade: ScrollFade,
    /// A handle from [`use_virtual_scroller`] that scrolls to an item by index
    /// (`VirtualGrid` also by cell), in either mode: see [`VirtualScroller`].
    pub scroller: Option<VirtualScroller>,
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
/// The canvas inside the container carries `data-measured="false"` while its height is the
/// estimates' sum (the server's markup and the client's first frame) and `"true"` once a row
/// has been measured, so a stylesheet or a test can tell an estimated canvas from a measured one.
#[component]
pub fn VirtualWrapGrid(props: VirtualWrapGridProps) -> Element {
    let mut props = props;
    props.attributes = merge_attributes(vec![
        std::mem::take(&mut props.attributes),
        use_scroll_fade(props.fade),
    ]);
    let (count, virtualize, auto_threshold) = (props.count, props.virtualize, props.auto_threshold);

    // One body per mode, as in `VirtualGrid`.
    let windowed = use_memo(move || {
        virtualize().resolve(count(), auto_threshold()) == VirtualizeMode::Windowed
    });

    if windowed() {
        rsx! {
            VirtualWrapGridWindowed { grid: props }
        }
    } else {
        rsx! {
            VirtualWrapGridContentVisibility { grid: props }
        }
    }
}

/// The active item of a [`VirtualWrapGrid`] (the one tab stop), which follows its item through a
/// reorder, and the focus request a keyboard move makes; the same in both mode bodies.
#[derive(Clone, Copy)]
struct WrapFocus {
    active: Signal<usize>,
    active_key: CopyValue<Option<String>>,
    focus_request: Signal<u64>,
    item_key: Option<Callback<usize, String>>,
    on_active_change: Option<Callback<usize>>,
}

impl WrapFocus {
    /// The hook: the active item starts at 0, and when the items are reordered it moves to where
    /// its item (by `item_key`) is now.
    fn use_new(
        item_key: Option<Callback<usize, String>>,
        count: ReadSignal<usize>,
        on_active_change: Option<Callback<usize>>,
    ) -> Self {
        let active = use_signal(|| 0usize);
        let active_key = use_hook(|| CopyValue::new(None::<String>));
        let focus_request = use_signal(|| 0u64);
        {
            let index = active();
            let key = active_key.peek().clone();
            let now = resolve_index(item_key, key.as_deref(), index, count());
            if let Some(now) = now.filter(|now| *now != index) {
                let mut active = active;
                active.set(now);
            }
        }
        Self {
            active,
            active_key,
            focus_request,
            item_key,
            on_active_change,
        }
    }

    /// Make item `index` the active one.
    fn set(self, index: usize) {
        let (mut active, mut active_key) = (self.active, self.active_key);
        active_key.set(self.item_key.map(|_| render_key(self.item_key, index)));
        if *active.peek() != index {
            active.set(index);
            if let Some(callback) = self.on_active_change {
                callback.call(index);
            }
        }
    }

    /// The grid pattern's key map over rows of `columns` items (a page is `page` rows).
    fn on_key(
        self,
        event: &KeyboardEvent,
        direction: Direction,
        shape: GridShape,
        columns: usize,
        page: usize,
    ) {
        let Some(key) = grid_key(&event.data(), direction) else {
            return;
        };
        event.prevent_default();
        let from = index_to_cell(*self.active.peek(), columns);
        let (row, column) = shape.navigate(from, key, page);
        self.set(row * columns + column);
        let mut focus_request = self.focus_request;
        focus_request += 1;
    }
}

/// The selector of the mounted item a find match names (`data-find-index` is the item index).
fn wrap_item_selector(index: &str) -> String {
    format!(r#"[data-grid-index="{index}"]"#)
}

/// The cells of row `row` of a [`VirtualWrapGrid`].
#[expect(
    clippy::too_many_arguments,
    reason = "one argument per input of a row's layout; a struct would only rename the list"
)]
fn wrap_row_cells(
    row: usize,
    shape: GridShape,
    columns: usize,
    tab_stop: Option<(usize, usize)>,
    find_match: Option<usize>,
    item_key: Option<Callback<usize, String>>,
    render_item: Callback<usize, Element>,
    focus: WrapFocus,
) -> Element {
    rsx! {
        {(0..shape.row_len(row)).map(|column| {
            let index = row * columns + column;
            let focusable = tab_stop == Some((row, column));
            rsx! {
                div {
                    key: "{render_key(item_key, index)}",
                    role: "gridcell",
                    "aria-colindex": column + 1,
                    "data-grid-cell": "{row}:{column}",
                    "data-grid-index": index,
                    "data-find-match": (find_match == Some(index)).then_some("true"),
                    tabindex: if focusable { "0" } else { "-1" },
                    style: "min-width: 0;",
                    onfocus: move |_| focus.set(index),
                    {render_item(index)}
                }
            }
        })}
    }
}

/// Every item mounted in rows of `columns` tracks, chunks of rows skipped by the browser while
/// off-screen: no engine, no measuring, no scroll listener.
#[component]
fn VirtualWrapGridContentVisibility(
    /// The grid's props, the fade already merged into `attributes`.
    grid: VirtualWrapGridProps,
) -> Element {
    let VirtualWrapGridProps {
        count,
        columns,
        estimate_row_height,
        render_item,
        item_key,
        dir,
        on_active_change,
        scroller,
        attributes,
        ..
    } = grid;

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    let focus = WrapFocus::use_new(item_key, count, on_active_change);
    // Every item is in the DOM: the browser scrolls to it.
    let on_scroll = use_callback(move |request: ScrollRequest| {
        if let Some(last) = count.peek().checked_sub(1) {
            scroll_dom_to(
                container_id.peek().clone(),
                wrap_item_selector(&request.index.min(last).to_string()),
                request.align,
                (0, 0, 0),
                false,
            );
        }
    });
    use_scroller_attach(scroller, on_scroll);
    let active_cell = use_active_cell(focus.active, columns);
    let no_cover = use_hook(|| CopyValue::new(Cover::default()));
    use_focus_active_cell(
        container_id,
        active_cell,
        focus.focus_request,
        false,
        no_cover,
    );
    // The viewport's height, for PageUp/PageDown (a viewport of rows).
    let mut viewport = use_signal(|| 0u32);

    let columns_now = columns().max(1);
    let shape = GridShape::wrapping(count(), columns_now);
    let tab_stop = shape.clamp(index_to_cell((focus.active)(), columns_now));
    let row_height = move |row: usize| estimate_row_height.map_or(DEFAULT_SIZE, |f| f(row));
    let onkeydown = move |event: KeyboardEvent| {
        let row = row_height(index_to_cell(*focus.active.peek(), columns_now).0).max(1);
        let page = (*viewport.peek() / row) as usize;
        focus.on_key(&event, direction, shape, columns_now, page);
    };

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows(),
        "aria-colcount": columns_now,
        "data-virtual-grid": "wrap",
        "data-virtualize": "content-visibility",
    });
    let merged = merge_attributes(vec![attributes, owned]);
    let skip = ChunkSkip::Inline {
        estimate: estimate_row_height,
        fallback: DEFAULT_SIZE,
    };

    rsx! {
        div {
            onkeydown,
            onresize: move |event: Event<ResizeData>| {
                let size = event.data().get_border_box_size().unwrap_or_default();
                viewport.set(size.height.max(0.0) as u32);
            },
            ..merged,
            {cv_chunks(shape.rows(), 0, skip, &[], |row| rsx! {
                div {
                    // key-ok: a wrapping grid's row is a slot of `columns` items; its cells are keyed
                    key: "{row}",
                    role: "row",
                    "aria-rowindex": row + 1,
                    style: "box-sizing: border-box; display: grid; grid-template-columns: repeat({columns_now}, minmax(0, 1fr));",
                    {wrap_row_cells(row, shape, columns_now, tab_stop, None, item_key, render_item, focus)}
                }
            })}
        }
    }
}

/// Only the rows in view (plus a buffer) mounted over a full-height canvas, rows measured as
/// they render.
#[component]
fn VirtualWrapGridWindowed(
    /// The grid's props, the fade already merged into `attributes`.
    grid: VirtualWrapGridProps,
) -> Element {
    let VirtualWrapGridProps {
        count,
        columns,
        estimate_row_height,
        render_item,
        item_key,
        text_value,
        buffer,
        dir,
        on_active_change,
        scroller,
        attributes,
        ..
    } = grid;

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    // The shared DOM bridge with the one-axis flow `VirtualList` uses: the rows axis, with the
    // anchoring correction for rows measured above the viewport.
    let bridge = use_virtual_bridge(container_id, Engine::new, Bridge::scroll_axis);
    let engine = bridge.engine();
    // The public scroll-to-index: an item is scrolled to by the row that holds it.
    let on_scroll = use_callback(move |request: ScrollRequest| {
        if let Some(last) = count.peek().checked_sub(1) {
            let row = request.index.min(last) / columns.peek().max(1);
            bridge.scroll_to(ScrollRequest {
                index: row,
                column: None,
                ..request
            });
        }
    });
    use_scroller_attach(scroller, on_scroll);
    let laid_out_for = use_hook(|| CopyValue::new(0usize));
    // The active item, as an index: it keeps its item when the column count changes.
    let focus = WrapFocus::use_new(item_key, count, on_active_change);

    let estimates = use_memo(move || {
        let shape = GridShape::wrapping(count(), columns());
        let estimate = estimate_row_height.map(|f| move |row: usize| f(row));
        SizeEstimates::evaluate(
            shape.rows(),
            estimate.as_ref().map(|f| f as &dyn Fn(usize) -> u32),
        )
        .with_keys(engine_row_keys(item_key, count(), columns()))
    });

    let active_cell = use_active_cell(focus.active, columns);
    let no_cover = use_hook(|| CopyValue::new(Cover::default()));
    use_focus_active_cell(
        container_id,
        active_cell,
        focus.focus_request,
        false,
        no_cover,
    );

    // Find-in-page placeholders (opt-in, `virtual/find.rs`): rows of placeholders at the
    // estimated row heights; a match names the item, and the grid centres its row (which mounts
    // it) and marks it.
    let mut find_match = use_signal(|| None::<usize>);
    let hydrated = use_find_in_page(
        container_id,
        text_value.is_some(),
        Some(wrap_item_selector),
        move |index: String| {
            if let Ok(index) = index.parse::<usize>() {
                find_match.set(Some(index));
                bridge.reveal(index / columns.peek().max(1));
            }
        },
    );

    let columns_now = columns().max(1);
    let shape = GridShape::wrapping(count(), columns_now);
    let tab_stop = shape.clamp(index_to_cell((focus.active)(), columns_now));
    let onkeydown = move |event: KeyboardEvent| {
        let page = engine.read().visible_len();
        focus.on_key(&event, direction, shape, columns_now, page);
    };

    let onresize = move |row: usize| {
        move |event: Event<ResizeData>| {
            let size = event.data().get_border_box_size().unwrap_or_default();
            bridge.measure_exact(row, size.height);
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
    // Whether the canvas height comes from measured rows (the client, after mount) or from the
    // estimates alone (the server, and the client's first frame).
    let measured = engine.read().has_measurements();
    let find_match = find_match();
    let items = count();

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": shape.rows(),
        "aria-colcount": columns_now,
        "data-virtual-grid": "wrap",
        "data-virtualize": "windowed",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    rsx! {
        div { onkeydown, ..merged,
            div {
                "data-measured": "{measured}",
                style: "position: relative; height: {height}px;",
                if let Some(text_value) = text_value.filter(|_| hydrated()) {
                    FindLayer {
                        rows: shape.rows(),
                        layout: FindLayout::Tracks { count: items, columns: columns_now },
                        row_height: move |row| estimate_row_height.map_or(DEFAULT_SIZE, |f| f(row)),
                        text: move |(row, slot): (usize, usize)| text_value(row * columns_now + slot),
                    }
                }
                for row in rows {
                    div {
                        // key-ok: a wrapping grid's row is a slot of `columns` items; its cells are keyed
                        key: "{row.index()}",
                        role: "row",
                        "aria-rowindex": row.index() + 1,
                        style: "position: absolute; inset-inline: 0; top: {row.start()}px; box-sizing: border-box; display: grid; grid-template-columns: repeat({columns_now}, minmax(0, 1fr));",
                        onresize: onresize(row.index()),
                        {wrap_row_cells(row.index(), shape, columns_now, tab_stop, find_match, item_key, render_item, focus)}
                    }
                }
            }
        }
    }
}

/// The target row height [`VirtualJustifiedGrid`] uses without `target_row_height`, in px.
pub const DEFAULT_TARGET_ROW_HEIGHT: f64 = 150.0;
/// The gap [`VirtualJustifiedGrid`] uses without `gap`, in px.
pub const DEFAULT_TILE_GAP: f64 = 4.0;
/// How far beyond the viewport [`VirtualJustifiedGrid`] mounts tiles without `overscan`, in px.
pub const DEFAULT_OVERSCAN: f64 = 320.0;

/// Where a [`VirtualJustifiedGrid`] tile sits: its box in the grid's canvas, in px, `x` measured
/// from the inline start (the right edge in RTL) and `y` from the top. The tile already has this
/// size; the size is passed so a renderer can pick an image resolution.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TileRect {
    /// Distance of the tile's inline-start edge from the canvas's.
    pub x: f64,
    /// Distance of the tile's top edge from the canvas's.
    pub y: f64,
    /// The tile's width, which is `height * aspect`.
    pub width: f64,
    /// The tile's height, its row's height.
    pub height: f64,
}

/// The props for the [`VirtualJustifiedGrid`] component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualJustifiedGridProps {
    /// The aspect ratio (width / height) of every item, in order; the item count is its length.
    /// Each tile has exactly its item's aspect, so a photo shown in it is never cropped. A zero,
    /// negative or non-finite aspect is drawn as a 1:20 sliver.
    pub aspects: ReadSignal<Vec<f64>>,
    /// Renders item `index` inside its tile, given the tile's box (see [`TileRect`]). The tile
    /// is sized and placed already: fill it, e.g. with an `img` at `width: 100%; height: 100%`.
    pub render_tile: Callback<(usize, TileRect), Element>,
    /// The row height, in px, that rows are packed toward: a full row takes items until they
    /// would overflow the width at this height, then shrinks to fill the width exactly, so it is
    /// never taller (unless `min_tile_width` backs it off). Defaults to
    /// [`DEFAULT_TARGET_ROW_HEIGHT`] (150). The last row keeps this height and may end short.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_TARGET_ROW_HEIGHT)))]
    pub target_row_height: ReadSignal<f64>,
    /// The space between tiles and between rows, in px. Defaults to [`DEFAULT_TILE_GAP`] (4).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_TILE_GAP)))]
    pub gap: ReadSignal<f64>,
    /// The narrowest a tile may be squeezed, in px: a row whose narrowest tile would fall below it
    /// gives up items to the next row and grows taller (best effort, bounded at three times the
    /// target height). `0` (the default) disables it.
    #[props(default)]
    pub min_tile_width: ReadSignal<f64>,
    /// The width the rows fill, in px. `None` (the default) fills the grid's own width and lays
    /// out again when it resizes: the grid scrolls vertically only. A width larger than the grid
    /// makes a two-axis wall: the grid scrolls both ways and windows the tiles of each row by
    /// their horizontal position too.
    #[props(default)]
    pub canvas_width: ReadSignal<Option<f64>>,
    /// How far beyond each edge of the viewport tiles are mounted, in px (the window is moved
    /// once fewer than half of them remain). Defaults to [`DEFAULT_OVERSCAN`] (320).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_OVERSCAN)))]
    pub overscan: ReadSignal<f64>,
    /// The reading direction for the horizontal arrow keys. Defaults to the nearest
    /// `DirectionProvider`, else left-to-right. Set the matching `dir` attribute (or inherit one)
    /// so the tiles flip with it.
    pub dir: Option<Direction>,
    /// Called with the item index when the active tile changes (by keyboard, click or focus).
    pub on_active_change: Option<Callback<usize>>,
    /// The active tile (the grid's one tab stop) as a signal the caller owns, both ways: the
    /// grid writes every keyboard, click or focus move into it, and a write from outside moves
    /// the active tile there, scrolls it into view and focuses it (a "jump to photo" button, or
    /// keeping one selection across a switch between layouts). Without it the grid keeps its own.
    pub active: Option<Signal<usize>>,
    /// A scroll-edge fade on the container, which is the element that scrolls (the `dx-scroll-fade-*`
    /// classes of `dx-effects.css`; see [`ScrollFade`]). Defaults to [`ScrollFade::None`]. It follows
    /// the container's own scroll offset, whatever rows or cells are mounted, and a container whose
    /// content fits has no fade. The fade is a mask, so it also fades a border or background on the
    /// container (put those on a wrapper) and a `VirtualGrid`'s pinned header (prefer
    /// [`ScrollFade::Bottom`] / [`ScrollFade::End`] with `render_header`); [`ScrollFade::Both`] masks
    /// all four edges of a two-axis scroller.
    #[props(default)]
    pub fade: ScrollFade,
    /// A handle from [`use_virtual_scroller`] that scrolls to an item by index
    /// (`VirtualGrid` also by cell), in either mode: see [`VirtualScroller`].
    pub scroller: Option<VirtualScroller>,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The layout options for a width, from the props (read reactively in a render, peeked in a
/// handler, through `read`).
fn justified_options(
    width: Option<f64>,
    target_row_height: f64,
    gap: f64,
    min_tile_width: f64,
) -> Option<JustifiedOptions> {
    width.map(|width| {
        JustifiedOptions::new(width, target_row_height, gap).with_min_tile_width(min_tile_width)
    })
}

/// # VirtualJustifiedGrid
///
/// A photo wall that mounts only the tiles in view: items of mixed aspect ratios packed into
/// justified rows (each row as tall as it must be for its tiles to fill the width edge to edge,
/// every tile exactly its item's aspect, nothing cropped). Unlike [`VirtualGrid`], a row's
/// composition depends on the content, so there are no columns.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::virtual_grid::{TileRect, VirtualJustifiedGrid};
///
/// #[component]
/// fn Demo() -> Element {
///     let aspects = use_signal(|| (0..10_000).map(|i| [1.5, 0.75, 1.0][i % 3]).collect::<Vec<f64>>());
///     rsx! {
///         VirtualJustifiedGrid {
///             aspects,
///             target_row_height: 180.0,
///             render_tile: |(index, _rect): (usize, TileRect)| rsx! { "Photo {index}" },
///             style: "height: 30rem; overflow: auto;",
///             "aria-label": "Photos",
///         }
///     }
/// }
/// ```
///
/// ## Keyboard and focus
///
/// The grid pattern with spatial rows: one tab stop (the active tile), `ArrowLeft`/`ArrowRight`
/// move in reading order (past a row's end to the next row's start), `ArrowUp`/`ArrowDown` move
/// to the tile of the adjacent row whose horizontal centre is closest, `Home`/`End` to the row's
/// ends, `Ctrl+Home`/`Ctrl+End` to the first/last tile, `PageUp`/`PageDown` a viewport up/down.
/// The horizontal arrows mirror in RTL. The active tile stays rendered wherever the grid
/// scrolls, and a re-layout that re-creates it gives focus back to it.
///
/// ## Styling
///
/// The container is the scroll container (give it a height and `overflow: auto`, plus
/// `scrollbar-gutter: stable` so a scrollbar appearing does not re-flow the rows); it owns `id`,
/// `role` and `aria-rowcount`. A fluid grid carries `data-justified-axes="block"`, a wall
/// `"both"`. The canvas inside it carries `data-measured="false"` until a fluid grid knows its
/// width (the server's markup: no rows yet, `aria-busy`), `"true"` once rows are laid out. Rows
/// are `[role=row]`, tiles `[role=gridcell]` with `data-grid-index` and
/// `data-grid-cell="row:column"`; positions and sizes are inline styles. Do not pad the container:
/// the rows fill its content box.
#[component]
pub fn VirtualJustifiedGrid(props: VirtualJustifiedGridProps) -> Element {
    let VirtualJustifiedGridProps {
        aspects,
        render_tile,
        target_row_height,
        gap,
        min_tile_width,
        canvas_width,
        overscan,
        dir,
        on_active_change,
        active: active_prop,
        fade,
        scroller,
        attributes,
    } = props;
    let attributes = merge_attributes(vec![attributes, use_scroll_fade(fade)]);

    let direction = use_direction(dir);
    let container_id = use_unique_id();
    let own_active = use_signal(|| 0usize);
    let active = active_prop.unwrap_or(own_active);
    let mut focus_request = use_signal(|| 0u64);
    // The active index the grid itself last set or saw, so a write from outside is told apart.
    let mut seen_active = use_hook(|| CopyValue::new(*active.peek()));
    // Whether focus is inside the grid, so a re-layout that re-creates the focused tile under
    // another row can hand focus back to it.
    let mut focus_inside = use_hook(|| CopyValue::new(false));

    // The shared DOM bridge. A fluid grid lays out for the measured width; when that changes the
    // tile at the top of the viewport is kept in place (the scroll position is corrected).
    let bridge = use_virtual_bridge(
        container_id,
        JustifiedEngine::new,
        move |bridge: Bridge<JustifiedEngine>, report: ScrollReport| {
            let mut engine = bridge.engine();
            let fluid = canvas_width.peek().is_none();
            let corrected = engine.with_mut(|engine| {
                let resized = fluid && engine.measured_width() != Some(f64::from(report.width));
                let anchor = engine.anchor();
                engine.scroll(report);
                if !resized {
                    return None;
                }
                let options = justified_options(
                    Some(f64::from(report.width)),
                    *target_row_height.peek(),
                    *gap.peek(),
                    *min_tile_width.peek(),
                );
                engine.sync(&aspects.peek(), options);
                let top = anchor.and_then(|index| engine.tile_top(index))?;
                let top = top.round().clamp(0.0, f64::from(u32::MAX)) as u32;
                engine.set_top(top);
                Some(top)
            });
            if let Some(top) = corrected {
                if top != report.top {
                    bridge.set_scroll_top(top);
                }
                if *focus_inside.peek() {
                    focus_request += 1;
                }
            }
            bridge.publish();
            bridge.run_deferred();
        },
    );
    let engine = bridge.engine();
    // The public scroll-to-index: the tile's row, from the layout.
    let on_scroll = use_callback(move |request: ScrollRequest| bridge.scroll_to(request));
    use_scroller_attach(scroller, on_scroll);
    use_focus_active_tile(container_id, active, focus_request);

    let set_active = move |index: usize| {
        let (mut active, mut seen_active) = (active, seen_active);
        seen_active.set(index);
        if *active.peek() != index {
            active.set(index);
            if let Some(callback) = on_active_change {
                callback.call(index);
            }
        }
    };
    // A write from outside: reveal and focus the new active tile (it is pinned, so the render
    // that follows mounts it).
    use_effect(move || {
        let current = active();
        if current != *seen_active.peek() {
            seen_active.set(current);
            focus_request += 1;
        }
    });

    let onkeydown = move |event: KeyboardEvent| {
        let Some(key) = grid_key(&event.data(), direction) else {
            return;
        };
        event.prevent_default();
        let from = *active.peek();
        let to = engine.with(|engine| {
            let page = f64::from(engine.viewport().height);
            justified_navigate(engine.layout(), from, key, page)
        });
        set_active(to);
        focus_request += 1;
    };

    // Subscribe: this render runs again whenever the bridge publishes a new window.
    bridge.track();
    let fixed_width = canvas_width();
    let count = aspects.read().len();
    let tab_stop = (count > 0).then(|| active().min(count - 1));
    let (rows, row_count, width, height) = {
        let aspects = aspects.read();
        let mut engine = engine;
        engine.with_mut(|engine| {
            let width = fixed_width.or_else(|| engine.measured_width());
            let options = justified_options(width, target_row_height(), gap(), min_tile_width());
            engine.sync(&aspects, options);
            let rows = engine.plan(overscan(), tab_stop);
            let layout = engine.layout();
            (
                rows,
                layout.rows.len(),
                layout.container_width,
                layout.total_height,
            )
        })
    };
    let measured = fixed_width.is_some() || engine.read().measured_width().is_some();
    let canvas_width_style = if fixed_width.is_some() {
        format!("{width:.3}px")
    } else {
        "100%".to_string()
    };

    let owned = attributes!(div {
        id: container_id,
        role: "grid",
        "aria-rowcount": if measured { row_count as i64 } else { -1 },
        "aria-busy": (!measured).then_some("true"),
        "data-virtual-grid": "justified",
        "data-justified-axes": if fixed_width.is_some() { "both" } else { "block" },
    });
    let merged = merge_attributes(vec![attributes, owned]);

    let tile = move |plan: &RowPlan, tile: crate::r#virtual::layout::justified::Tile| {
        let index = tile.index;
        let column = index - plan.start;
        let focusable = tab_stop == Some(index);
        let rect = TileRect {
            x: tile.x,
            y: tile.y,
            width: tile.width,
            height: tile.height,
        };
        rsx! {
            div {
                // key-ok: the justified grid has no `item_key` (its items are bare aspects)
                key: "{index}",
                role: "gridcell",
                "aria-colindex": column + 1,
                "data-grid-index": index,
                "data-grid-cell": "{plan.row}:{column}",
                tabindex: if focusable { "0" } else { "-1" },
                style: "position: absolute; top: 0; inset-inline-start: {tile.x:.3}px; box-sizing: border-box; width: {tile.width:.3}px; height: 100%;",
                onfocus: move |_| set_active(index),
                {render_tile((index, rect))}
            }
        }
    };

    rsx! {
        div {
            onkeydown,
            onfocusin: move |_| focus_inside.set(true),
            onfocusout: move |_| focus_inside.set(false),
            ..merged,
            div {
                "data-measured": "{measured}",
                style: "position: relative; width: {canvas_width_style}; height: {height:.3}px;",
                for plan in rows {
                    div {
                        // key-ok: the justified grid's rows are positions of a computed layout
                        key: "{plan.row}",
                        role: "row",
                        "aria-rowindex": plan.row + 1,
                        style: "position: absolute; inset-inline-start: 0; top: {plan.y:.3}px; width: 100%; height: {plan.height:.3}px;",
                        // Keyed items straight from an iterator (see `VirtualGrid`): a window
                        // move then keeps every tile's DOM node.
                        {plan.tiles.iter().map(|t| tile(&plan, *t))}
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

/// The `aria-sort` token of column `column`, or `None` for an unsorted one (or without the hook).
fn sort_token(
    column_sort: Option<Callback<usize, Option<SortDirection>>>,
    column: usize,
) -> Option<&'static str> {
    column_sort
        .and_then(|f| f(column))
        .map(SortDirection::aria_sort)
}

/// Whether row `index` is selected: `None` without the `row_selected` hook, else the answer for
/// the row's key (its identity, so the answer follows the row through a sort).
fn row_is_selected(
    row_selected: Option<Callback<String, bool>>,
    row_key: Option<Callback<usize, String>>,
    index: usize,
) -> Option<bool> {
    row_selected.map(|f| f(render_key(row_key, index)))
}

/// The row's `aria-selected` value (every row carries one once the hook is given).
fn selected_token(
    row_selected: Option<Callback<String, bool>>,
    row_key: Option<Callback<usize, String>>,
    index: usize,
) -> Option<&'static str> {
    row_is_selected(row_selected, row_key, index).map(|on| if on { "true" } else { "false" })
}

/// The attributes of a body row: its role, position, selection and style.
fn row_attributes(
    row_selected: Option<Callback<String, bool>>,
    row_key: Option<Callback<usize, String>>,
    index: usize,
    row_index: usize,
    style: String,
) -> Vec<Attribute> {
    attributes!(div {
        role: "row",
        "aria-rowindex": row_index,
        "aria-selected": selected_token(row_selected, row_key, index),
        style: style,
        ..selected_state(row_selected, row_key, index),
    })
}

/// The row's `data-state`: `selected` for a selected row, nothing otherwise.
fn selected_state(
    row_selected: Option<Callback<String, bool>>,
    row_key: Option<Callback<usize, String>>,
    index: usize,
) -> Vec<Attribute> {
    state_attrs::selected(row_is_selected(row_selected, row_key, index).unwrap_or(false))
}

/// Keeps the one-tab-stop promise when cells hold controls (APG grid, "focus the widget"
/// option): see [`CELL_CONTROLS_JS`]. Installed once per grid; it follows the DOM, so a window
/// move, a re-render or a caller's own `tabindex` cannot leave a second stop behind.
fn use_cell_controls(container_id: Signal<String>) {
    use_effect(move || {
        let eval = document::eval(CELL_CONTROLS_JS);
        let _ = eval.send((container_id.peek().clone(), CONTROL_SELECTOR));
    });
}

/// What counts as a cell's control: a natively focusable element, or one with a widget role that
/// a library control (a checkbox button, a switch) renders. A cell holds ONE (the first).
const CONTROL_SELECTOR: &str = concat!(
    "a[href], button:not([disabled]), input:not([type=hidden]):not([disabled]), ",
    "select:not([disabled]), textarea:not([disabled]), ",
    "[role=checkbox], [role=switch], [role=radio], [role=button], [role=link], ",
    "[data-grid-control]"
);

/// For every data cell that holds a control: the cell itself leaves the tab order
/// (`tabindex=-1`), the control is the tab stop (`tabindex=0`) in the active cell and `-1`
/// everywhere else, and the control is marked `data-grid-control-active` where it is the stop.
/// Runs on every DOM change under the grid (a MutationObserver, coalesced to one frame) and is
/// idempotent, so its own writes settle after one pass. Stops when the grid leaves the page.
const CELL_CONTROLS_JS: &str = r#"
    const [id, selector] = await dioxus.recv();
    const grid = document.getElementById(id);
    if (!grid) return;
    const set = (el, name, value) => {
        if (el.getAttribute(name) !== value) el.setAttribute(name, value);
    };
    let queued = false;
    const sync = () => {
        queued = false;
        if (!grid.isConnected) { observer.disconnect(); return; }
        for (const cell of grid.querySelectorAll('[data-grid-cell]:not([data-grid-cell^="header"])')) {
            const control = cell.querySelector(selector);
            if (!control) continue;
            const active = cell.getAttribute("data-grid-active") === "true";
            set(cell, "data-grid-has-control", "true");
            set(cell, "tabindex", "-1");
            set(control, "tabindex", active ? "0" : "-1");
        }
    };
    const observer = new MutationObserver(() => {
        if (!queued) { queued = true; requestAnimationFrame(sync); }
    });
    observer.observe(grid, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["tabindex", "data-grid-active"],
    });
    sync();
"#;

/// Maps a key press to a grid move, or `None` for a key the grid does not handle (which then
/// keeps its default action). The horizontal arrows go through `direction.rs`, so they mirror in
/// RTL.
fn grid_key(event: &KeyboardData, direction: Direction) -> Option<GridKey> {
    let ctrl = event.modifiers().ctrl() || event.modifiers().meta();
    key_to_grid_key(event.key(), ctrl, direction)
}

/// [`grid_key`] on a bare key and the Ctrl (or Cmd) state.
fn key_to_grid_key(key: Key, ctrl: bool, direction: Direction) -> Option<GridKey> {
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
/// the pinned header and the pinned columns on both sides (`cover`, read when the effect runs).
fn use_focus_active_cell(
    container_id: Signal<String>,
    active: impl Readable<Target = (usize, usize)> + Copy + 'static,
    focus_request: Signal<u64>,
    controls: bool,
    cover: CopyValue<Cover>,
) {
    use_effect(move || {
        if focus_request() == 0 {
            return;
        }
        let (row, column) = *active.peek();
        focus_and_reveal(
            container_id.peek().clone(),
            format!(r#"[data-grid-cell="{row}:{column}"]"#),
            if controls { CONTROL_SELECTOR } else { "" },
            *cover.read(),
        );
    });
}

/// After a keyboard move (or a re-layout that may have re-created the focused tile) has
/// rendered, focus a justified grid's active tile and scroll it into view on both axes.
fn use_focus_active_tile(
    container_id: Signal<String>,
    active: Signal<usize>,
    focus_request: Signal<u64>,
) {
    use_effect(move || {
        if focus_request() == 0 {
            return;
        }
        let index = *active.peek();
        focus_and_reveal(
            container_id.peek().clone(),
            format!(r#"[data-grid-index="{index}"]"#),
            "",
            Cover::default(),
        );
    });
}

/// Runs [`FOCUS_CELL_JS`] for the cell `selector` names inside the grid `container_id`.
///
/// With a non-empty `control_selector` (the data grid's [`CONTROL_SELECTOR`]) focus goes to the
/// cell's control when it holds one, else to the cell.
fn focus_and_reveal(container_id: String, selector: String, control_selector: &str, cover: Cover) {
    let eval = document::eval(FOCUS_CELL_JS);
    let _ = eval.send((
        container_id,
        selector,
        control_selector,
        cover,
        true,
        None::<(u32, u32)>,
    ));
}

/// Scrolls the cell `selector` names into view beside the pinned edges, without moving focus:
/// where a find-in-page match landed (the find bar keeps focus). `at` is the cell's place on the
/// canvas (its row's top, its column's start), for a cell the window does not hold yet.
fn reveal_cell(container_id: String, selector: String, cover: Cover, at: (u32, u32)) {
    let eval = document::eval(FOCUS_CELL_JS);
    let _ = eval.send((container_id, selector, "", cover, false, at));
}

/// Focuses the active cell once it is in the DOM (it is pinned, so the render that made it
/// active mounts it; with `focus` false it only waits for it) and scrolls the container just
/// enough to show it beside the pinned edges (`cover`: the header's height, the px the pinned
/// header covers, and the px the start and end columns cover).
const FOCUS_CELL_JS: &str = r#"
    const [id, selector, controlSelector, cover, focus, at] = await dioxus.recv();
    let tries = 0;
    const go = () => {
        const grid = document.getElementById(id);
        if (!grid) return;
        const cell = grid.querySelector(selector);
        if (!cell) {
            // A cell that is not mounted but whose place is known (`at`: its row's top and its
            // column's start on the canvas): scroll there once, which moves the window onto it.
            if (at && tries === 0) {
                const [y, x] = at;
                const open = grid.clientHeight - cover.top;
                grid.scrollTop = Math.max(0, cover.header + y - cover.top - open / 2);
                const pane = grid.clientWidth - cover.start - cover.end;
                const inline = Math.max(0, x - cover.start - pane / 2);
                grid.scrollLeft = getComputedStyle(grid).direction === "rtl" ? -inline : inline;
            }
            if (tries++ < 30) requestAnimationFrame(go);
            return;
        }
        // focus-ok: the grid pattern's roving tab stop; the key press already holds focus inside the grid
        if (focus) (controlSelector && cell.querySelector(controlSelector) || cell).focus({ preventScroll: true });
        const box = grid.getBoundingClientRect();
        const rect = cell.getBoundingClientRect();
        const top = box.top + grid.clientTop + cover.top;
        const bottom = box.top + grid.clientTop + grid.clientHeight;
        if (rect.top < top) grid.scrollTop -= top - rect.top;
        else if (rect.bottom > bottom) grid.scrollTop += rect.bottom - bottom;
        if (cell.hasAttribute("data-grid-pinned")) return;
        const rtl = getComputedStyle(grid).direction === "rtl";
        const left = box.left + grid.clientLeft + (rtl ? cover.end : cover.start);
        const right = box.left + grid.clientLeft + grid.clientWidth - (rtl ? cover.start : cover.end);
        if (rect.left < left) grid.scrollLeft -= left - rect.left;
        else if (rect.right > right) grid.scrollLeft += rect.right - right;
    };
    go();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_horizontal_arrows_mirror_in_rtl_and_ctrl_picks_the_grid_ends() {
        let map =
            |key: Key, ctrl: bool, direction: Direction| key_to_grid_key(key, ctrl, direction);
        assert_eq!(
            map(Key::ArrowRight, false, Direction::Ltr),
            Some(GridKey::Next)
        );
        assert_eq!(
            map(Key::ArrowLeft, false, Direction::Ltr),
            Some(GridKey::Prev)
        );
        assert_eq!(
            map(Key::ArrowLeft, false, Direction::Rtl),
            Some(GridKey::Next)
        );
        assert_eq!(
            map(Key::ArrowRight, false, Direction::Rtl),
            Some(GridKey::Prev)
        );
        assert_eq!(
            map(Key::ArrowDown, false, Direction::Rtl),
            Some(GridKey::Down)
        );
        assert_eq!(
            map(Key::Home, false, Direction::Ltr),
            Some(GridKey::RowStart)
        );
        assert_eq!(map(Key::Home, true, Direction::Ltr), Some(GridKey::First));
        assert_eq!(map(Key::End, true, Direction::Rtl), Some(GridKey::Last));
        assert_eq!(map(Key::Enter, false, Direction::Ltr), None);
    }

    fn pinning(columns: usize, start: usize, end: usize) -> Pinning {
        // Column `c` is `10 * (c + 1)` px wide, so every offset is a distinct sum.
        Pinning::new(columns, start, end, |column| 10 * (column as u32 + 1))
    }

    #[test]
    fn start_columns_stack_from_the_inline_start_by_cumulative_width() {
        let pins = pinning(8, 3, 0);
        assert_eq!((pins.start_count(), pins.end_count()), (3, 0));
        assert_eq!(pins.scrolling_count(), 5);
        assert_eq!(pins.start_width(), 10 + 20 + 30);
        let at = |column| pins.at(column).unwrap();
        assert_eq!((at(0).offset, at(0).width), (0, 10));
        assert_eq!((at(1).offset, at(1).width), (10, 20));
        assert_eq!((at(2).offset, at(2).width), (30, 30));
        assert!(!at(0).edge && !at(1).edge && at(2).edge);
        assert!(pins.at(0).is_some_and(|pin| pin.side == PinSide::Start));
        assert_eq!(pins.at(3), None);
    }

    #[test]
    fn end_columns_stack_from_the_inline_end_by_cumulative_width() {
        let pins = pinning(8, 0, 3);
        assert_eq!(pins.scrolling_count(), 5);
        assert_eq!(pins.end_width(), 60 + 70 + 80);
        let at = |column| pins.at(column).unwrap();
        // The last column hugs the edge; the one before it sits one column further in.
        assert_eq!((at(7).offset, at(7).width), (0, 80));
        assert_eq!((at(6).offset, at(6).width), (80, 70));
        assert_eq!((at(5).offset, at(5).width), (150, 60));
        assert!(at(5).edge && !at(6).edge && !at(7).edge);
        assert!(pins.at(7).is_some_and(|pin| pin.side == PinSide::End));
        assert_eq!(pins.at(4), None);
        assert_eq!(pins.at(8), None);
    }

    #[test]
    fn both_sides_leave_the_columns_between_them_scrolling() {
        let pins = pinning(10, 2, 1);
        assert_eq!(pins.scrolling_count(), 7);
        assert_eq!((pins.start_width(), pins.end_width()), (30, 100));
        let pinned: Vec<usize> = (0..10).filter(|c| pins.at(*c).is_some()).collect();
        assert_eq!(pinned, vec![0, 1, 9]);
    }

    #[test]
    fn the_counts_clamp_to_the_column_count_start_side_first() {
        let pins = pinning(4, 3, 5);
        assert_eq!((pins.start_count(), pins.end_count()), (3, 1));
        assert_eq!(pins.scrolling_count(), 0);
        let pins = pinning(2, 9, 9);
        assert_eq!((pins.start_count(), pins.end_count()), (2, 0));
        let pins = pinning(0, 1, 1);
        assert_eq!(pins.scrolling_count(), 0);
        assert_eq!(pins.at(0), None);
    }

    #[test]
    fn a_pinned_edge_cell_of_each_side_carries_the_separator() {
        let pins = pinning(6, 2, 2);
        let edges: Vec<usize> = (0..6)
            .filter(|c| pins.at(*c).is_some_and(|pin| pin.edge))
            .collect();
        assert_eq!(edges, vec![1, 4]);
    }

    #[test]
    fn pinned_cells_are_sticky_at_logical_offsets() {
        let pins = pinning(6, 2, 2);
        let style = |column| pinned_cell_style(pins.at(column).unwrap());
        assert!(style(1).contains("inset-inline-start: 10px;"));
        assert!(!style(1).contains("margin-inline-start"));
        // The innermost end column takes the free space, so the end columns reach the edge.
        assert!(style(4).contains("inset-inline-end: 60px; margin-inline-start: auto;"));
        assert!(style(5).contains("inset-inline-end: 0px;") && !style(5).contains("margin"));
        for column in [0, 1, 4, 5] {
            assert!(style(column).starts_with("position: sticky;"));
            assert!(!style(column).contains("left") && !style(column).contains("right"));
        }
    }

    #[test]
    fn the_cover_is_what_the_pinned_edges_take() {
        let pins = pinning(10, 2, 1);
        let pinned = Cover::new(40, true, &pins);
        assert_eq!(
            pinned,
            Cover {
                header: 40,
                top: 40,
                start: 30,
                end: 100
            }
        );
        assert_eq!(
            pinned.pins(&pins),
            Pins {
                start_columns: 2,
                top: 40,
                lead: 0,
                inline: 130
            }
        );
        // A header that scrolls away covers nothing and shifts the body down instead.
        let scrolling = Cover::new(40, false, &pins);
        assert_eq!(scrolling.top, 0);
        assert_eq!(
            scrolling.pins(&pins),
            Pins {
                start_columns: 2,
                top: 0,
                lead: 40,
                inline: 130
            }
        );
    }

    #[test]
    fn the_header_row_is_sticky_only_when_pinned() {
        assert!(header_row_style(true, 500, 40).starts_with("position: sticky; top: 0;"));
        assert!(header_row_style(false, 500, 40).starts_with("position: relative;"));
    }

    #[test]
    fn index_and_cell_round_trip() {
        assert_eq!(index_to_cell(0, 4), (0, 0));
        assert_eq!(index_to_cell(9, 4), (2, 1));
        assert_eq!(index_to_cell(9, 0), (9, 0));
    }
}
