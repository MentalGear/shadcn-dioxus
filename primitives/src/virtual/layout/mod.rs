//! Pure, DOM-free layout math for windowed photo walls.
//!
//! Nothing here touches Dioxus, the DOM or a signal: every function takes numbers and returns
//! numbers, so the geometry is unit-tested natively and can be recomputed off the render path
//! whenever the container resizes or the zoom changes.
//!
//! - [`justified`]: the justified-rows packer. Aspect ratios in, rows and positioned tiles out,
//!   with a minimum-tile-width floor that backs a row off to fewer, wider tiles rather than
//!   cramping them, plus two-axis windowing (`visible_rows`, per row `visible_tiles`) by binary
//!   search, and the RTL mirroring helpers.
//! - [`grid_model`]: the flat row model a sectioned wall renders: a header row per section
//!   followed by that section's justified rows (or fixed square rows), with the same windowing.
//! - [`stride_window`]: uniform-stride track arithmetic ("which item is centred", "which items
//!   are visible") for a horizontal strip of equally sized items.
//!
//! # Number type
//!
//! Positions and sizes are `f64`, unlike the integer pixel sizes of [`super::size_index`]. A
//! justified row's height is a quotient (`(width - gaps) / sum of aspects`), so the tiles of a row
//! only add up to the container width edge to edge in floating point; rounding each tile to whole
//! pixels would leave a visible hairline at the right edge. `f64` also keeps a canvas of hundreds
//! of thousands of pixels exact, where `f32` would start dropping sub-pixel positions.
//!
//! # Windowing
//!
//! Rows are stacked top to bottom with non-decreasing `y` and bottom edges, and the tiles of a row
//! are laid left to right with non-decreasing `x` and right edges. Both are therefore sorted, so
//! "which rows meet this vertical band" and "which tiles of this row meet this horizontal band"
//! are each two binary searches ([`window`]), `O(log n)` regardless of how many items exist.
//! A box meets a band when the two half-open intervals overlap; the optional overscan widens the
//! band by that many pixels on each side.

// `VirtualJustifiedGrid` (through `super::justified_engine`) consumes the justified packer; the
// sectioned model and the strip arithmetic have no consumer outside their tests, which exercise all of it.
#![allow(
    dead_code,
    reason = "the sectioned model and the strip arithmetic are used only by their tests, so it fires outside test builds"
)]

use std::ops::Range;

pub(crate) mod grid_model;
pub(crate) mod justified;
pub(crate) mod stride_window;

/// The number of leading indices `0..len` for which `pred` holds, given that `pred` is true for a
/// prefix and false afterwards (the classic partition point, over an index space rather than a
/// slice so callers can read their geometry in place).
pub(crate) fn partition_point(len: usize, pred: impl Fn(usize) -> bool) -> usize {
    let (mut lo, mut hi) = (0, len);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The sub-range of `0..len` whose spans meet `[lo, hi)` widened by `overscan` on each side.
///
/// `span(i)` is the half-open `(start, end)` extent of item `i` along one axis, and both `start`
/// and `end` must be non-decreasing in `i`. The result is a valid, possibly empty range inside
/// `0..len`. A NaN bound yields the empty range, a band of zero width matches nothing unless the
/// overscan widens it, and a negative or non-finite overscan counts as zero.
pub(crate) fn window(
    len: usize,
    span: impl Fn(usize) -> (f64, f64),
    lo: f64,
    hi: f64,
    overscan: f64,
) -> Range<usize> {
    let overscan = if overscan.is_finite() && overscan > 0.0 {
        overscan
    } else {
        0.0
    };
    let (lo, hi) = (lo - overscan, hi + overscan);
    if lo.is_nan() || hi.is_nan() {
        return 0..0;
    }
    // First item whose end is past the band's start, and first item that starts at or after the
    // band's end: everything between them overlaps.
    let first = partition_point(len, |i| span(i).1 <= lo);
    let last = partition_point(len, |i| span(i).0 < hi);
    first..last.max(first)
}

/// Mirrors an item's left edge across a container of `container_width` for a right-to-left
/// layout: `x' = container_width - x - width`. Its own inverse.
pub(crate) fn mirror_x(container_width: f64, x: f64, width: f64) -> f64 {
    container_width - x - width
}

/// Mirrors a horizontal band across the container, so a physical right-to-left viewport can be
/// asked of a layout computed left to right: window the mirrored band, then mirror the tiles.
pub(crate) fn mirror_range(container_width: f64, band: Range<f64>) -> Range<f64> {
    (container_width - band.end)..(container_width - band.start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_point_finds_the_prefix_length() {
        for len in 0..12 {
            for cut in 0..=len {
                assert_eq!(partition_point(len, |i| i < cut), cut);
            }
        }
    }

    #[test]
    fn window_matches_a_linear_scan_on_unit_spans() {
        // Item i occupies [i, i + 1).
        let span = |i: usize| (i as f64, i as f64 + 1.0);
        for lo in [-3.0, 0.0, 0.5, 4.0, 9.5, 20.0] {
            for width in [0.0, 0.25, 1.0, 3.5, 100.0] {
                for overscan in [0.0, 0.5, 2.0] {
                    let got = window(10, span, lo, lo + width, overscan);
                    let want: Vec<usize> = (0..10)
                        .filter(|&i| {
                            let (s, e) = span(i);
                            s < lo + width + overscan && e > lo - overscan
                        })
                        .collect();
                    let got_vec: Vec<usize> = got.clone().collect();
                    assert_eq!(got_vec, want, "lo={lo} width={width} overscan={overscan}");
                }
            }
        }
    }

    #[test]
    fn window_rejects_nan_and_ignores_a_bad_overscan() {
        let span = |i: usize| (i as f64, i as f64 + 1.0);
        assert_eq!(window(5, span, f64::NAN, 3.0, 0.0), 0..0);
        assert_eq!(window(5, span, 0.0, f64::NAN, 0.0), 0..0);
        assert_eq!(
            window(5, span, 1.0, 3.0, f64::NAN),
            window(5, span, 1.0, 3.0, 0.0)
        );
        assert_eq!(
            window(5, span, 1.0, 3.0, -50.0),
            window(5, span, 1.0, 3.0, 0.0)
        );
        assert_eq!(window(0, span, 0.0, 10.0, 0.0), 0..0);
    }

    #[test]
    fn mirror_is_an_involution() {
        let (cw, x, w) = (1000.0, 130.0, 220.0);
        let m = mirror_x(cw, x, w);
        assert_eq!(m, 650.0);
        assert_eq!(mirror_x(cw, m, w), x);
        let band = 100.0..400.0;
        let mirrored = mirror_range(cw, band.clone());
        assert_eq!(mirrored, 600.0..900.0);
        assert_eq!(mirror_range(cw, mirrored), band);
    }
}
