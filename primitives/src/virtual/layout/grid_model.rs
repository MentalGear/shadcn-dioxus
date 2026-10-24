//! The flat, absolutely positioned row model a sectioned wall renders: a header row per section
//! followed by that section's justified photo rows, or a uniform square grid with no headers.
//!
//! Pure, so it is unit-tested and can run once per layout change (resize, zoom) off the render
//! path. The component then windows [`GridModel::rows`] by scroll offset
//! ([`GridModel::visible_rows`]) and each photo row by horizontal offset
//! ([`PhotoRow::visible_tiles`]).
//!
//! # Invariants
//!
//! - Rows stack contiguously: `rows[0].y == 0` and `rows[k + 1].y == rows[k].y + rows[k].height`,
//!   so a binary search on `y` is valid and `total_height` is the sum of all row heights. A photo
//!   row's height folds in the bottom gap, and a section gap is folded into the previous
//!   section's last row, to keep that equality exact.
//! - Every item of a section appears in exactly one photo row of that section, in order.
//! - Tiles of a photo row are sorted by `x`, starting at 0.

use std::ops::Range;

use super::justified::{justified_layout, JustifiedOptions};
use super::window;

/// A contiguous run of items with display strings; one header row is emitted per section.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Section {
    /// Stable identifier.
    pub key: String,
    /// Header title.
    pub title: String,
    /// Optional header subtitle.
    pub subtitle: Option<String>,
    /// Inclusive start index into the item array.
    pub start: usize,
    /// Exclusive end index into the item array.
    pub end: usize,
    /// Item count shown in the header.
    pub count: usize,
}

/// One tile of a photo row. Its `y` is the row's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridTile {
    /// Index into the source item array.
    pub item_index: usize,
    /// Left edge within the row.
    pub x: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// A section header row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeaderRow {
    /// Top edge.
    pub y: f64,
    /// Height.
    pub height: f64,
    /// Index of the section.
    pub section: usize,
    /// Section title.
    pub title: String,
    /// Section subtitle.
    pub subtitle: Option<String>,
    /// Section item count.
    pub count: usize,
}

/// A row of photo tiles.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhotoRow {
    /// Top edge.
    pub y: f64,
    /// Height of the row's box, which includes the bottom gap.
    pub height: f64,
    /// Index of the section.
    pub section: usize,
    /// The tiles, sorted by `x`.
    pub tiles: Vec<GridTile>,
}

impl PhotoRow {
    /// The tiles meeting the horizontal band `x` (half-open) widened by `overscan` on each side,
    /// as indices into [`Self::tiles`]. Tiles are sorted by `x`, so this is `O(log tiles)`.
    pub(crate) fn visible_tiles(&self, x: Range<f64>, overscan: f64) -> Range<usize> {
        window(
            self.tiles.len(),
            |i| (self.tiles[i].x, self.tiles[i].x + self.tiles[i].width),
            x.start,
            x.end,
            overscan,
        )
    }
}

/// A row of the model.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GridRow {
    /// A section header.
    Header(HeaderRow),
    /// Justified or square photo tiles.
    Photos(PhotoRow),
}

impl GridRow {
    /// Top edge.
    pub(crate) fn y(&self) -> f64 {
        match self {
            GridRow::Header(h) => h.y,
            GridRow::Photos(p) => p.y,
        }
    }

    /// Height of the row's box.
    pub(crate) fn height(&self) -> f64 {
        match self {
            GridRow::Header(h) => h.height,
            GridRow::Photos(p) => p.height,
        }
    }

    /// Index of the section the row belongs to.
    pub(crate) fn section(&self) -> usize {
        match self {
            GridRow::Header(h) => h.section,
            GridRow::Photos(p) => p.section,
        }
    }

    fn height_mut(&mut self) -> &mut f64 {
        match self {
            GridRow::Header(h) => &mut h.height,
            GridRow::Photos(p) => &mut p.height,
        }
    }
}

/// Where a section starts, for a scrubber.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SectionOffset {
    /// Index of the section.
    pub section: usize,
    /// Top edge of its header.
    pub y: f64,
    /// Its title.
    pub title: String,
}

