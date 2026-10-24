//! Justified rows (the photo-wall layout of Flickr, Google Photos and Apple Photos): tiles of
//! varying aspect ratio are packed into rows of equal height that fill the container width edge to
//! edge.
//!
//! Pure and DOM-free: aspect ratios and geometry options in, positioned rectangles out. A wall
//! virtualizes by row using [`JustifiedLayout::rows`] (and by tile within a row for a canvas wider
//! than the viewport, see [`JustifiedLayout::visible_tiles`]), mounting only what is visible.
//!
//! # The algorithm
//!
//! Rows are built greedily, left to right. A row takes tiles until their widths, scaled to
//! `target_row_height`, plus the gaps between them, reach the container width. The row's height is
//! then solved so the tiles fill the container exactly:
//! `height = (container_width - gaps) / sum of aspects`. A row closed that way is never taller
//! than the target, because it stopped growing the moment the target-height widths overflowed.
//!
//! The true last row is the exception: it may hold too few tiles to reach the container width, so
//! it is capped at `target_row_height` instead of being stretched into oversized tiles, and may
//! therefore leave a gap at its right edge.
//!
//! ## The minimum tile width floor
//!
//! On a narrow container the greedy row can squeeze its narrowest tile to an illegible width. With
//! `min_tile_width > 0` the row drops tiles off its end (the only order-preserving choice; they
//! start the next row) until its narrowest tile clears the floor or one tile remains. Fewer, wider
//! tiles mean a taller row, so a backed-off row is deliberately taller than the target.
//!
//! Before each drop the loop peeks at the height the next step would need. If that height would
//! exceed `target_row_height * MAX_STRETCH` it stops instead and accepts that the floor goes
//! unmet. This matters: clamping a height afterwards would shrink every tile with it, and the row
//! would no longer reach the container edge. Stopping early keeps the row filling edge to edge at
//! a height that is still reasonable.
//!
//! The narrowest aspect of every prefix of the greedy window is computed once per row
//! ([`FloorLookup`]), so each backoff step is `O(1)` and a row costs `O(tiles)` rather than
//! `O(tiles^2)`. That is sound because the loop only ever shrinks the window from the right, so
//! every candidate is a prefix of the original window.
//!
//! # Invariants
//!
//! - Every item becomes exactly one tile, in order: `tiles[k].index == k`, and a row's tiles are
//!   the contiguous slice `tiles[row.start..row.end]`.
//! - Row offsets are monotonic: `rows[0].y == 0` and `rows[k + 1].y == rows[k].y + rows[k].height + gap`.
//!   Every height is finite and positive, so a binary search on `y` is always valid.
//! - Every row fills `container_width` edge to edge (within rounding), except the documented
//!   exceptions below. Within a row all tiles share `y` and `height`, and tile `x` starts at 0 and
//!   advances by `width + gap`.
//!
//! Exceptions to the edge-to-edge fill (and to the floor): a sparse true last row, capped at the
//! target (the cap may be raised, bounded by `MAX_STRETCH`, to help it clear the floor); a row
//! constrained to one tile because the container itself is narrower than the floor; a row holding
//! a degenerate aspect, which is exempt from the floor; a row that stopped backing off early to
//! stay under `MAX_STRETCH`.
//!
//! # Degenerate input
//!
//! An aspect of zero, a negative number, NaN or infinity is replaced by `MIN_ASPECT` (a 1:20
//! sliver) so a broken or placeholder item never yields a negative or non-finite height. A
//! non-finite or non-positive container width or target height returns the empty layout, and a
//! negative or non-finite gap counts as zero.

use std::ops::Range;

use super::{mirror_x, window};

/// Smallest aspect the layout will honour. Floors zero, negative, NaN and infinite aspects so a
/// degenerate one cannot produce a negative or non-finite row height and break the monotonic
/// row-offset invariant the windowing binary-searches. `0.05` is a 1:20 sliver.
pub(crate) const MIN_ASPECT: f64 = 0.05;

/// Hard ceiling on how far the minimum-width floor may stretch a row above `target_row_height`,
/// as a multiple of it. A row with a real but very narrow aspect (`0.01`: valid, so the
/// degenerate exemption does not apply) could otherwise need an unbounded height to clear any
/// positive floor, since width = height * aspect shrinks toward zero with the aspect.
///
/// The backoff loop's peek-ahead is the first line of defence (it refuses to commit to a tile
/// count whose natural height would exceed this, so the row still fills edge to edge); this
/// constant also bounds the last-row cap raise, the one other place a stretch can be unbounded.
pub(crate) const MAX_STRETCH: f64 = 3.0;

fn is_degenerate(aspect: f64) -> bool {
    !aspect.is_finite() || aspect <= 0.0
}

fn safe_aspect(aspect: f64) -> f64 {
    if is_degenerate(aspect) {
        MIN_ASPECT
    } else {
        aspect
    }
}

/// Geometry options for [`justified_layout`], all in the same pixel unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct JustifiedOptions {
    /// Width the rows fill. A canvas wider than the viewport passes the canvas width.
    pub container_width: f64,
    /// Row height a full row is packed toward; a full row is never taller unless the floor
    /// backed it off.
    pub target_row_height: f64,
    /// Space between tiles, horizontally and between rows.
    pub gap: f64,
    /// Floor under which a row's narrowest tile is not compressed. When honouring it would
    /// compress below it, the row backs off to fewer, wider tiles and stretches above the target.
    /// `0.0` (the default) disables the floor. The floor is best effort: see the module
    /// documentation for the exceptions in which it goes unmet.
    pub min_tile_width: f64,
}

