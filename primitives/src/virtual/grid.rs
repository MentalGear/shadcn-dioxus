//! The second axis: windowing a grid of rows AND columns, and moving through it by keyboard.
//!
//! A grid is two one-axis engines. [`GridEngine`] owns one [`Engine`] for the rows and one for
//! the scrolling columns, so each axis keeps everything the list engine already does (a Fenwick
//! [`super::size_index`] for positions, the hysteresis that re-renders only once per
//! `buffer / 2` items, the canvas size frozen while a scroll runs) and the 2-D window is simply
//! the product of the two ranges. Nothing here is grid-specific arithmetic on top of the axes
//! except three things:
//!
//! - **pinned edges** (row 388): a pinned header row and pinned columns at both inline edges
//!   cover part of the viewport, so the rows axis sees `height - header` and the columns axis
//!   sees `width - covered inline width`; the pinned columns are not part of the scrolling axis
//!   at all (column `c` of the data is item `c - start_columns` of the columns engine, and the
//!   end columns are past its last item). A header that is not pinned covers nothing but
//!   precedes the body, so the rows axis starts that much later;
//! - **the pinned cell**: the cell that holds keyboard focus is always rendered, even far
//!   outside the window (a `Ctrl+End` jump, or a reader who scrolled away), see
//!   [`with_pinned`](super::pin::with_pinned). A cell that is unmounted while it has focus loses it, and the grid pattern
//!   needs exactly one cell in the tab order;
//! - **navigation**: [`GridShape::navigate`] is the WAI-ARIA grid pattern's key map (arrows,
//!   Home/End, Ctrl+Home/End, PageUp/PageDown), with no wrapping at an edge. Horizontal keys
//!   arrive here already resolved to `Prev`/`Next` by `direction.rs`, so RTL needs nothing.
//!
//! The wrapping "gallery" layout (a fixed number of columns, items flowing into rows) windows
//! only the rows: it uses one [`Engine`] over `ceil(count / columns)` rows, plus [`with_pinned`](super::pin::with_pinned)
//! and [`GridShape::wrapping`] from this module.

use super::pin::with_pinned;
use super::scroll_to::ScrollAlign;
use super::size_index::SizeEstimates;
use super::types::{ScrollReport, VirtualItem};
use super::virtualizer::Engine;

/// A navigation intent from the keyboard, already direction-resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GridKey {
    /// `ArrowUp`.
    Up,
    /// `ArrowDown`.
    Down,
    /// The inline-start neighbour (`ArrowLeft` in LTR, `ArrowRight` in RTL).
    Prev,
    /// The inline-end neighbour.
    Next,
    /// `Home`: the first cell of the row.
    RowStart,
    /// `End`: the last cell of the row.
    RowEnd,
    /// `Ctrl+Home`: the first cell of the grid.
    First,
    /// `Ctrl+End`: the last cell of the grid.
    Last,
    /// `PageUp`: a viewport of rows up, same column.
    PageUp,
    /// `PageDown`: a viewport of rows down, same column.
    PageDown,
}

/// The extent of a grid for navigation: `rows x columns`, where the last row may be short
/// (`count` items flowing into rows of `columns`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridShape {
    rows: usize,
    columns: usize,
    /// The number of cells; `rows * columns` for a full grid, fewer when the last row is short.
    count: usize,
}

impl GridShape {
    /// A full `rows x columns` grid.
    pub(crate) fn full(rows: usize, columns: usize) -> Self {
        Self {
            rows,
            columns,
            count: rows.saturating_mul(columns),
        }
    }

    /// `count` items flowing into rows of `columns` (`columns` is at least 1).
    pub(crate) fn wrapping(count: usize, columns: usize) -> Self {
        let columns = columns.max(1);
        Self {
            rows: count.div_ceil(columns),
            columns,
            count,
        }
    }

    pub(crate) fn rows(&self) -> usize {
        self.rows
    }

    /// How many cells row `row` has: `columns`, except for a short last row.
    pub(crate) fn row_len(&self, row: usize) -> usize {
        if row >= self.rows {
            return 0;
        }
        (self.count - row * self.columns).min(self.columns)
    }

