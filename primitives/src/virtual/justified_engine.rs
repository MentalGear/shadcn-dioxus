//! The engine behind `VirtualJustifiedGrid`: the justified-rows layout
//! ([`super::layout::justified`]) made a windowed, keyboard-navigable grid.
//!
//! Plain Rust with no Dioxus types, like [`super::Engine`] and [`super::GridEngine`]:
//!
//! - [`JustifiedEngine`] holds the layout (recomputed only when the aspects or the geometry
//!   change), the last scroll report and the band of the canvas the last render drew. It
//!   implements the bridge's `Windowing`, so a scroll report renders only once the viewport (plus
//!   half the overscan) leaves that band, and [`JustifiedEngine::plan`] returns the rows that meet
//!   the band, each with only the tiles that meet it horizontally (a canvas wider than the
//!   viewport windows both axes), plus the active tile wherever it is (via `pin.rs`).
//! - [`navigate`] is the keyboard model: the reading-order neighbours for the horizontal keys
//!   (so `Prev` from a row's first tile reaches the previous row's last one), and spatial
//!   `Up`/`Down`/`PageUp`/`PageDown` onto the tile of the target row whose horizontal centre is
//!   closest to the current tile's.
//!
//! Everything is in layout space: `x` is measured from the inline start. The component places
//! tiles with `inset-inline-start`, and the bridge reports `|scrollLeft|`, so a right-to-left page
//! mirrors the drawing and the band alike and nothing here flips. The spatial moves are invariant
//! under that mirror (`Tile::mirrored` is an isometry), which the tests check.

use std::ops::Range;

use super::grid::GridKey;
use super::layout::justified::{justified_layout, JustifiedLayout, JustifiedOptions, Tile};
use super::layout::partition_point;
use super::pin::pin_sorted;
use super::scroll_to::{align_offset, ScrollAlign};
use super::types::ScrollReport;

/// The viewport the engine assumes before the first scroll report (the server render and the
/// first client render, which must agree).
const DEFAULT_VIEWPORT: ScrollReport = ScrollReport {
    top: 0,
    left: 0,
    width: 800,
    height: 600,
    is_scrolling: false,
};

/// One row to draw: its geometry and the tiles of it to mount, in index order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RowPlan {
    /// The row's index in the layout.
    pub row: usize,
    /// The row's top edge, px.
    pub y: f64,
    /// The row's height, px.
    pub height: f64,
    /// The index of the row's first tile (so a tile's column is `tile.index - start`).
    pub start: usize,
    /// The mounted tiles of the row.
    pub tiles: Vec<Tile>,
}

/// The part of the canvas a render drew, and for which layout.
#[derive(Clone, Debug, PartialEq)]
struct Band {
    x: Range<f64>,
    y: Range<f64>,
    generation: u64,
}

/// See the module documentation.
#[derive(Clone, Debug)]
pub(crate) struct JustifiedEngine {
    aspects: Vec<f64>,
    options: Option<JustifiedOptions>,
    layout: JustifiedLayout,
    /// Bumped on every re-layout, so a drawn band of an older layout is never reused.
    generation: u64,
    viewport: Option<ScrollReport>,
    drawn: Option<Band>,
    overscan: f64,
}

impl Default for JustifiedEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JustifiedEngine {
    pub(crate) fn new() -> Self {
        Self {
            aspects: Vec::new(),
            options: None,
            layout: justified_layout(&[], &JustifiedOptions::new(0.0, 0.0, 0.0)),
            generation: 0,
            viewport: None,
            drawn: None,
            overscan: 0.0,
        }
    }

    /// Lays out `aspects` with `options` (`None` while the width is not known yet: the empty
    /// layout), unless both are what the current layout was made from. Returns whether the
    /// layout changed.
    pub(crate) fn sync(&mut self, aspects: &[f64], options: Option<JustifiedOptions>) -> bool {
        if self.options == options && self.aspects == aspects {
            return false;
        }
        self.aspects.clear();
        self.aspects.extend_from_slice(aspects);
        self.options = options;
        self.layout = match options {
            Some(options) => justified_layout(aspects, &options),
            None => justified_layout(&[], &JustifiedOptions::new(0.0, 0.0, 0.0)),
        };
        self.generation = self.generation.wrapping_add(1);
        true
    }