impl JustifiedOptions {
    /// Options with no minimum tile width.
    pub(crate) fn new(container_width: f64, target_row_height: f64, gap: f64) -> Self {
        Self {
            container_width,
            target_row_height,
            gap,
            min_tile_width: 0.0,
        }
    }

    /// The same options with a minimum tile width floor.
    pub(crate) fn with_min_tile_width(mut self, min_tile_width: f64) -> Self {
        self.min_tile_width = min_tile_width;
        self
    }
}

/// One positioned tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tile {
    /// Index into the input aspects.
    pub index: usize,
    /// Index of the row holding this tile.
    pub row: usize,
    /// Left edge.
    pub x: f64,
    /// Top edge (the row's `y`).
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height (the row's height).
    pub height: f64,
}

impl Tile {
    /// The tile mirrored across a container of `container_width`, for a right-to-left layout.
    pub(crate) fn mirrored(&self, container_width: f64) -> Tile {
        Tile {
            x: mirror_x(container_width, self.x, self.width),
            ..*self
        }
    }
}

/// One row of tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Row {
    /// Top edge.
    pub y: f64,
    /// Height of the tiles in the row (the bottom gap is not included).
    pub height: f64,
    /// Inclusive index of the row's first tile.
    pub start: usize,
    /// Exclusive end index of the row's tiles.
    pub end: usize,
}

impl Row {
    /// The tile indices of the row, an index range into [`JustifiedLayout::tiles`].
    pub(crate) fn tiles(&self) -> Range<usize> {
        self.start..self.end
    }
}

/// The output of [`justified_layout`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JustifiedLayout {
    /// One tile per input item, in input order.
    pub tiles: Vec<Tile>,
    /// Rows from top to bottom.
    pub rows: Vec<Row>,
    /// Bottom edge of the last row (no trailing gap); `0` when empty.
    pub total_height: f64,
    /// The container width the layout was solved for, which the RTL helpers mirror across.
    pub container_width: f64,
}

impl JustifiedLayout {
    fn empty(container_width: f64) -> Self {
        Self {
            tiles: Vec::new(),
            rows: Vec::new(),
            total_height: 0.0,
            container_width,
        }
    }

    /// The row whose box starts at or before `y`, by binary search on the row offsets; row `0` for
    /// a `y` above the first row, the last row past the end, `None` for an empty layout.
    pub(crate) fn row_at(&self, y: f64) -> Option<usize> {
        if self.rows.is_empty() {
            return None;
        }
        let after = super::partition_point(self.rows.len(), |i| self.rows[i].y <= y);
        Some(after.saturating_sub(1))
    }

    /// The tiles of `row`, empty for an out-of-range row.
    pub(crate) fn row_tiles(&self, row: usize) -> &[Tile] {
        self.rows
            .get(row)
            .map_or(&[], |r| &self.tiles[r.start..r.end])
    }

    /// The rows whose boxes meet the vertical band `y` (half-open, in layout pixels), widened by
    /// `overscan` on each side. Two binary searches, `O(log rows)`.
    pub(crate) fn visible_rows(&self, y: Range<f64>, overscan: f64) -> Range<usize> {
        window(
            self.rows.len(),
            |i| (self.rows[i].y, self.rows[i].y + self.rows[i].height),
            y.start,
            y.end,
            overscan,
        )
    }

    /// The tiles of `row` that meet the horizontal band `x` (half-open, in layout pixels),
    /// widened by `overscan` on each side, as indices into [`Self::tiles`]. A row's tiles are
    /// sorted by `x`, so this is two binary searches, `O(log tiles in the row)`.
    ///
    /// For a right-to-left layout, mirror the band with [`super::mirror_range`] first and the
    /// resulting tiles with [`Tile::mirrored`].
    pub(crate) fn visible_tiles(&self, row: usize, x: Range<f64>, overscan: f64) -> Range<usize> {
        let Some(r) = self.rows.get(row) else {
            return 0..0;
        };
        let tiles = &self.tiles[r.start..r.end];
        let local = window(
            tiles.len(),
            |i| (tiles[i].x, tiles[i].x + tiles[i].width),
            x.start,
            x.end,
            overscan,
        );
        (r.start + local.start)..(r.start + local.end)
    }

    /// Every tile meeting the rectangle `x` by `y` (each half-open, in layout pixels) widened by
    /// `overscan` on every side: [`Self::visible_rows`], then [`Self::visible_tiles`] per row.
    pub(crate) fn visible_in(
        &self,
        x: Range<f64>,
        y: Range<f64>,
        overscan: f64,
    ) -> impl Iterator<Item = &Tile> + '_ {
        self.visible_rows(y, overscan)
            .flat_map(move |row| self.tiles[self.visible_tiles(row, x.clone(), overscan)].iter())
    }
}

/// For one row's greedy window `items[start..start + count]`, answers in `O(1)` for every prefix
/// length `c` in `1..=count`: does the prefix hold a degenerate aspect, and what is its narrowest
/// real aspect. Built once per row in `O(count)`.
struct FloorLookup {
    /// `min_aspect_at[c]`: the narrowest safe aspect among the first `c` items.
    min_aspect_at: Vec<f64>,
    /// Prefix length from which a degenerate item is present, if any: the first degenerate
    /// item's offset plus one.
    degenerate_from: Option<usize>,
}