    /// `cell` moved into the grid: the row clamped to the last row, the column to that row's
    /// last cell. `None` for an empty grid.
    pub(crate) fn clamp(&self, (row, column): (usize, usize)) -> Option<(usize, usize)> {
        if self.count == 0 || self.columns == 0 {
            return None;
        }
        let row = row.min(self.rows - 1);
        Some((row, column.min(self.row_len(row) - 1)))
    }

    /// Where `key` moves the focus from `from`, given that a page is `page` rows. Never wraps
    /// at an edge (the grid pattern's default): at the last column `Next` stays put.
    pub(crate) fn navigate(
        &self,
        from: (usize, usize),
        key: GridKey,
        page: usize,
    ) -> (usize, usize) {
        let Some((row, column)) = self.clamp(from) else {
            return from;
        };
        let last_row = self.rows - 1;
        let page = page.max(1);
        let target = match key {
            GridKey::Up => (row.saturating_sub(1), column),
            GridKey::Down => ((row + 1).min(last_row), column),
            GridKey::Prev => (row, column.saturating_sub(1)),
            GridKey::Next => (row, column + 1),
            GridKey::RowStart => (row, 0),
            GridKey::RowEnd => (row, usize::MAX),
            GridKey::First => (0, 0),
            GridKey::Last => (last_row, usize::MAX),
            GridKey::PageUp => (row.saturating_sub(page), column),
            GridKey::PageDown => (row.saturating_add(page).min(last_row), column),
        };
        self.clamp(target).unwrap_or(from)
    }
}

/// The rows and columns to draw now.
#[derive(Debug)]
pub(crate) struct GridPlan {
    /// Mounted rows (with the pinned row), `start` relative to the top of the body.
    pub(crate) rows: Vec<VirtualItem>,
    /// Mounted scrolling columns (with the pinned column). `index` is the DATA column, `start`
    /// is relative to the inline start of the scrolling pane (after the pinned start columns).
    pub(crate) columns: Vec<VirtualItem>,
    /// The height of the body (below the header).
    pub(crate) body_height: u32,
    /// The width of the scrolling pane (between the pinned columns).
    pub(crate) pane_width: u32,
}

/// What the grid pins, as the engine needs it (row 388).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pins {
    /// How many leading data columns are pinned at the inline start; they are not in the
    /// columns engine (the end columns are simply past its last item).
    pub(crate) start_columns: usize,
    /// The px the pinned header covers at the top of the viewport (0 for an unpinned header).
    pub(crate) top: u32,
    /// The px of header that sit above the body but scroll away with it (0 for a pinned one).
    pub(crate) lead: u32,
    /// The px the pinned columns cover at both inline edges together.
    pub(crate) inline: u32,
}

/// The 2-D windowing engine: one [`Engine`] per axis. See the module doc.
pub(crate) struct GridEngine {
    rows: Engine,
    columns: Engine,
    pins: Pins,
}

impl GridEngine {
    pub(crate) fn new() -> Self {
        Self {
            rows: Engine::new(),
            columns: Engine::new(),
            pins: Pins::default(),
        }
    }

    /// Adopt the current geometry: row heights, the sizes of the SCROLLING columns only (the
    /// pinned ones are not in the engine) and what is pinned. `O(1)` when nothing changed (the
    /// estimates are compared by identity, as the list engine does).
    pub(crate) fn sync(
        &mut self,
        rows: &SizeEstimates,
        scrolling_columns: &SizeEstimates,
        pins: Pins,
    ) {
        self.rows.sync(rows);
        self.columns.sync(scrolling_columns);
        self.pins = pins;
    }

    /// Handle a scroll report. Both axes see only the part of the viewport the pinned edges do
    /// not cover, and the rows axis starts where the body does.
    pub(crate) fn scroll(&mut self, scroll: ScrollReport) {
        self.rows.scroll(
            scroll.top.saturating_sub(self.pins.lead),
            scroll.height.saturating_sub(self.pins.top),
            scroll.is_scrolling,
        );
        self.columns.scroll(
            scroll.left,
            scroll.width.saturating_sub(self.pins.inline),
            scroll.is_scrolling,
        );
    }