    /// The current layout.
    pub(crate) fn layout(&self) -> &JustifiedLayout {
        &self.layout
    }

    /// Records a scroll report.
    pub(crate) fn scroll(&mut self, report: ScrollReport) {
        self.viewport = Some(report);
    }

    /// The last reported viewport, or the default one before the first report.
    pub(crate) fn viewport(&self) -> ScrollReport {
        self.viewport.unwrap_or(DEFAULT_VIEWPORT)
    }

    /// The container width the bridge measured, `None` before the first report. A fluid grid
    /// lays out for it.
    pub(crate) fn measured_width(&self) -> Option<f64> {
        self.viewport.map(|v| f64::from(v.width))
    }

    /// Overrides the recorded scroll offset (after the component wrote a corrected `scrollTop`).
    pub(crate) fn set_top(&mut self, top: u32) {
        if let Some(viewport) = &mut self.viewport {
            viewport.top = top;
        }
    }

    /// Whether there is a layout to scroll in: the container has reported its size and there are
    /// tiles.
    pub(crate) fn has_layout(&self) -> bool {
        self.viewport.is_some() && !self.layout.tiles.is_empty()
    }

    /// The `scrollTop` that puts the row holding tile `index` (the last tile when `index` is
    /// past the end) at `align`, clamped to the canvas, and records it as the engine's offset.
    /// `None` without a layout. Vertical only: a row is the unit, whatever its width.
    pub(crate) fn align_tile(&mut self, index: usize, align: ScrollAlign) -> Option<u32> {
        let last = self.layout.tiles.len().checked_sub(1)?;
        let row = self.layout.rows[self.layout.tiles[index.min(last)].row];
        let viewport = self.viewport();
        let px = |value: f64| value.round().clamp(0.0, f64::from(u32::MAX)) as u32;
        let target = align_offset(
            px(row.y),
            px(row.height),
            viewport.top,
            viewport.height,
            px(self.layout.total_height),
            align,
        );
        self.set_top(target);
        Some(target)
    }

    /// The first tile of the row at the top of the viewport: what a re-layout keeps in place.
    pub(crate) fn anchor(&self) -> Option<usize> {
        let row = self.layout.row_at(f64::from(self.viewport().top))?;
        Some(self.layout.rows[row].start)
    }

    /// The top edge of the row now holding tile `index`.
    pub(crate) fn tile_top(&self, index: usize) -> Option<f64> {
        self.layout.tiles.get(index).map(|tile| tile.y)
    }

    /// The canvas band a render now must draw: the viewport widened by `overscan` on every side,
    /// clamped to the canvas.
    fn band(&self, overscan: f64) -> (Range<f64>, Range<f64>) {
        let v = self.viewport();
        let (top, left) = (f64::from(v.top), f64::from(v.left));
        let (width, height) = (f64::from(v.width), f64::from(v.height));
        let canvas_width = self.layout.container_width.max(0.0);
        let canvas_height = self.layout.total_height.max(0.0);
        let clamp = |lo: f64, hi: f64, max: f64| lo.clamp(0.0, max)..hi.clamp(0.0, max);
        (
            clamp(left - overscan, left + width + overscan, canvas_width),
            clamp(top - overscan, top + height + overscan, canvas_height),
        )
    }

    /// Whether rendering now would draw something different from the last plan: a new layout,
    /// or a viewport that came within half the overscan of the drawn band's edge.
    pub(crate) fn needs_render(&mut self) -> bool {
        let Some(drawn) = &self.drawn else {
            return true;
        };
        if drawn.generation != self.generation {
            return true;
        }
        let (x, y) = self.band(self.overscan / 2.0);
        !(drawn.x.start <= x.start
            && x.end <= drawn.x.end
            && drawn.y.start <= y.start
            && y.end <= drawn.y.end)
    }