impl FloorLookup {
    fn new(aspects: &[f64]) -> Self {
        let mut min_aspect_at = Vec::with_capacity(aspects.len() + 1);
        let mut degenerate_from = None;
        let mut running = f64::INFINITY;
        min_aspect_at.push(running);
        for (offset, &a) in aspects.iter().enumerate() {
            if degenerate_from.is_none() && is_degenerate(a) {
                degenerate_from = Some(offset + 1);
            }
            running = running.min(safe_aspect(a));
            min_aspect_at.push(running);
        }
        Self {
            min_aspect_at,
            degenerate_from,
        }
    }

    /// `(has_degenerate, narrowest_aspect)` of the first `c` items.
    ///
    /// A window holding a degenerate item is exempt from the floor entirely: `MIN_ASPECT` is a
    /// safety floor for broken input, not narrow real content worth widening, and treating it as
    /// such divides by about `0.05` and explodes the row height for no legibility benefit.
    /// Conservative on purpose: a row mixing one degenerate item with genuinely narrow tiles still
    /// skips the floor.
    fn at(&self, c: usize) -> (bool, f64) {
        (
            self.degenerate_from.is_some_and(|from| from <= c),
            self.min_aspect_at[c],
        )
    }
}

/// Packs `aspects` (width over height, one per item) into justified rows.
pub(crate) fn justified_layout(aspects: &[f64], opts: &JustifiedOptions) -> JustifiedLayout {
    let container_width = opts.container_width;
    let target = opts.target_row_height;
    let gap = if opts.gap.is_finite() && opts.gap > 0.0 {
        opts.gap
    } else {
        0.0
    };
    let min_tile_width = if opts.min_tile_width.is_finite() {
        opts.min_tile_width
    } else {
        0.0
    };

    let valid = |v: f64| v.is_finite() && v > 0.0;
    if aspects.is_empty() || !valid(container_width) || !valid(target) {
        return JustifiedLayout::empty(container_width);
    }

    let mut tiles = Vec::with_capacity(aspects.len());
    let mut rows: Vec<Row> = Vec::new();
    let mut y = 0.0;
    let mut i = 0;

    // Accumulate items into the current row until adding the next would overflow, then justify
    // the row's height so its tiles exactly fill the width.
    while i < aspects.len() {
        let mut aspect_sum = 0.0;
        let mut j = i;
        // Grow the row until the tiles scaled to the target height, plus the gaps between them,
        // reach the container width.
        while j < aspects.len() {
            aspect_sum += safe_aspect(aspects[j]);
            j += 1;
            let gaps = (j - i - 1) as f64 * gap;
            if target * aspect_sum + gaps >= container_width {
                break;
            }
        }

        let mut tile_count = j - i;

        // Floor safeguard: drop tiles off the end until the row's narrowest tile clears the
        // floor or one tile remains, unless the window holds a degenerate aspect (backing off
        // cannot help there: the degenerate item stays the bottleneck at any count).
        let floor = (min_tile_width > 0.0).then(|| FloorLookup::new(&aspects[i..j]));
        if let Some(floor) = &floor {
            while tile_count > 1 {
                let (has_degenerate, min_aspect) = floor.at(tile_count);
                if has_degenerate {
                    break;
                }
                let gaps = (tile_count - 1) as f64 * gap;
                let row_height = (container_width - gaps) / aspect_sum;
                if row_height * min_aspect >= min_tile_width {
                    break;
                }
                // Peek at the height ONE more backoff step would need; stop here rather than
                // commit to a height MAX_STRETCH would have to rein in afterwards (that would
                // shrink the row's width below the container: a gap, not a fill).
                let next_count = tile_count - 1;
                let next_sum = aspect_sum - safe_aspect(aspects[i + tile_count - 1]);
                let next_gaps = (next_count - 1) as f64 * gap;
                let next_height = (container_width - next_gaps) / next_sum;
                if next_height > target * MAX_STRETCH {
                    break;
                }
                aspect_sum = next_sum;
                tile_count = next_count;
            }
            j = i + tile_count;
        }

        let is_last_row = j >= aspects.len();
        let avail_for_tiles = container_width - (tile_count - 1) as f64 * gap;

        // Height that makes the row's tiles fill `avail_for_tiles` exactly. A full row is at or
        // below the target by construction, unless the floor backed it off to fewer tiles.
        let mut row_height = avail_for_tiles / aspect_sum;
        if is_last_row && row_height > target {
            // The last row may be partial, so it is not stretched past the target. A sparse
            // trailing row (one tile left after an earlier row backed off) could still land
            // under the floor re-capped at the target, so the cap is raised just enough to clear
            // it, except for a degenerate row (no real width to protect, only an absurd one to
            // avoid).
            let mut cap = target;
            if let Some(floor) = &floor {
                let (has_degenerate, min_aspect) = floor.at(tile_count);
                if !has_degenerate {
                    cap = cap.max(min_tile_width / min_aspect);
                }
            }
            row_height = row_height.min(cap);
        }
        // Backstop for the cap raise above: the stretch a sparse trailing row with a very narrow
        // aspect would need is as unbounded as the backoff case was before the peek-ahead.
        if min_tile_width > 0.0 && row_height > target * MAX_STRETCH {
            row_height = target * MAX_STRETCH;
        }

        let row = rows.len();
        let mut x = 0.0;
        for (k, &aspect) in aspects.iter().enumerate().take(j).skip(i) {
            let width = row_height * safe_aspect(aspect);
            tiles.push(Tile {
                index: k,
                row,
                x,
                y,
                width,
                height: row_height,
            });
            x += width + gap;
        }

        rows.push(Row {
            y,
            height: row_height,
            start: i,
            end: j,
        });
        y += row_height + gap;
        i = j;
    }

    let total_height = rows.last().map_or(0.0, |last| last.y + last.height);
    JustifiedLayout {
        tiles,
        rows,
        total_height,
        container_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTS: JustifiedOptions = JustifiedOptions {
        container_width: 1000.0,
        target_row_height: 200.0,
        gap: 4.0,
        min_tile_width: 0.0,
    };

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!(close(a, b, tol), "{a} is not within {tol} of {b}");
    }

    fn used_width(tiles: &[Tile], gap: f64) -> f64 {
        tiles.iter().map(|t| t.width).sum::<f64>() + gap * (tiles.len() as f64 - 1.0)
    }

    fn run(aspects: &[f64], opts: &JustifiedOptions) -> JustifiedLayout {
        justified_layout(aspects, opts)
    }

    #[test]
    fn returns_nothing_for_an_empty_input() {
        let r = run(&[], &OPTS);
        assert!(r.tiles.is_empty());
        assert!(r.rows.is_empty());
        assert_eq!(r.total_height, 0.0);
    }

    #[test]
    fn keeps_a_single_short_row_at_the_target_height() {
        let r = run(&[1.5], &OPTS);
        assert_eq!(r.rows.len(), 1);
        assert_close(r.tiles[0].height, 200.0, 1e-5);
        assert_close(r.tiles[0].width, 300.0, 1e-5);
        assert_eq!(r.tiles[0].x, 0.0);
        assert_eq!(r.tiles[0].y, 0.0);
    }

    #[test]
    fn justifies_full_rows_to_exactly_fill_the_container_width() {
        let r = run(&[1.0; 40], &OPTS);
        // Every row except possibly the last fills the width.
        for i in 0..r.rows.len() - 1 {
            assert_close(
                used_width(r.row_tiles(i), OPTS.gap),
                OPTS.container_width,
                0.05,
            );
        }
    }

    #[test]
    fn gives_all_tiles_in_a_row_the_same_height_and_y() {
        let aspects: Vec<f64> = (0..20).map(|i| 1.0 + (i % 3) as f64 * 0.3).collect();
        let r = run(&aspects, &OPTS);
        for i in 0..r.rows.len() {
            let row_tiles = r.row_tiles(i);
            let (h, y) = (row_tiles[0].height, row_tiles[0].y);
            for t in row_tiles {
                assert_close(t.height, h, 1e-5);
                assert_eq!(t.y, y);
            }
        }
    }

    #[test]
    fn lays_tiles_left_to_right_with_gaps_starting_at_zero() {
        let r = run(&[1.0; 10], &OPTS);
        let row0 = r.row_tiles(0);
        assert_eq!(row0[0].x, 0.0);
        for pair in row0.windows(2) {
            assert_close(pair[1].x, pair[0].x + pair[0].width + OPTS.gap, 1e-3);
        }
    }

    #[test]
    fn stacks_rows_vertically_and_reports_a_consistent_total_height() {
        let r = run(&[1.4; 50], &OPTS);
        let mut expected_y = 0.0;
        for row in &r.rows {
            assert_close(row.y, expected_y, 1e-3);
            expected_y += row.height + OPTS.gap;
        }
        // Total height is the last row's bottom, with no trailing gap.
        let last = r.rows.last().unwrap();
        assert_close(r.total_height, last.y + last.height, 1e-3);
    }

    #[test]
    fn keeps_full_justified_rows_at_or_below_the_target_height() {
        // Full rows pack until target-height widths fill the container, so the justified height
        // is never taller than the target.
        let r = run(&[0.5; 40], &OPTS);
        for row in &r.rows[..r.rows.len() - 1] {
            assert!(row.height <= OPTS.target_row_height + 0.001);
        }
    }

    #[test]
    fn does_not_stretch_a_single_short_last_row_past_target() {
        let r = run(&[0.2], &OPTS);
        assert_close(r.tiles[0].height, 200.0, 1e-5);
    }

    #[test]
    fn produces_every_tile_exactly_once_in_order() {
        let aspects: Vec<f64> = (0..37).map(|i| 1.0 + (i % 5) as f64 * 0.2).collect();
        let r = run(&aspects, &OPTS);
        assert_eq!(r.tiles.len(), 37);
        let indices: Vec<usize> = r.tiles.iter().map(|t| t.index).collect();
        assert_eq!(indices, (0..37).collect::<Vec<_>>());
    }

    // Degenerate aspects (0, negative, NaN, infinity) must not produce negative or non-finite
    // heights, which would break the monotonic row-offset invariant.
    #[test]
    fn floors_a_degenerate_aspect_to_a_finite_positive_height() {
        for aspect in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let r = run(&[aspect], &OPTS);
            assert_eq!(r.tiles.len(), 1, "aspect {aspect}");
            assert!(r.tiles[0].height.is_finite(), "aspect {aspect}");
            assert!(r.tiles[0].height > 0.0, "aspect {aspect}");
            assert!(r.total_height.is_finite(), "aspect {aspect}");
            assert!(r.total_height > 0.0, "aspect {aspect}");
        }
    }

    #[test]
    fn keeps_row_offsets_monotonic_with_a_mix_of_degenerate_and_normal_aspects() {
        let aspects = [1.0, -5.0, 0.0, 1.5, f64::NAN];
        let opts = JustifiedOptions {
            container_width: 300.0,
            ..OPTS
        };
        let r = run(&aspects, &opts);
        for pair in r.rows.windows(2) {
            assert!(pair[1].y >= pair[0].y);
        }
        assert!(r
            .tiles
            .iter()
            .all(|t| t.height.is_finite() && t.height > 0.0));
    }

    #[test]
    fn invalid_options_give_the_empty_layout() {
        for opts in [
            JustifiedOptions::new(0.0, 200.0, 4.0),
            JustifiedOptions::new(-10.0, 200.0, 4.0),
            JustifiedOptions::new(f64::NAN, 200.0, 4.0),
            JustifiedOptions::new(f64::INFINITY, 200.0, 4.0),
            JustifiedOptions::new(1000.0, 0.0, 4.0),
            JustifiedOptions::new(1000.0, f64::NAN, 4.0),
        ] {
            let r = run(&[1.0, 2.0], &opts);
            assert!(r.tiles.is_empty() && r.rows.is_empty());
            assert_eq!(r.total_height, 0.0);
        }
    }

    #[test]
    fn a_bad_gap_counts_as_zero() {
        let zero = run(&[1.0; 12], &JustifiedOptions::new(1000.0, 200.0, 0.0));
        for gap in [-5.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                run(&[1.0; 12], &JustifiedOptions::new(1000.0, 200.0, gap)),
                zero
            );
        }
    }

    mod min_tile_width_floor {
        use super::*;

        // A fixed aspect below 1 on a narrow phone-width container. Without a floor this greedily
        // packs 3 tiles per row at about 117px: a cramped wall.
        const NARROW: JustifiedOptions = JustifiedOptions {
            container_width: 375.0,
            target_row_height: 210.0,
            gap: 12.0,
            min_tile_width: 0.0,
        };

        fn floored(floor: f64) -> JustifiedOptions {
            JustifiedOptions {
                min_tile_width: floor,
                ..NARROW
            }
        }

        #[test]
        fn with_no_floor_compresses_to_the_greedy_tile_count() {
            let r = run(&[0.82; 9], &NARROW);
            let row0 = r.row_tiles(0);
            assert_eq!(row0.len(), 3);
            assert!(row0[0].width < 120.0);
        }

        #[test]
        fn backs_off_to_fewer_wider_tiles_once_the_floor_would_be_crossed() {
            let r = run(&[0.82; 9], &floored(140.0));
            let row0 = r.row_tiles(0);
            assert_eq!(row0.len(), 2);
            assert!(row0[0].width >= 140.0);
        }

        #[test]
        fn still_fills_the_row_edge_to_edge_after_backing_off() {
            let r = run(&[0.82; 9], &floored(140.0));
            let row0 = r.row_tiles(0);
            assert_close(used_width(row0, NARROW.gap), NARROW.container_width, 0.05);
            assert!(row0[0].height > NARROW.target_row_height);
        }

        #[test]
        fn never_reduces_a_single_tile_row_further() {
            // The container alone is already under the floor: a lone tile spans the full width
            // regardless; there is nothing narrower to back off to.
            let opts = JustifiedOptions {
                container_width: 100.0,
                min_tile_width: 140.0,
                ..NARROW
            };
            let r = run(&[0.82], &opts);
            assert_eq!(r.tiles.len(), 1);
            assert_close(r.tiles[0].width, 100.0, 1e-5);
        }

        fn varied() -> Vec<f64> {
            (0..37).map(|i| 0.5 + (i % 5) as f64 * 0.1).collect()
        }

        fn varied_opts() -> JustifiedOptions {
            JustifiedOptions {
                container_width: 300.0,
                min_tile_width: 150.0,
                ..OPTS
            }
        }

        #[test]
        fn produces_every_tile_exactly_once_in_order_when_backing_off() {
            let r = run(&varied(), &varied_opts());
            assert_eq!(r.tiles.len(), 37);
            let indices: Vec<usize> = r.tiles.iter().map(|t| t.index).collect();
            assert_eq!(indices, (0..37).collect::<Vec<_>>());
        }

        #[test]
        fn keeps_row_offsets_monotonic_when_backing_off() {
            let r = run(&varied(), &varied_opts());
            let mut expected_y = 0.0;
            for row in &r.rows {
                assert_close(row.y, expected_y, 1e-3);
                expected_y += row.height + OPTS.gap;
            }
        }

        #[test]
        fn is_a_no_op_when_the_greedy_row_already_clears_the_floor() {
            let without = run(&[1.0; 40], &OPTS);
            let with = run(
                &[1.0; 40],
                &JustifiedOptions {
                    min_tile_width: 50.0,
                    ..OPTS
                },
            );
            assert_eq!(with, without);
        }

        #[test]
        fn exercises_a_floor_of_120_directly() {
            let r = run(&[0.82; 9], &floored(120.0));
            for t in r.row_tiles(0) {
                assert!(t.width >= 120.0);
            }
        }

        #[test]
        fn a_trailing_single_tile_row_created_by_backoff_still_clears_the_floor() {
            // Three items just barely reach the threshold together; backing the first row off to
            // two tiles leaves one leftover item as its own trailing (genuinely last) row, which
            // must not be re-capped at the target height (105px against a 140px floor).
            let r = run(&[0.5; 3], &floored(140.0));
            for t in &r.tiles {
                assert!(t.width >= 140.0, "tile {} is {}px wide", t.index, t.width);
            }
        }

        #[test]
        fn a_degenerate_aspect_with_a_floor_does_not_explode_row_height() {
            // A degenerate item (floored to the 1:20 sliver) at the start of a row must not make
            // every backoff candidate look sub-floor, cascading to a one tile row and dividing by
            // about 0.05: a 375px container exploded to 7500px tall. Such a row is exempt from
            // the floor entirely, matching the no-floor baseline exactly.
            let mut aspects = vec![0.0];
            aspects.extend([1.0; 8]);
            let base = JustifiedOptions {
                container_width: 375.0,
                target_row_height: 200.0,
                gap: 12.0,
                min_tile_width: 0.0,
            };
            let with_floor = run(
                &aspects,
                &JustifiedOptions {
                    min_tile_width: 140.0,
                    ..base
                },
            );
            let without_floor = run(&aspects, &base);
            assert_close(
                with_floor.rows[0].height,
                without_floor.rows[0].height,
                1e-5,
            );
            assert!(with_floor.rows[0].height < 200.0 * 3.0);
        }

        fn sliver_then_squares() -> Vec<f64> {
            let mut aspects = vec![0.01];
            aspects.extend([1.0; 5]);
            aspects
        }

        fn sliver_opts() -> JustifiedOptions {
            JustifiedOptions {
                min_tile_width: 120.0,
                ..NARROW
            }
        }

        #[test]
        fn a_valid_but_very_narrow_aspect_does_not_explode_row_height_either() {
            // A real, valid aspect that is just very small (0.01) is not exempted as degenerate,
            // so backoff could isolate it in its own row: 375 / 0.01 = 37500px tall, uncapped.
            // Clamping the height alone broke edge to edge fill instead (width shrinks with the
            // height), so the backoff peeks ahead and stops BEFORE committing to such a height.
            // Asserts the fill is preserved, not just the bound.
            let r = run(&sliver_then_squares(), &sliver_opts());
            assert!(r.rows[0].height <= 210.0 * MAX_STRETCH);
            assert!(r
                .tiles
                .iter()
                .all(|t| t.height.is_finite() && t.height > 0.0));
            assert_close(used_width(r.row_tiles(0), 12.0), 375.0, 0.05);
        }

        #[test]
        fn every_row_fills_edge_to_edge_and_stays_height_bounded_across_many_rows() {
            // Mixed real aspects, no degenerate values. The floor is best effort (a row may stop
            // backing off early rather than stretch past MAX_STRETCH), but these two properties
            // hold on every row.
            let aspects: Vec<f64> = (0..200)
                .map(|i| 0.5 + ((i * 37) % 130) as f64 / 100.0)
                .collect();
            let opts = JustifiedOptions {
                container_width: 340.0,
                min_tile_width: 130.0,
                ..OPTS
            };
            let r = run(&aspects, &opts);
            for (i, row) in r.rows.iter().enumerate() {
                let is_last = i == r.rows.len() - 1;
                // A genuinely sparse true last row is the one documented exception to the fill.
                if !is_last {
                    assert_close(used_width(r.row_tiles(i), OPTS.gap), 340.0, 0.05);
                }
                assert!(row.height <= OPTS.target_row_height * MAX_STRETCH + 1e-6);
            }
        }

        #[test]
        fn a_row_may_stop_backing_off_before_every_tile_clears_the_floor() {
            // Demonstrates (not just permits) the peek-ahead early stop.
            let r = run(&sliver_then_squares(), &sliver_opts());
            let row0 = r.row_tiles(0);
            assert_eq!(row0.len(), 2, "stopped at 2, not fully backed off to 1");
            assert!(
                row0.iter().any(|t| t.width < 120.0),
                "the 0.01 tile misses the floor"
            );
            assert_close(used_width(row0, 12.0), 375.0, 0.05);
        }

        #[test]
        fn stays_fast_when_many_tiles_in_one_row_share_the_same_narrow_aspect() {
            // The backoff loop once rescanned the whole shrinking window on every step:
            // O(tiles) per step, O(tiles^2) per row, tens of seconds for 200k tiles. The
            // one-time prefix lookup makes it linear; a generous time budget pins that.
            let aspects = vec![0.001; 50_000];
            let opts = JustifiedOptions {
                container_width: 5000.0,
                target_row_height: 200.0,
                gap: 0.0,
                min_tile_width: 120.0,
            };
            let start = std::time::Instant::now();
            let r = run(&aspects, &opts);
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
            assert_eq!(r.tiles.len(), 50_000);
        }

        #[test]
        fn honours_the_floor_at_the_exact_boundary() {
            // The container is chosen so two tiles land exactly on the floor once backed off from
            // three: the `>=` in the backoff loop must accept that as cleared.
            let (aspect, floor, gap) = (0.5, 130.0, 12.0);
            // height * aspect == floor  =>  height = 260; two tiles fill 2 * 260 * 0.5 + gap.
            let opts = JustifiedOptions {
                container_width: 2.0 * 260.0 * aspect + gap,
                target_row_height: 210.0,
                gap,
                min_tile_width: floor,
            };
            let r = run(&[aspect; 3], &opts);
            let row0 = r.row_tiles(0);
            assert_eq!(row0.len(), 2);
            assert_close(row0[0].width, floor, 1e-5);
        }
    }

    // --- Windowing -------------------------------------------------------------------------

    /// A small deterministic generator (xorshift64*), so the brute-force tests need no crate.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        /// Uniform in `[lo, hi)`.
        fn range(&mut self, lo: f64, hi: f64) -> f64 {
            lo + (self.next() >> 11) as f64 / (1u64 << 53) as f64 * (hi - lo)
        }
    }

    fn random_layout(rng: &mut Rng) -> JustifiedLayout {
        let n = (rng.range(0.0, 120.0)) as usize;
        let aspects: Vec<f64> = (0..n).map(|_| rng.range(0.3, 2.8)).collect();
        let opts = JustifiedOptions {
            container_width: rng.range(150.0, 3000.0),
            target_row_height: rng.range(60.0, 300.0),
            gap: rng.range(0.0, 12.0),
            min_tile_width: if rng.range(0.0, 1.0) < 0.5 {
                0.0
            } else {
                rng.range(40.0, 160.0)
            },
        };
        justified_layout(&aspects, &opts)
    }

    fn meets(t: &Tile, x: &Range<f64>, y: &Range<f64>, overscan: f64) -> bool {
        t.x < x.end + overscan
            && t.x + t.width > x.start - overscan
            && t.y < y.end + overscan
            && t.y + t.height > y.start - overscan
    }

    #[test]
    fn visible_in_returns_exactly_the_tiles_intersecting_the_rect() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for _ in 0..300 {
            let layout = random_layout(&mut rng);
            for _ in 0..12 {
                let (x0, y0) = (
                    rng.range(-200.0, layout.container_width + 200.0),
                    rng.range(-200.0, layout.total_height + 200.0),
                );
                let x = x0..x0 + rng.range(0.0, 1500.0);
                let y = y0..y0 + rng.range(0.0, 1500.0);
                let overscan = if rng.range(0.0, 1.0) < 0.5 {
                    0.0
                } else {
                    rng.range(0.0, 400.0)
                };
                let got: Vec<usize> = layout
                    .visible_in(x.clone(), y.clone(), overscan)
                    .map(|t| t.index)
                    .collect();
                let want: Vec<usize> = layout
                    .tiles
                    .iter()
                    .filter(|t| meets(t, &x, &y, overscan))
                    .map(|t| t.index)
                    .collect();
                assert_eq!(got, want, "x={x:?} y={y:?} overscan={overscan}");
            }
        }
    }

    #[test]
    fn visible_rows_returns_exactly_the_rows_meeting_the_band() {
        let mut rng = Rng(42);
        for _ in 0..300 {
            let layout = random_layout(&mut rng);
            for _ in 0..12 {
                let y0 = rng.range(-300.0, layout.total_height + 300.0);
                let y = y0..y0 + rng.range(0.0, 1200.0);
                let overscan = rng.range(0.0, 300.0);
                let got: Vec<usize> = layout.visible_rows(y.clone(), overscan).collect();
                let want: Vec<usize> = (0..layout.rows.len())
                    .filter(|&i| {
                        let r = &layout.rows[i];
                        r.y < y.end + overscan && r.y + r.height > y.start - overscan
                    })
                    .collect();
                assert_eq!(got, want);
            }
        }
    }

    #[test]
    fn visible_tiles_returns_exactly_the_tiles_of_the_row_meeting_the_band() {
        let mut rng = Rng(7);
        for _ in 0..200 {
            let layout = random_layout(&mut rng);
            for row in 0..layout.rows.len() {
                let x0 = rng.range(-100.0, layout.container_width + 100.0);
                let x = x0..x0 + rng.range(0.0, 900.0);
                let overscan = rng.range(0.0, 200.0);
                let got = layout.visible_tiles(row, x.clone(), overscan);
                let want: Vec<usize> = layout.rows[row]
                    .tiles()
                    .filter(|&i| {
                        let t = &layout.tiles[i];
                        t.x < x.end + overscan && t.x + t.width > x.start - overscan
                    })
                    .collect();
                assert_eq!(got.collect::<Vec<_>>(), want);
            }
        }
    }

    #[test]
    fn a_band_covering_everything_returns_everything_and_a_far_band_nothing() {
        let r = run(&[1.0; 60], &OPTS);
        let all = r
            .visible_in(
                -1.0..r.container_width + 1.0,
                -1.0..r.total_height + 1.0,
                0.0,
            )
            .count();
        assert_eq!(all, 60);
        assert_eq!(r.visible_rows(1e9..2e9, 0.0), r.rows.len()..r.rows.len());
        assert_eq!(r.visible_rows(-2e9..-1e9, 0.0).len(), 0);
        assert_eq!(r.visible_tiles(r.rows.len(), 0.0..10.0, 0.0), 0..0);
        assert_eq!(r.visible_in(5e8..6e8, 0.0..10.0, 0.0).count(), 0);
    }

    #[test]
    fn overscan_widens_the_window_in_pixels() {
        // Rows of height ~200 + 4 gap: a thin band inside row 3 sees only row 3 without overscan,
        // and the neighbours once overscan reaches past the gaps.
        let r = run(&[1.0; 60], &OPTS);
        let mid = r.rows[3].y + r.rows[3].height / 2.0;
        assert_eq!(r.visible_rows(mid..mid + 1.0, 0.0), 3..4);
        let reach = r.rows[3].height / 2.0 + OPTS.gap + 1.0;
        assert_eq!(r.visible_rows(mid..mid + 1.0, reach), 2..5);
    }

    #[test]
    fn row_at_finds_the_row_at_or_before_y() {
        let r = run(&[1.0; 60], &OPTS);
        assert_eq!(r.row_at(0.0), Some(0));
        assert_eq!(r.row_at(-50.0), Some(0));
        assert_eq!(r.row_at(r.rows[4].y), Some(4));
        assert_eq!(r.row_at(r.rows[4].y - 0.5), Some(3));
        assert_eq!(r.row_at(1e9), Some(r.rows.len() - 1));
        assert_eq!(run(&[], &OPTS).row_at(0.0), None);
    }

    #[test]
    fn rows_expose_their_tiles_as_contiguous_slices() {
        let r = run(&[1.3; 33], &OPTS);
        let mut next = 0;
        for (i, row) in r.rows.iter().enumerate() {
            assert_eq!(row.start, next);
            assert!(row.end > row.start);
            assert!(r.row_tiles(i).iter().all(|t| t.row == i));
            next = row.end;
        }
        assert_eq!(next, 33);
        assert!(r.row_tiles(r.rows.len()).is_empty());
    }

    // --- Right-to-left ---------------------------------------------------------------------

    #[test]
    fn a_mirrored_row_still_fills_edge_to_edge_in_reverse_order() {
        let r = run(&[1.0; 40], &OPTS);
        let row0: Vec<Tile> = r
            .row_tiles(0)
            .iter()
            .map(|t| t.mirrored(r.container_width))
            .collect();
        // The first tile now sits against the right edge and the last against the left one.
        assert_close(row0[0].x + row0[0].width, r.container_width, 1e-9);
        assert_close(row0[row0.len() - 1].x, 0.0, 0.05);
        for pair in row0.windows(2) {
            assert!(pair[1].x < pair[0].x);
            assert_close(pair[0].x - (pair[1].x + pair[1].width), OPTS.gap, 1e-6);
        }
    }

    #[test]
    fn mirroring_a_tile_twice_restores_it() {
        let r = run(&[1.1, 0.8, 1.6, 1.0, 1.3], &OPTS);
        for t in &r.tiles {
            let back = t.mirrored(r.container_width).mirrored(r.container_width);
            assert_close(back.x, t.x, 1e-9);
            assert_eq!(back.width, t.width);
        }
    }

    #[test]
    fn rtl_windowing_through_a_mirrored_band_matches_a_brute_force_over_mirrored_tiles() {
        let mut rng = Rng(0xC0FFEE);
        for _ in 0..200 {
            let layout = random_layout(&mut rng);
            let cw = layout.container_width;
            for _ in 0..8 {
                // A physical (right-to-left) viewport band and rows band.
                let x0 = rng.range(-100.0, cw + 100.0);
                let x = x0..x0 + rng.range(0.0, 1200.0);
                let y0 = rng.range(-100.0, layout.total_height + 100.0);
                let y = y0..y0 + rng.range(0.0, 1200.0);
                let got: Vec<(usize, u64)> = layout
                    .visible_in(super::super::mirror_range(cw, x.clone()), y.clone(), 0.0)
                    .map(|t| {
                        let m = t.mirrored(cw);
                        (m.index, m.x.to_bits())
                    })
                    .collect();
                let want: Vec<(usize, u64)> = layout
                    .tiles
                    .iter()
                    .map(|t| t.mirrored(cw))
                    .filter(|m| meets(m, &x, &y, 0.0))
                    .map(|m| (m.index, m.x.to_bits()))
                    .collect();
                // Boundary rounding can differ in the last bit of x; compare indices instead.
                let got_idx: Vec<usize> = got.iter().map(|g| g.0).collect();
                let want_idx: Vec<usize> = want.iter().map(|g| g.0).collect();
                assert_eq!(got_idx, want_idx);
            }
        }
    }

    #[test]
    fn a_canvas_wider_than_the_viewport_windows_a_small_slice() {
        // A 20000px canvas with a 1200px viewport scrolled to the middle: a few tiles per row,
        // nowhere near the whole row.
        let opts = JustifiedOptions::new(20_000.0, 200.0, 4.0);
        let r = run(&[1.5; 4000], &opts);
        let scroll_x = 9_000.0;
        let band = scroll_x..scroll_x + 1200.0;
        let row = r.visible_rows(0.0..800.0, 0.0).start;
        let tiles = r.visible_tiles(row, band.clone(), 0.0);
        assert!(
            tiles.len() >= 3 && tiles.len() <= 8,
            "{} tiles",
            tiles.len()
        );
        for t in &r.tiles[tiles] {
            assert!(t.x < band.end && t.x + t.width > band.start);
        }
    }
}