    /// Whether a render now would draw a different window on either axis.
    pub(crate) fn needs_render(&mut self) -> bool {
        // Both, not `||`: each axis's refresh must run so its hysteresis state is current.
        let rows = self.rows.needs_render();
        let columns = self.columns.needs_render();
        rows || columns
    }

    /// How many rows the viewport shows: the PageUp/PageDown step.
    pub(crate) fn page_rows(&self) -> usize {
        self.rows.visible_len()
    }

    /// Whether the container has reported its size (a scroll target needs it).
    pub(crate) fn viewport_known(&self) -> bool {
        self.rows.viewport_known()
    }

    /// The container scroll offsets `(top, left)` that put the cell `(row, column)` at `align`,
    /// `None` on an axis that stays (no `column`, or a pinned column, which is always in view).
    /// A row past the last goes to the last. The engine's offsets are set to the targets, as a
    /// scroll report would, so the render that follows mounts the window there.
    pub(crate) fn align_cell(
        &mut self,
        row: usize,
        column: Option<usize>,
        align: ScrollAlign,
    ) -> (Option<u32>, Option<u32>) {
        let top = self
            .rows
            .item_count()
            .checked_sub(1)
            .and_then(|last| self.rows.align_target(row.min(last), align))
            .map(|target| {
                self.rows.set_scroll_offset(target);
                // Offset 0 is the very top, header and all.
                if target == 0 {
                    0
                } else {
                    target.saturating_add(self.pins.lead)
                }
            });
        let left = column
            .and_then(|column| column.checked_sub(self.pins.start_columns))
            .filter(|&scrolling| scrolling < self.columns.item_count())
            .and_then(|scrolling| self.columns.align_target(scrolling, align))
            .inspect(|&target| self.columns.set_scroll_offset(target));
        (top, left)
    }