    /// The rows and tiles to draw for the current viewport widened by `overscan` px, plus the
    /// tile `pinned` (the grid's active tile) wherever it is, so focus never falls out of the
    /// DOM. Records the band as drawn.
    pub(crate) fn plan(&mut self, overscan: f64, pinned: Option<usize>) -> Vec<RowPlan> {
        let overscan = if overscan.is_finite() {
            overscan.max(0.0)
        } else {
            0.0
        };
        self.overscan = overscan;
        let (x, y) = self.band(overscan);
        let layout = &self.layout;
        let mut rows: Vec<RowPlan> = layout
            .visible_rows(y.clone(), 0.0)
            .map(|row| {
                let r = layout.rows[row];
                RowPlan {
                    row,
                    y: r.y,
                    height: r.height,
                    start: r.start,
                    tiles: layout.tiles[layout.visible_tiles(row, x.clone(), 0.0)].to_vec(),
                }
            })
            .collect();
        if let Some(tile) = pinned.and_then(|index| layout.tiles.get(index)) {
            let r = layout.rows[tile.row];
            let empty = RowPlan {
                row: tile.row,
                y: r.y,
                height: r.height,
                start: r.start,
                tiles: Vec::new(),
            };
            let at = pin_sorted(&mut rows, empty, |plan| plan.row);
            pin_sorted(&mut rows[at].tiles, *tile, |tile| tile.index);
        }
        self.drawn = Some(Band {
            x,
            y,
            generation: self.generation,
        });
        rows
    }
}

/// The horizontal centre of a tile.
fn centre(tile: &Tile) -> f64 {
    tile.x + tile.width / 2.0
}

/// The tile of `row` whose horizontal centre is closest to `x` (the lower index on a tie).
pub(crate) fn nearest_in_row(layout: &JustifiedLayout, row: usize, x: f64) -> Option<usize> {
    let tiles = layout.row_tiles(row);
    if tiles.is_empty() {
        return None;
    }
    // Centres increase along a row: the first centre at or past `x`, or the one before it.
    let after = partition_point(tiles.len(), |i| centre(&tiles[i]) < x);
    let pick = match (after.checked_sub(1), tiles.get(after)) {
        (Some(before), Some(next)) => {
            if x - centre(&tiles[before]) <= centre(next) - x {
                before
            } else {
                after
            }
        }
        (Some(before), None) => before,
        (None, _) => 0,
    };
    Some(tiles[pick].index)
}