/// The rows of a wall, stacked contiguously.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GridModel {
    /// Rows from top to bottom.
    pub rows: Vec<GridRow>,
    /// Sum of all row heights.
    pub total_height: f64,
    /// One entry per section.
    pub section_offsets: Vec<SectionOffset>,
}

impl GridModel {
    /// The rows meeting the vertical band `y` (half-open) widened by `overscan` on each side.
    /// `O(log rows)`.
    pub(crate) fn visible_rows(&self, y: Range<f64>, overscan: f64) -> Range<usize> {
        window(
            self.rows.len(),
            |i| (self.rows[i].y(), self.rows[i].y() + self.rows[i].height()),
            y.start,
            y.end,
            overscan,
        )
    }

    /// Every photo tile meeting the rectangle `x` by `y` (each half-open, widened by `overscan`),
    /// as `(row index, tile)` pairs. A tile's vertical extent is its row's box.
    pub(crate) fn visible_in(
        &self,
        x: Range<f64>,
        y: Range<f64>,
        overscan: f64,
    ) -> impl Iterator<Item = (usize, &GridTile)> + '_ {
        self.visible_rows(y, overscan).flat_map(move |row| {
            let (tiles, window) = match &self.rows[row] {
                GridRow::Photos(p) => (&p.tiles[..], p.visible_tiles(x.clone(), overscan)),
                GridRow::Header(_) => (&[][..], 0..0),
            };
            tiles[window].iter().map(move |t| (row, t))
        })
    }
}

/// Geometry options for [`build_grid_model`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridOptions {
    /// Width the rows fill.
    pub container_width: f64,
    /// Row height a full row is packed toward.
    pub target_row_height: f64,
    /// Space between tiles and between rows.
    pub gap: f64,
    /// Height of a section header row.
    pub header_height: f64,
    /// Extra vertical space above every section header except the first, so a header reads as
    /// attached to the photos below it. `0.0` for none.
    pub section_gap: f64,
    /// Forwarded to the justified layout: see [`JustifiedOptions::min_tile_width`].
    pub min_tile_width: f64,
}

/// Builds the row model for `aspects` split into `sections`: a header row per section, then that
/// section's justified rows. A section range past the end of `aspects` is clamped to it.
pub(crate) fn build_grid_model(
    aspects: &[f64],
    sections: &[Section],
    opts: &GridOptions,
) -> GridModel {
    let mut rows: Vec<GridRow> = Vec::new();
    let mut section_offsets = Vec::with_capacity(sections.len());
    let section_gap = if opts.section_gap.is_finite() && opts.section_gap > 0.0 {
        opts.section_gap
    } else {
        0.0
    };
    let mut y = 0.0;

    for (s, sec) in sections.iter().enumerate() {
        // Breathing room above every section except the first. Folded into the previous row's
        // height so the running offset still satisfies `row.y == sum of previous heights`.
        if s > 0 && section_gap > 0.0 {
            if let Some(last) = rows.last_mut() {
                *last.height_mut() += section_gap;
                y += section_gap;
            }
        }
        section_offsets.push(SectionOffset {
            section: s,
            y,
            title: sec.title.clone(),
        });
        rows.push(GridRow::Header(HeaderRow {
            y,
            height: opts.header_height,
            section: s,
            title: sec.title.clone(),
            subtitle: sec.subtitle.clone(),
            count: sec.count,
        }));
        y += opts.header_height;

        let end = sec.end.min(aspects.len());
        let start = sec.start.min(end);
        let layout = justified_layout(
            &aspects[start..end],
            &JustifiedOptions {
                container_width: opts.container_width,
                target_row_height: opts.target_row_height,
                gap: opts.gap,
                min_tile_width: opts.min_tile_width,
            },
        );

        let gap = if opts.gap.is_finite() && opts.gap > 0.0 {
            opts.gap
        } else {
            0.0
        };
        for lr in &layout.rows {
            let tiles = layout.tiles[lr.start..lr.end]
                .iter()
                .map(|t| GridTile {
                    item_index: start + t.index,
                    x: t.x,
                    width: t.width,
                    height: t.height,
                })
                .collect();
            // The box height folds in the bottom gap so rows stack contiguously.
            let height = lr.height + gap;
            rows.push(GridRow::Photos(PhotoRow {
                y,
                height,
                section: s,
                tiles,
            }));
            y += height;
        }
    }

    GridModel {
        rows,
        total_height: y,
        section_offsets,
    }
}