    /// The window to draw, with the `pinned` cell (data row, data column) added on each axis
    /// where it is outside the window. A pinned column is always drawn anyway (it is no item of
    /// the columns engine, so a pinned active cell adds nothing to the scrolling columns).
    pub(crate) fn plan(
        &mut self,
        row_buffer: usize,
        column_buffer: usize,
        pinned: Option<(usize, usize)>,
    ) -> GridPlan {
        let rows = self.rows.plan(row_buffer);
        let columns = self.columns.plan(column_buffer);
        let start = self.pins.start_columns;
        let pinned_row = pinned.and_then(|(row, _)| self.rows.item(row));
        let pinned_column = pinned
            .and_then(|(_, column)| column.checked_sub(start))
            .and_then(|scrolling| self.columns.item(scrolling));
        let to_data = |item: VirtualItem| {
            VirtualItem::new(
                item.key() + start,
                item.index() + start,
                item.start(),
                item.size(),
            )
        };
        GridPlan {
            rows: with_pinned(rows.items, pinned_row),
            columns: with_pinned(columns.items, pinned_column)
                .into_iter()
                .map(to_data)
                .collect(),
            body_height: rows.canvas_height,
            pane_width: columns.canvas_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(count: usize, size: u32) -> SizeEstimates {
        let estimate: &dyn Fn(usize) -> u32 = &move |_| size;
        SizeEstimates::evaluate(count, Some(estimate))
    }

    fn indices(items: &[VirtualItem]) -> Vec<usize> {
        items.iter().map(VirtualItem::index).collect()
    }

    /// The demo's data grid: 10,000 rows of 36px, 50 columns of 120px, a 40px pinned header and
    /// `start` pinned columns at the inline start.
    fn data_grid(start: usize) -> GridEngine {
        pinned_grid(start, 0)
    }

    /// The same grid with `start` and `end` pinned columns.
    fn pinned_grid(start: usize, end: usize) -> GridEngine {
        let mut grid = GridEngine::new();
        grid.sync(
            &uniform(10_000, 36),
            &uniform(50 - start - end, 120),
            Pins {
                start_columns: start,
                top: 40,
                lead: 0,
                inline: 120 * (start + end) as u32,
            },
        );
        grid
    }

    fn at(grid: &mut GridEngine, top: u32, left: u32) {
        grid.scroll(ScrollReport {
            top,
            left,
            width: 800,
            height: 640,
            is_scrolling: false,
        });
    }

    #[test]
    fn nothing_is_windowed_before_the_viewport_is_known_but_the_canvas_is() {
        let mut grid = data_grid(0);
        let plan = grid.plan(6, 3, None);
        assert!(plan.rows.is_empty() && plan.columns.is_empty());
        assert_eq!(plan.body_height, 360_000);
        assert_eq!(plan.pane_width, 6_000);
        // The pinned (focused) cell is drawn even then: the tab stop exists from the first frame.
        let plan = grid.plan(6, 3, Some((0, 0)));
        assert_eq!(
            (indices(&plan.rows), indices(&plan.columns)),
            (vec![0], vec![0])
        );
    }

    #[test]
    fn the_window_is_the_product_of_both_axes_and_tiny_against_the_grid() {
        let mut grid = data_grid(0);
        at(&mut grid, 0, 0);
        let plan = grid.plan(6, 3, None);
        // Rows: (640 - 40) / 36 -> rows 0..=16 visible, + the neighbour + 6 buffer.
        assert_eq!(indices(&plan.rows), (0..=23).collect::<Vec<_>>());
        // Columns: 800 / 120 -> columns 0..=6 visible, + the neighbour + 3 buffer.
        assert_eq!(indices(&plan.columns), (0..=10).collect::<Vec<_>>());
        let cells = plan.rows.len() * plan.columns.len();
        assert!(cells < 300, "{cells} cells mounted of 500,000");
    }

    #[test]
    fn a_far_scroll_windows_both_axes_around_the_far_cell() {
        let mut grid = data_grid(0);
        // Row 5,000 starts at 180,000px; column 30 at 3,600px.
        at(&mut grid, 180_000, 3_600);
        let plan = grid.plan(6, 3, None);
        let rows = indices(&plan.rows);
        let columns = indices(&plan.columns);
        assert!(rows.contains(&5_000) && rows.len() < 40, "{rows:?}");
        assert!(columns.contains(&30) && columns.len() < 16, "{columns:?}");
        let row = plan.rows.iter().find(|r| r.index() == 5_000).unwrap();
        assert_eq!(row.start(), 180_000);
        let column = plan.columns.iter().find(|c| c.index() == 30).unwrap();
        assert_eq!(column.start(), 3_600);
    }

    #[test]
    fn the_pinned_column_is_not_windowed_and_the_pane_starts_after_it() {
        let mut grid = data_grid(1);
        at(&mut grid, 0, 0);
        let plan = grid.plan(6, 3, None);
        // Data column 1 is the pane's first column, at pane offset 0; the pane sees 800 - 120.
        assert_eq!(plan.columns[0].index(), 1);
        assert_eq!(plan.columns[0].start(), 0);
        // 680 / 120 -> pane columns 0..=5 visible (data 1..=6), + neighbour + 3 buffer.
        assert_eq!(indices(&plan.columns), (1..=10).collect::<Vec<_>>());
        assert_eq!(plan.pane_width, 49 * 120);
        // Scrolled to the end, the last data column is mounted and nothing past it.
        at(&mut grid, 0, 49 * 120 - 680);
        let plan = grid.plan(6, 3, None);
        assert_eq!(*indices(&plan.columns).last().unwrap(), 49);
    }

    #[test]
    fn end_pinned_columns_leave_the_pane_and_shrink_the_columns_viewport() {
        let mut grid = pinned_grid(1, 2);
        at(&mut grid, 0, 0);
        let plan = grid.plan(6, 3, None);
        // The pane holds data columns 1..=47; it sees 800 - 3 * 120 = 440px (4 columns).
        assert_eq!(plan.columns[0].index(), 1);
        assert_eq!(plan.pane_width, 47 * 120);
        assert_eq!(indices(&plan.columns), (1..=8).collect::<Vec<_>>());
        // Scrolled to the pane's end, the last scrolling column is data column 47, not 49.
        at(&mut grid, 0, 47 * 120 - 440);
        let plan = grid.plan(6, 3, None);
        assert_eq!(*indices(&plan.columns).last().unwrap(), 47);
        // An active cell in an end column adds nothing to the scrolling columns.
        let plan = grid.plan(6, 3, Some((3, 49)));
        assert_eq!(*indices(&plan.columns).last().unwrap(), 47);
    }

    #[test]
    fn a_header_that_scrolls_away_moves_the_rows_start_and_covers_nothing() {
        let mut grid = GridEngine::new();
        grid.sync(
            &uniform(10_000, 36),
            &uniform(50, 120),
            Pins {
                lead: 40,
                ..Pins::default()
            },
        );
        // 400px down, the body is 360px down; the viewport is all body (no cover).
        at(&mut grid, 400, 0);
        let plan = grid.plan(0, 0, None);
        assert_eq!(plan.rows[0].index(), 10);
        assert_eq!(grid.page_rows(), 18);
    }

    #[test]
    fn align_cell_targets_both_axes_past_the_pinned_edges() {
        let mut grid = data_grid(1);
        at(&mut grid, 0, 0);
        assert!(grid.viewport_known());
        let start = ScrollAlign::Start;
        // Row 5,000 is 180,000px down; column 30 is the pane's 29th item, 3,480px along.
        assert_eq!(grid.align_cell(5_000, Some(30), start), (Some(180_000), Some(3_480)));
        // A pinned column is always in view: only the rows move. No column, no horizontal move.
        assert_eq!(grid.align_cell(10, Some(0), start), (Some(360), None));
        assert_eq!(grid.align_cell(10, None, start), (Some(360), None));
        // Past the last row goes to the last, clamped to the scroll range (600px of body).
        assert_eq!(grid.align_cell(99_999, None, start).0, Some(10_000 * 36 - 600));
        // The engine now holds the target, so `Nearest` from there is a no-op.
        assert_eq!(
            grid.align_cell(10_000, None, ScrollAlign::Nearest).0,
            Some(10_000 * 36 - 600)
        );
    }

    #[test]
    fn align_cell_adds_the_header_that_scrolls_away_except_at_the_very_top() {
        let mut grid = GridEngine::new();
        grid.sync(
            &uniform(10_000, 36),
            &uniform(50, 120),
            Pins {
                lead: 40,
                ..Pins::default()
            },
        );
        at(&mut grid, 0, 0);
        assert_eq!(grid.align_cell(10, None, ScrollAlign::Start).0, Some(400));
        assert_eq!(grid.align_cell(0, None, ScrollAlign::Start).0, Some(0));
    }

    #[test]
    fn the_pinned_header_shrinks_the_rows_viewport() {
        let mut with = data_grid(0);
        let mut without = GridEngine::new();
        without.sync(&uniform(10_000, 36), &uniform(50, 120), Pins::default());
        at(&mut with, 0, 0);
        at(&mut without, 0, 0);
        assert_eq!(with.page_rows(), 17); // 600px of body: rows 0..=16
        assert_eq!(without.page_rows(), 18); // 640px: rows 0..=17 (17 ends exactly at 648)
    }

    #[test]
    fn the_pinned_cell_is_drawn_far_outside_the_window_once() {
        let mut grid = data_grid(1);
        at(&mut grid, 0, 0);
        let plan = grid.plan(6, 3, Some((9_999, 49)));
        assert_eq!(*indices(&plan.rows).last().unwrap(), 9_999);
        assert_eq!(*indices(&plan.columns).last().unwrap(), 49);
        let row = plan.rows.last().unwrap();
        assert_eq!(row.start(), 9_999 * 36);
        let column = plan.columns.last().unwrap();
        assert_eq!(column.start(), 48 * 120);
        // Inside the window it is not added twice.
        let plan = grid.plan(6, 3, Some((3, 2)));
        assert_eq!(indices(&plan.rows), (0..=23).collect::<Vec<_>>());
        assert_eq!(indices(&plan.columns), (1..=10).collect::<Vec<_>>());
        // A pinned start column adds nothing to the scrolling columns.
        let plan = grid.plan(6, 3, Some((3, 0)));
        assert_eq!(indices(&plan.columns), (1..=10).collect::<Vec<_>>());
    }

    #[test]
    fn a_small_scroll_on_either_axis_needs_no_render() {
        let mut grid = data_grid(0);
        at(&mut grid, 0, 0);
        grid.plan(6, 3, None);
        at(&mut grid, 36, 60);
        assert!(!grid.needs_render());
        // Far enough that the visible rows come within buffer / 2 of the edge: a render.
        at(&mut grid, 36 * 6, 60);
        assert!(grid.needs_render());
        grid.plan(6, 3, None);
        at(&mut grid, 36 * 6, 120 * 3);
        assert!(grid.needs_render());
    }

    #[test]
    fn arrows_move_one_cell_and_stop_at_the_edges() {
        let grid = GridShape::full(100, 5);
        assert_eq!(grid.navigate((0, 0), GridKey::Up, 10), (0, 0));
        assert_eq!(grid.navigate((0, 0), GridKey::Prev, 10), (0, 0));
        assert_eq!(grid.navigate((0, 0), GridKey::Down, 10), (1, 0));
        assert_eq!(grid.navigate((0, 0), GridKey::Next, 10), (0, 1));
        assert_eq!(grid.navigate((0, 4), GridKey::Next, 10), (0, 4));
        assert_eq!(grid.navigate((99, 2), GridKey::Down, 10), (99, 2));
    }

    #[test]
    fn home_end_and_ctrl_home_end() {
        let grid = GridShape::full(100, 5);
        assert_eq!(grid.navigate((7, 3), GridKey::RowStart, 10), (7, 0));
        assert_eq!(grid.navigate((7, 1), GridKey::RowEnd, 10), (7, 4));
        assert_eq!(grid.navigate((7, 3), GridKey::First, 10), (0, 0));
        assert_eq!(grid.navigate((7, 3), GridKey::Last, 10), (99, 4));
    }

    #[test]
    fn page_keys_move_a_viewport_of_rows_and_clamp() {
        let grid = GridShape::full(100, 5);
        assert_eq!(grid.navigate((50, 2), GridKey::PageDown, 17), (67, 2));
        assert_eq!(grid.navigate((50, 2), GridKey::PageUp, 17), (33, 2));
        assert_eq!(grid.navigate((95, 2), GridKey::PageDown, 17), (99, 2));
        assert_eq!(grid.navigate((5, 2), GridKey::PageUp, 17), (0, 2));
        // A page is at least one row, even before the viewport is known.
        assert_eq!(grid.navigate((5, 2), GridKey::PageDown, 0), (6, 2));
    }

    #[test]
    fn a_short_last_row_clamps_the_column() {
        // 23 items in rows of 5: the last row (4) holds items 20, 21, 22.
        let grid = GridShape::wrapping(23, 5);
        assert_eq!(grid.rows(), 5);
        assert_eq!(grid.row_len(4), 3);
        assert_eq!(grid.row_len(5), 0);
        assert_eq!(grid.navigate((3, 4), GridKey::Down, 3), (4, 2));
        assert_eq!(grid.navigate((4, 0), GridKey::RowEnd, 3), (4, 2));
        assert_eq!(grid.navigate((0, 4), GridKey::Last, 3), (4, 2));
        assert_eq!(grid.navigate((4, 2), GridKey::Up, 3), (3, 2));
        // A stale position (the count shrank) is clamped into the grid first.
        assert_eq!(grid.navigate((40, 4), GridKey::Up, 3), (3, 2));
    }

    #[test]
    fn an_empty_grid_goes_nowhere() {
        let grid = GridShape::wrapping(0, 4);
        assert_eq!(grid.clamp((0, 0)), None);
        assert_eq!(grid.navigate((0, 0), GridKey::Down, 3), (0, 0));
        assert_eq!(GridShape::full(10, 0).clamp((1, 1)), None);
    }
}