/// The keyboard model (see the module documentation): the tile a key moves to from `from`.
/// `page` is the viewport height, the distance `PageUp`/`PageDown` travel. Moves stop at the
/// edges; an empty layout keeps `from`.
pub(crate) fn navigate(layout: &JustifiedLayout, from: usize, key: GridKey, page: f64) -> usize {
    let count = layout.tiles.len();
    if count == 0 {
        return from;
    }
    let from = from.min(count - 1);
    let tile = layout.tiles[from];
    let row = layout.rows[tile.row];
    let last_row = layout.rows.len() - 1;
    let x = centre(&tile);
    let to_row = |target: usize| nearest_in_row(layout, target, x).unwrap_or(from);
    match key {
        GridKey::Prev => from.saturating_sub(1),
        GridKey::Next => (from + 1).min(count - 1),
        GridKey::RowStart => row.start,
        GridKey::RowEnd => row.end - 1,
        GridKey::First => 0,
        GridKey::Last => count - 1,
        GridKey::Up if tile.row == 0 => from,
        GridKey::Up => to_row(tile.row - 1),
        GridKey::Down if tile.row == last_row => from,
        GridKey::Down => to_row(tile.row + 1),
        GridKey::PageUp | GridKey::PageDown => {
            let page = if page.is_finite() { page.max(0.0) } else { 0.0 };
            let middle = tile.y + tile.height / 2.0;
            let down = key == GridKey::PageDown;
            let target_y = if down { middle + page } else { middle - page };
            let mut target = layout.row_at(target_y).unwrap_or(tile.row);
            // A page always moves at least one row (a row taller than the viewport).
            if down && target <= tile.row {
                target = (tile.row + 1).min(last_row);
            } else if !down && target >= tile.row {
                target = tile.row.saturating_sub(1);
            }
            to_row(target)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction::{Direction, HorizontalNav};
    use dioxus::prelude::Key;

    /// Rows of the 1000 px wide test layout (target 200, gap 4) built from a repeating mix.
    fn layout() -> JustifiedLayout {
        let aspects: Vec<f64> = (0..200)
            .map(|i| [0.667, 1.5, 1.0, 1.333, 0.75, 1.778, 3.0][i % 7])
            .collect();
        justified_layout(&aspects, &JustifiedOptions::new(1000.0, 200.0, 4.0))
    }

    fn brute_nearest(tiles: &[Tile], x: f64) -> usize {
        let mut best = tiles[0];
        for tile in tiles {
            if (centre(tile) - x).abs() < (centre(&best) - x).abs() {
                best = *tile;
            }
        }
        best.index
    }

    #[test]
    fn up_and_down_land_on_the_nearest_centre_in_the_adjacent_row() {
        let layout = layout();
        assert!(layout.rows.len() > 10);
        for from in 0..layout.tiles.len() {
            let tile = layout.tiles[from];
            let down = navigate(&layout, from, GridKey::Down, 600.0);
            if tile.row + 1 < layout.rows.len() {
                let want = brute_nearest(layout.row_tiles(tile.row + 1), centre(&tile));
                assert_eq!(down, want, "Down from {from}");
                assert_eq!(layout.tiles[down].row, tile.row + 1);
            } else {
                assert_eq!(down, from, "Down stops at the last row");
            }
            let up = navigate(&layout, from, GridKey::Up, 600.0);
            if tile.row > 0 {
                let want = brute_nearest(layout.row_tiles(tile.row - 1), centre(&tile));
                assert_eq!(up, want, "Up from {from}");
            } else {
                assert_eq!(up, from, "Up stops at the first row");
            }
        }
    }

    #[test]
    fn nearest_prefers_the_lower_index_on_an_exact_tie() {
        // Two equal tiles: x at the boundary between their centres' midpoint.
        let layout = justified_layout(&[1.0, 1.0], &JustifiedOptions::new(204.0, 200.0, 4.0));
        assert_eq!(layout.rows.len(), 1);
        let mid = (centre(&layout.tiles[0]) + centre(&layout.tiles[1])) / 2.0;
        assert_eq!(nearest_in_row(&layout, 0, mid), Some(0));
        assert_eq!(nearest_in_row(&layout, 0, -1e9), Some(0));
        assert_eq!(nearest_in_row(&layout, 0, 1e9), Some(1));
        assert_eq!(nearest_in_row(&layout, 5, 0.0), None);
    }

    #[test]
    fn horizontal_keys_follow_reading_order_across_rows() {
        let layout = layout();
        let second = layout.rows[1];
        assert_eq!(
            navigate(&layout, second.start, GridKey::Prev, 0.0),
            second.start - 1,
            "Prev from a row's first tile is the previous row's last"
        );
        assert_eq!(
            layout.tiles[second.start - 1].row,
            0,
            "and that tile is in the previous row"
        );
        assert_eq!(
            navigate(&layout, second.end - 1, GridKey::Next, 0.0),
            second.end
        );
        assert_eq!(navigate(&layout, 0, GridKey::Prev, 0.0), 0);
        let last = layout.tiles.len() - 1;
        assert_eq!(navigate(&layout, last, GridKey::Next, 0.0), last);
    }

    #[test]
    fn home_end_and_ctrl_home_end() {
        let layout = layout();
        let row = layout.rows[3];
        let inside = row.start + 1;
        assert_eq!(navigate(&layout, inside, GridKey::RowStart, 0.0), row.start);
        assert_eq!(navigate(&layout, inside, GridKey::RowEnd, 0.0), row.end - 1);
        assert_eq!(navigate(&layout, inside, GridKey::First, 0.0), 0);
        assert_eq!(
            navigate(&layout, inside, GridKey::Last, 0.0),
            layout.tiles.len() - 1
        );
    }

    #[test]
    fn page_keys_travel_a_viewport_and_always_at_least_a_row() {
        let layout = layout();
        let from = layout.rows[1].start;
        let paged = navigate(&layout, from, GridKey::PageDown, 600.0);
        let rows_moved = layout.tiles[paged].row - 1;
        // Rows are about 200 px tall plus a 4 px gap: 600 px is about three rows.
        assert!((2..=4).contains(&rows_moved), "moved {rows_moved} rows");
        let back = navigate(&layout, paged, GridKey::PageUp, 600.0);
        assert!(
            layout.tiles[back].row.abs_diff(1) <= 1,
            "PageUp came back near row 1, not row {}",
            layout.tiles[back].row
        );
        // A page shorter than a row still moves one row, and stops at the edges.
        assert_eq!(
            layout.tiles[navigate(&layout, from, GridKey::PageDown, 10.0)].row,
            2
        );
        assert_eq!(
            layout.tiles[navigate(&layout, from, GridKey::PageUp, 10.0)].row,
            0
        );
        assert_eq!(
            layout.tiles[navigate(&layout, 0, GridKey::PageUp, 600.0)].row,
            0
        );
        let last = layout.tiles.len() - 1;
        assert_eq!(
            layout.tiles[navigate(&layout, last, GridKey::PageDown, 600.0)].row,
            layout.rows.len() - 1
        );
    }

    #[test]
    fn an_empty_layout_keeps_the_index() {
        let empty = justified_layout(&[], &JustifiedOptions::new(1000.0, 200.0, 4.0));
        assert_eq!(navigate(&empty, 7, GridKey::Down, 600.0), 7);
    }

    /// Right to left: the horizontal arrows resolve through `direction.rs` (in RTL `ArrowLeft`
    /// is the inline end, i.e. the next tile), and the spatial moves computed in layout space are
    /// the moves on the mirrored drawing, because mirroring every tile is an isometry.
    #[test]
    fn rtl_mirrors_the_arrows_and_keeps_the_spatial_moves() {
        let layout = layout();
        let from = layout.rows[2].start + 1;
        let key = |direction: Direction, key: Key| match direction.resolve_horizontal(&key) {
            Some(HorizontalNav::Prev) => GridKey::Prev,
            Some(HorizontalNav::Next) => GridKey::Next,
            None => unreachable!(),
        };
        assert_eq!(
            navigate(&layout, from, key(Direction::Rtl, Key::ArrowLeft), 0.0),
            from + 1
        );
        assert_eq!(
            navigate(&layout, from, key(Direction::Rtl, Key::ArrowRight), 0.0),
            from - 1
        );
        assert_eq!(
            navigate(&layout, from, key(Direction::Ltr, Key::ArrowRight), 0.0),
            from + 1
        );

        let width = layout.container_width;
        for from in 0..layout.tiles.len() {
            let tile = layout.tiles[from];
            if tile.row + 1 >= layout.rows.len() {
                continue;
            }
            let mirrored: Vec<Tile> = layout
                .row_tiles(tile.row + 1)
                .iter()
                .map(|t| t.mirrored(width))
                .collect();
            let want = brute_nearest(&mirrored, centre(&tile.mirrored(width)));
            assert_eq!(navigate(&layout, from, GridKey::Down, 600.0), want);
        }
    }

    fn report(top: u32, left: u32, width: u32, height: u32) -> ScrollReport {
        ScrollReport {
            top,
            left,
            width,
            height,
            is_scrolling: false,
        }
    }

    fn engine(width: f64, count: usize) -> JustifiedEngine {
        let aspects: Vec<f64> = (0..count)
            .map(|i| [0.667, 1.5, 1.0, 1.333, 0.75, 1.778, 3.0][i % 7])
            .collect();
        let mut engine = JustifiedEngine::new();
        engine.sync(&aspects, Some(JustifiedOptions::new(width, 160.0, 4.0)));
        engine
    }

    #[test]
    fn align_tile_scrolls_to_the_tiles_row_and_clamps_to_the_canvas() {
        let mut engine = engine(800.0, 20_000);
        assert!(!engine.has_layout(), "no viewport has been reported");
        engine.scroll(report(0, 0, 800, 480));
        assert!(engine.has_layout());
        let layout = engine.layout().clone();
        let tile = 12_345;
        let row = layout.rows[layout.tiles[tile].row];
        let start = engine.align_tile(tile, ScrollAlign::Start).unwrap();
        assert_eq!(start, row.y.round() as u32);
        assert_eq!(engine.viewport().top, start, "the engine holds the target");
        let end = engine.align_tile(tile, ScrollAlign::End).unwrap();
        assert_eq!(end, (row.y + row.height).round() as u32 - 480);
        // Nearest from where Start left it is fully visible: no move.
        engine.align_tile(tile, ScrollAlign::Start);
        assert_eq!(engine.align_tile(tile, ScrollAlign::Nearest), Some(start));
        // Past the last tile, the bottom of the canvas.
        let max = layout.total_height.round() as u32 - 480;
        assert_eq!(engine.align_tile(usize::MAX, ScrollAlign::Start), Some(max));
        assert!(!JustifiedEngine::new().has_layout());
        assert_eq!(JustifiedEngine::new().align_tile(0, ScrollAlign::Start), None);
    }

    #[test]
    fn plans_only_the_rows_meeting_the_viewport_and_renders_only_when_it_leaves_the_band() {
        let mut engine = engine(800.0, 20_000);
        engine.scroll(report(0, 0, 800, 480));
        assert!(engine.needs_render());
        let rows = engine.plan(300.0, None);
        let tiles: usize = rows.iter().map(|r| r.tiles.len()).sum();
        assert!(rows.len() < 10, "{} rows", rows.len());
        assert!(tiles < 60, "{tiles} tiles");
        assert_eq!(rows[0].row, 0);
        assert!(!engine.needs_render());
        // A small scroll stays inside the band: no render.
        engine.scroll(report(100, 0, 800, 480));
        assert!(!engine.needs_render());
        // A long one leaves it.
        engine.scroll(report(500_000, 0, 800, 480));
        assert!(engine.needs_render());
        let rows = engine.plan(300.0, None);
        assert!(rows[0].y > 499_000.0 && rows[0].y < 500_000.0);
        // A re-layout always renders.
        let aspects = engine.aspects.clone();
        assert!(!engine.sync(&aspects, Some(JustifiedOptions::new(800.0, 160.0, 4.0))));
        assert!(!engine.needs_render());
        assert!(engine.sync(&aspects, Some(JustifiedOptions::new(640.0, 160.0, 4.0))));
        assert!(engine.needs_render());
    }

    #[test]
    fn a_wide_canvas_windows_the_tiles_of_each_row_by_x() {
        let mut engine = engine(4800.0, 20_000);
        engine.scroll(report(0, 2000, 800, 480));
        let rows = engine.plan(200.0, None);
        for plan in &rows {
            assert!(!plan.tiles.is_empty());
            for tile in &plan.tiles {
                assert!(tile.x + tile.width > 1800.0 && tile.x < 3000.0);
            }
            let all = engine.layout().row_tiles(plan.row).len();
            assert!(plan.tiles.len() < all, "row {} is windowed", plan.row);
        }
        // A horizontal scroll past the band asks for a render.
        engine.scroll(report(0, 2080, 800, 480));
        assert!(!engine.needs_render());
        engine.scroll(report(0, 3500, 800, 480));
        assert!(engine.needs_render());
    }

    #[test]
    fn the_active_tile_is_pinned_into_the_plan_wherever_it_is() {
        let mut engine = engine(800.0, 20_000);
        engine.scroll(report(0, 0, 800, 480));
        let last = 19_999;
        let rows = engine.plan(300.0, Some(last));
        let tail = rows.last().unwrap();
        assert_eq!(tail.row, engine.layout().tiles[last].row);
        assert_eq!(
            tail.tiles.iter().map(|t| t.index).collect::<Vec<_>>(),
            vec![last]
        );
        // Inside the window it is not duplicated.
        let rows = engine.plan(300.0, Some(1));
        let count = rows
            .iter()
            .flat_map(|r| r.tiles.iter())
            .filter(|t| t.index == 1)
            .count();
        assert_eq!(count, 1);
        // Rows stay sorted and unique.
        let rows = engine.plan(300.0, Some(5_000));
        assert!(rows.windows(2).all(|w| w[0].row < w[1].row));
    }

    #[test]
    fn an_unmeasured_fluid_grid_draws_nothing_and_a_relayout_keeps_its_anchor() {
        let mut engine = JustifiedEngine::new();
        let aspects = vec![1.5; 1_000];
        engine.sync(&aspects, None);
        assert!(engine.plan(300.0, Some(0)).is_empty());
        assert_eq!(engine.measured_width(), None);
        engine.scroll(report(0, 0, 900, 480));
        assert_eq!(engine.measured_width(), Some(900.0));
        engine.sync(&aspects, Some(JustifiedOptions::new(900.0, 160.0, 4.0)));
        engine.scroll(report(5_000, 0, 900, 480));
        let anchor = engine.anchor().unwrap();
        let row = engine.layout().tiles[anchor].row;
        assert_eq!(engine.layout().rows[row].start, anchor);
        assert!(engine.tile_top(anchor).unwrap() <= 5_000.0);
        engine.sync(&aspects, Some(JustifiedOptions::new(450.0, 160.0, 4.0)));
        // The same tile now sits further down (half the width, about twice the rows).
        assert!(engine.tile_top(anchor).unwrap() > 8_000.0);
    }
}