/// Options for [`square_grid_model`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SquareOptions {
    /// Width the columns fill.
    pub container_width: f64,
    /// Preferred tile side; the real side is solved so the columns fill the width.
    pub target_size: f64,
    /// Space between tiles and between rows.
    pub gap: f64,
    /// Floor under which tiles are not compressed: fewer, larger columns instead. `0.0` for none.
    pub min_tile_width: f64,
}

/// A uniform square grid (an "all photos" wall): fixed square tiles, no headers, the same
/// [`GridModel`] shape so the same windowing applies.
pub(crate) fn square_grid_model(count: usize, opts: &SquareOptions) -> GridModel {
    let (container_width, target) = (opts.container_width, opts.target_size);
    let valid = |v: f64| v.is_finite() && v > 0.0;
    if count == 0 || !valid(container_width) || !valid(target) {
        return GridModel::default();
    }
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

    let mut cols = (((container_width + gap) / (target + gap)).round() as usize).max(1);
    let side = |cols: usize| (container_width - (cols - 1) as f64 * gap) / cols as f64;
    let mut size = side(cols);
    // Floor safeguard, mirroring the justified layout's: fewer, wider (taller) square tiles
    // instead of compressing below the floor. A single column cannot be reduced further.
    while min_tile_width > 0.0 && cols > 1 && size < min_tile_width {
        cols -= 1;
        size = side(cols);
    }

    let mut rows = Vec::new();
    let mut y = 0.0;
    let mut i = 0;
    while i < count {
        let in_row = cols.min(count - i);
        let tiles = (0..in_row)
            .map(|c| GridTile {
                item_index: i + c,
                x: c as f64 * (size + gap),
                width: size,
                height: size,
            })
            .collect();
        rows.push(GridRow::Photos(PhotoRow {
            y,
            height: size + gap,
            section: 0,
            tiles,
        }));
        y += size + gap;
        i += in_row;
    }
    GridModel {
        rows,
        total_height: y,
        section_offsets: Vec::new(),
    }
}

/// The row whose box starts at or before `scroll_top`, by binary search on the row offsets:
/// row `0` for an offset above the first row (or an empty model), the last row past the end.
pub(crate) fn first_visible_row(rows: &[GridRow], scroll_top: f64) -> usize {
    super::partition_point(rows.len(), |i| rows[i].y() <= scroll_top).saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deterministic, domain-free fixtures.
    const ASPECTS: [f64; 6] = [0.66, 0.75, 1.0, 1.33, 1.5, 1.78];

    fn newest_first() -> Vec<f64> {
        (0..600).map(|i| ASPECTS[i % ASPECTS.len()]).collect()
    }

    fn sections() -> Vec<Section> {
        (0..20)
            .map(|s| Section {
                key: format!("s{s}"),
                title: format!("Section {s}"),
                subtitle: None,
                start: s * 30,
                end: s * 30 + 30,
                count: 30,
            })
            .collect()
    }

    const OPTS: GridOptions = GridOptions {
        container_width: 1200.0,
        target_row_height: 180.0,
        gap: 3.0,
        header_height: 44.0,
        section_gap: 0.0,
        min_tile_width: 0.0,
    };

    fn assert_close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} is not within {tol} of {b}");
    }

    fn photo_tiles(model: &GridModel) -> Vec<&GridTile> {
        model
            .rows
            .iter()
            .flat_map(|r| match r {
                GridRow::Photos(p) => p.tiles.iter(),
                GridRow::Header(_) => [].iter(),
            })
            .collect()
    }

    fn first_tiles(model: &GridModel) -> &[GridTile] {
        match &model.rows[0] {
            GridRow::Photos(p) => &p.tiles,
            GridRow::Header(_) => panic!("first row is a header"),
        }
    }

    mod build {
        use super::*;

        fn model() -> GridModel {
            build_grid_model(&newest_first(), &sections(), &OPTS)
        }

        #[test]
        fn starts_each_section_with_a_header_row() {
            let m = model();
            let headers = m
                .rows
                .iter()
                .filter(|r| matches!(r, GridRow::Header(_)))
                .count();
            assert_eq!(headers, sections().len());
            assert!(matches!(m.rows[0], GridRow::Header(_)));
        }

        #[test]
        fn stacks_rows_with_monotonically_increasing_offsets() {
            let m = model();
            let mut y = 0.0;
            for r in &m.rows {
                assert_close(r.y(), y, 0.01);
                y += r.height();
            }
            assert_close(m.total_height, y, 0.01);
        }

        #[test]
        fn references_every_photo_exactly_once_across_photo_rows() {
            let m = model();
            let seen: Vec<usize> = photo_tiles(&m).iter().map(|t| t.item_index).collect();
            assert_eq!(seen.len(), 600);
            let mut sorted = seen.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), 600);
            assert_eq!((sorted[0], sorted[599]), (0, 599));
        }

        #[test]
        fn keeps_photo_tiles_within_the_container_width() {
            for r in &model().rows {
                if let GridRow::Photos(p) = r {
                    let right = p.tiles.iter().map(|t| t.x + t.width).fold(0.0, f64::max);
                    assert!(right <= OPTS.container_width + 1.0);
                }
            }
        }

        #[test]
        fn exposes_per_section_vertical_offsets_for_the_scrubber() {
            let m = model();
            assert_eq!(m.section_offsets.len(), sections().len());
            assert_eq!(m.section_offsets[0].y, 0.0);
            for pair in m.section_offsets.windows(2) {
                assert!(pair[1].y > pair[0].y);
            }
        }

        #[test]
        fn handles_empty_input() {
            let m = build_grid_model(&[], &[], &OPTS);
            assert!(m.rows.is_empty());
            assert_eq!(m.total_height, 0.0);
        }

        #[test]
        fn inserts_section_gap_above_each_section_except_the_first_keeping_offsets_contiguous() {
            let base = model();
            let g = 28.0;
            let gapped = build_grid_model(
                &newest_first(),
                &sections(),
                &GridOptions {
                    section_gap: g,
                    ..OPTS
                },
            );
            // The invariant the windowing relies on: row.y == sum of previous heights.
            let mut y = 0.0;
            for r in &gapped.rows {
                assert_close(r.y(), y, 0.01);
                y += r.height();
            }
            // The first section stays flush to the top; the total grows by g per later section.
            assert_eq!(gapped.section_offsets[0].y, 0.0);
            let n = sections().len();
            assert_close(
                gapped.total_height,
                base.total_height + g * (n - 1) as f64,
                0.01,
            );
            // Each later header sits g lower than in the no-gap model.
            for i in 1..n {
                assert_close(
                    gapped.section_offsets[i].y,
                    base.section_offsets[i].y + g * i as f64,
                    0.01,
                );
            }
        }

        #[test]
        fn clamps_a_section_range_past_the_end_of_the_items() {
            let aspects = [1.0; 10];
            let sec = Section {
                key: "a".into(),
                title: "A".into(),
                subtitle: Some("sub".into()),
                start: 4,
                end: 99,
                count: 6,
            };
            let m = build_grid_model(&aspects, &[sec], &OPTS);
            let idx: Vec<usize> = photo_tiles(&m).iter().map(|t| t.item_index).collect();
            assert_eq!(idx, (4..10).collect::<Vec<_>>());
            match &m.rows[0] {
                GridRow::Header(h) => assert_eq!(h.subtitle.as_deref(), Some("sub")),
                GridRow::Photos(_) => panic!("expected a header"),
            }
        }

        #[test]
        fn forwards_the_minimum_tile_width_to_the_justified_layout() {
            let aspects = [0.82; 9];
            let sec = Section {
                key: "a".into(),
                title: "A".into(),
                subtitle: None,
                start: 0,
                end: 9,
                count: 9,
            };
            let narrow = GridOptions {
                container_width: 375.0,
                target_row_height: 210.0,
                gap: 12.0,
                header_height: 40.0,
                section_gap: 0.0,
                min_tile_width: 0.0,
            };
            let rows_of = |m: &GridModel| match &m.rows[1] {
                GridRow::Photos(p) => p.tiles.len(),
                GridRow::Header(_) => panic!("expected photos"),
            };
            assert_eq!(
                rows_of(&build_grid_model(
                    &aspects,
                    std::slice::from_ref(&sec),
                    &narrow
                )),
                3
            );
            let floored = GridOptions {
                min_tile_width: 140.0,
                ..narrow
            };
            assert_eq!(rows_of(&build_grid_model(&aspects, &[sec], &floored)), 2);
        }
    }

    mod square {
        use super::*;

        const SQ: SquareOptions = SquareOptions {
            container_width: 1000.0,
            target_size: 120.0,
            gap: 4.0,
            min_tile_width: 0.0,
        };

        #[test]
        fn packs_all_photos_into_uniform_square_tiles_with_no_headers() {
            let m = square_grid_model(50, &SQ);
            assert!(m.rows.iter().all(|r| matches!(r, GridRow::Photos(_))));
            let tiles = photo_tiles(&m);
            assert_eq!(tiles.len(), 50);
            let w = tiles[0].width;
            for t in tiles {
                assert_close(t.width, w, 1e-5);
                assert_close(t.height, w, 1e-5);
            }
        }

        #[test]
        fn fills_the_container_width_within_rounding() {
            let m = square_grid_model(40, &SQ);
            let row0 = first_tiles(&m);
            let used = row0.iter().map(|t| t.width).sum::<f64>() + SQ.gap * (row0.len() - 1) as f64;
            assert_close(used, SQ.container_width, 0.05);
        }

        #[test]
        fn references_every_photo_once_in_order_and_stacks_contiguously() {
            let m = square_grid_model(37, &SQ);
            let seen: Vec<usize> = photo_tiles(&m).iter().map(|t| t.item_index).collect();
            assert_eq!(seen, (0..37).collect::<Vec<_>>());
            let mut y = 0.0;
            for r in &m.rows {
                assert_close(r.y(), y, 1e-3);
                y += r.height();
            }
            assert_close(m.total_height, y, 1e-3);
        }

        #[test]
        fn handles_empty_input() {
            assert_eq!(
                square_grid_model(0, &SQ),
                GridModel {
                    rows: vec![],
                    total_height: 0.0,
                    section_offsets: vec![]
                }
            );
        }

        #[test]
        fn min_tile_width_reduces_columns_instead_of_compressing_tiles_below_the_floor() {
            // A target of 120 on a 375 wide container greedily picks 3 columns (about 121px
            // each); with a 140 floor it backs off to 2 wider columns.
            let narrow = SquareOptions {
                container_width: 375.0,
                target_size: 120.0,
                gap: 12.0,
                min_tile_width: 0.0,
            };
            let baseline = square_grid_model(9, &narrow);
            let row0 = first_tiles(&baseline);
            assert_eq!(row0.len(), 3);
            assert!(row0[0].width < 140.0);

            let floored = square_grid_model(
                9,
                &SquareOptions {
                    min_tile_width: 140.0,
                    ..narrow
                },
            );
            let row0 = first_tiles(&floored);
            assert_eq!(row0.len(), 2);
            assert!(row0[0].width >= 140.0);
        }

        #[test]
        fn min_tile_width_never_reduces_below_a_single_column() {
            let m = square_grid_model(
                4,
                &SquareOptions {
                    container_width: 100.0,
                    target_size: 120.0,
                    gap: 4.0,
                    min_tile_width: 500.0,
                },
            );
            assert_eq!(first_tiles(&m).len(), 1);
        }

        #[test]
        fn min_tile_width_is_a_no_op_when_the_greedy_column_count_already_clears_the_floor() {
            let without = square_grid_model(50, &SQ);
            let with = square_grid_model(
                50,
                &SquareOptions {
                    min_tile_width: 50.0,
                    ..SQ
                },
            );
            assert_eq!(with, without);
        }

        #[test]
        fn invalid_geometry_gives_the_empty_model() {
            for opts in [
                SquareOptions {
                    container_width: 0.0,
                    ..SQ
                },
                SquareOptions {
                    target_size: f64::NAN,
                    ..SQ
                },
                SquareOptions {
                    target_size: 0.0,
                    ..SQ
                },
            ] {
                assert_eq!(square_grid_model(10, &opts), GridModel::default());
            }
        }
    }

    mod first_visible {
        use super::*;

        fn rows() -> Vec<GridRow> {
            build_grid_model(&newest_first(), &sections(), &OPTS).rows
        }

        #[test]
        fn returns_0_at_the_very_top() {
            let rows = rows();
            assert_eq!(first_visible_row(&rows, 0.0), 0);
            assert_eq!(first_visible_row(&rows, -100.0), 0);
        }

        #[test]
        fn returns_the_row_whose_box_contains_the_offset() {
            let rows = rows();
            for idx in [1, 5, 20, rows.len() - 1] {
                let r = &rows[idx];
                assert_eq!(first_visible_row(&rows, r.y() + r.height() / 2.0), idx);
            }
        }

        #[test]
        fn is_the_row_at_or_before_the_offset_at_exact_boundaries() {
            let rows = rows();
            assert_eq!(first_visible_row(&rows, rows[10].y()), 10);
            // One pixel before lands on the previous row.
            assert_eq!(first_visible_row(&rows, rows[10].y() - 1.0), 9);
        }

        #[test]
        fn clamps_to_the_last_row_past_the_end() {
            let rows = rows();
            assert_eq!(first_visible_row(&rows, 1e9), rows.len() - 1);
        }

        #[test]
        fn is_0_for_an_empty_model() {
            assert_eq!(first_visible_row(&[], 5.0), 0);
        }
    }

    mod windowing {
        use super::*;

        #[test]
        fn visible_in_returns_exactly_the_tiles_whose_row_box_and_x_extent_meet_the_rect() {
            let m = build_grid_model(
                &newest_first(),
                &sections(),
                &GridOptions {
                    section_gap: 10.0,
                    ..OPTS
                },
            );
            let mut state = 12_345u64;
            let mut next = move |lo: f64, hi: f64| {
                state ^= state >> 12;
                state ^= state << 25;
                state ^= state >> 27;
                let u =
                    (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64;
                lo + u * (hi - lo)
            };
            for _ in 0..400 {
                let x0 = next(-100.0, 1300.0);
                let y0 = next(-100.0, m.total_height + 100.0);
                let x = x0..x0 + next(0.0, 900.0);
                let y = y0..y0 + next(0.0, 1500.0);
                let overscan = next(0.0, 250.0);
                let got: Vec<(usize, usize)> = m
                    .visible_in(x.clone(), y.clone(), overscan)
                    .map(|(row, t)| (row, t.item_index))
                    .collect();
                let mut want = Vec::new();
                for (i, r) in m.rows.iter().enumerate() {
                    let GridRow::Photos(p) = r else { continue };
                    if !(p.y < y.end + overscan && p.y + p.height > y.start - overscan) {
                        continue;
                    }
                    for t in &p.tiles {
                        if t.x < x.end + overscan && t.x + t.width > x.start - overscan {
                            want.push((i, t.item_index));
                        }
                    }
                }
                assert_eq!(got, want);
            }
        }

        #[test]
        fn visible_rows_includes_headers_and_matches_a_linear_scan() {
            let m = build_grid_model(&newest_first(), &sections(), &OPTS);
            for y0 in [
                -50.0,
                0.0,
                43.9,
                44.0,
                500.0,
                m.total_height - 1.0,
                m.total_height + 5.0,
            ] {
                let band = y0..y0 + 400.0;
                let got: Vec<usize> = m.visible_rows(band.clone(), 0.0).collect();
                let want: Vec<usize> = (0..m.rows.len())
                    .filter(|&i| {
                        m.rows[i].y() < band.end && m.rows[i].y() + m.rows[i].height() > band.start
                    })
                    .collect();
                assert_eq!(got, want, "y0={y0}");
            }
            assert!(m.visible_rows(0.0..1.0, 0.0).contains(&0));
        }

        #[test]
        fn square_model_windows_by_column_too() {
            let m = square_grid_model(
                10_000,
                &SquareOptions {
                    container_width: 8000.0,
                    target_size: 100.0,
                    gap: 0.0,
                    min_tile_width: 0.0,
                },
            );
            // 80 columns of 100px: a 1000px band at x = 3000 sees exactly 10 columns per row.
            let in_row: Vec<usize> = m
                .visible_in(3000.0..4000.0, 0.0..50.0, 0.0)
                .map(|(row, t)| {
                    assert_eq!(row, 0);
                    t.item_index
                })
                .collect();
            assert_eq!(in_row, (30..40).collect::<Vec<_>>());
        }
    }
}
