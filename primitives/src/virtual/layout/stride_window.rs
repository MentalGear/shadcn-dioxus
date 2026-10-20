//! Uniform-stride track geometry: the arithmetic behind "which item is centred" and "which items
//! are visible" for a horizontally scrolling strip whose items are all the same size.
//!
//! This maths fails quietly: an off-by-half-a-gap centring error, or a window that excludes the
//! item it just centred on, reads as "the strip feels slightly off" rather than as a bug, and no
//! end-to-end assertion about which item is selected catches it. Keeping it pure lets the
//! invariant that matters, that **the centred index is always inside the visible range**, be
//! asserted directly rather than hoped for.
//!
//! # What it deliberately does not know
//!
//! Edge padding policy. `origin` is just "the content-space x where item 0 starts"; whether that
//! came from half a viewport of reserved space (so any item can reach the centre) or from plain
//! padding is the caller's business. That is what lets one module serve a centred-active rail and
//! an ordinary left-aligned one without a mode.

use std::ops::Range;

/// A track of `count` equally spaced items.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StrideTrack {
    /// Distance from one item's start to the next, in content px: item size plus any gap.
    pub stride: f64,
    /// Total number of items in the track (not the rendered window).
    pub count: usize,
    /// Content-space x of item 0's start.
    pub origin: f64,
    /// The item's own rendered size, when it differs from `stride` (there is a gap). Only affects
    /// centring, which must target the item's visual middle rather than its slot's middle.
    /// `None` means `stride`.
    pub item_size: Option<f64>,
}

impl StrideTrack {
    /// A track with `origin` 0 and no separate item size.
    pub(crate) fn new(stride: f64, count: usize) -> Self {
        Self {
            stride,
            count,
            origin: 0.0,
            item_size: None,
        }
    }

    /// The same track starting at content-space `origin`.
    pub(crate) fn with_origin(mut self, origin: f64) -> Self {
        self.origin = origin;
        self
    }

    /// The same track with items of `item_size` inside each stride.
    pub(crate) fn with_item_size(mut self, item_size: f64) -> Self {
        self.item_size = Some(item_size);
        self
    }

    fn item_size(&self) -> f64 {
        self.item_size.unwrap_or(self.stride)
    }
}

/// Content-space x of an item's visual centre: what a "centre on this x" call wants.
pub(crate) fn item_center_offset(index: usize, track: &StrideTrack) -> f64 {
    track.origin + index as f64 * track.stride + track.item_size() / 2.0
}

/// The index whose centre is nearest the viewport's centre, clamped to `[0, count - 1]`.
///
/// The inverse of [`item_center_offset`] at the centring scroll position; the round trip is
/// asserted in the tests, because a drift between the two is invisible until it is a whole item
/// wide. `0` for an empty track or a non-positive stride: an item size driven by a live resize
/// gesture can legitimately report 0 for a frame, and dividing by it would give infinity.
pub(crate) fn index_at_center(scroll_left: f64, viewport: f64, track: &StrideTrack) -> usize {
    if track.count == 0 || track.stride <= 0.0 || track.stride.is_nan() {
        return 0;
    }
    let centre = scroll_left + viewport / 2.0;
    let raw = ((centre - track.origin - track.item_size() / 2.0) / track.stride).round();
    // Clamp in f64 before the cast: NaN becomes the first index, never a wrapped value.
    raw.clamp(0.0, (track.count - 1) as f64) as usize
}

/// The half-open index range `[first, last)` covering the viewport, widened by `buffer` px on
/// each side (the overscan a virtualized rail needs so items are mounted before they scroll into
/// view rather than popping in at the edge).
///
/// Clamped to the track and never inverted: scrolled far past the end it collapses to an empty
/// range at `count`. A non-positive stride renders the whole track for the frame rather than
/// dividing by it.
pub(crate) fn visible_range(
    scroll_left: f64,
    viewport: f64,
    track: &StrideTrack,
    buffer: f64,
) -> Range<usize> {
    let count = track.count;
    if count == 0 {
        return 0..0;
    }
    if track.stride <= 0.0 || track.stride.is_nan() {
        return 0..count;
    }
    let left = scroll_left - buffer - track.origin;
    let right = scroll_left + viewport + buffer - track.origin;
    let clamp = |v: f64| v.clamp(0.0, count as f64) as usize;
    let first = clamp((left / track.stride).floor());
    let last = clamp((right / track.stride).ceil()).max(first);
    first..last
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A realistic strip geometry, so the tests exercise numbers that ship rather than round
    /// ones: a thumbnail of 44 plus a 4 gap gives a stride of 48, a 16 left padding, and an edge
    /// pad of half the viewport so the first and last item can still reach the visual centre.
    fn strip() -> StrideTrack {
        StrideTrack::new(48.0, 25_000)
            .with_origin(300.0 + 16.0)
            .with_item_size(44.0)
    }

    mod item_center {
        use super::*;

        #[test]
        fn places_item_0_half_an_item_past_the_origin_not_at_it() {
            // `item_size / 2`, not `stride / 2`: the stride bakes in the inter-item gap, which is
            // not part of the item's own visual centre. Getting this wrong offsets every centred
            // item by half a gap.
            assert_eq!(item_center_offset(0, &strip()), 316.0 + 22.0);
        }

        #[test]
        fn advances_by_exactly_one_stride_per_index() {
            let t = strip();
            assert_eq!(
                item_center_offset(5, &t) - item_center_offset(4, &t),
                t.stride
            );
        }

        #[test]
        fn defaults_item_size_to_stride() {
            assert_eq!(item_center_offset(0, &StrideTrack::new(48.0, 10)), 24.0);
        }

        #[test]
        fn defaults_origin_to_0() {
            assert_eq!(
                item_center_offset(2, &StrideTrack::new(10.0, 10).with_item_size(10.0)),
                25.0
            );
        }
    }

    mod index_at_center_tests {
        use super::*;

        #[test]
        fn round_trips_with_item_center_offset() {
            let t = strip();
            // The property that matters: if the two disagree, navigation and the "which item is
            // centred" readout drift apart, invisibly until it is a whole item.
            for i in [0usize, 1, 7, 100, 12_499, 24_999] {
                let scroll_left = item_center_offset(i, &t) - 600.0 / 2.0;
                assert_eq!(index_at_center(scroll_left, 600.0, &t), i);
            }
        }

        #[test]
        fn rounds_to_the_nearest_item() {
            let t = strip();
            let centred = item_center_offset(10, &t) - 300.0;
            assert_eq!(index_at_center(centred + t.stride * 0.49, 600.0, &t), 10);
            assert_eq!(index_at_center(centred + t.stride * 0.51, 600.0, &t), 11);
        }

        #[test]
        fn clamps_to_the_item_range() {
            let t = StrideTrack {
                count: 3,
                ..strip()
            };
            assert_eq!(index_at_center(-10_000.0, 600.0, &t), 0);
            assert_eq!(index_at_center(10_000.0, 600.0, &t), 2);
        }

        #[test]
        fn returns_0_for_an_empty_track() {
            assert_eq!(index_at_center(0.0, 600.0, &StrideTrack::new(48.0, 0)), 0);
        }

        #[test]
        fn returns_0_for_a_zero_or_negative_stride() {
            assert_eq!(index_at_center(500.0, 600.0, &StrideTrack::new(0.0, 10)), 0);
            assert_eq!(
                index_at_center(500.0, 600.0, &StrideTrack::new(-48.0, 10)),
                0
            );
            assert_eq!(
                index_at_center(500.0, 600.0, &StrideTrack::new(f64::NAN, 10)),
                0
            );
        }

        #[test]
        fn a_nan_scroll_offset_is_the_first_item() {
            assert_eq!(index_at_center(f64::NAN, 600.0, &strip()), 0);
        }
    }

    mod visible_range_tests {
        use super::*;

        #[test]
        fn returns_a_half_open_range_covering_exactly_the_viewport() {
            // Origin 0 keeps the arithmetic checkable by hand: a 600px viewport at scroll 0 over
            // a 48px stride shows items 0..12 (12 * 48 = 576 < 600, 13 * 48 = 624 > 600).
            let t = StrideTrack::new(48.0, 1000);
            assert_eq!(visible_range(0.0, 600.0, &t, 0.0), 0..13);
        }

        #[test]
        fn widens_by_the_buffer_on_both_sides() {
            let t = StrideTrack::new(48.0, 1000);
            let plain = visible_range(4800.0, 600.0, &t, 0.0);
            let buffered = visible_range(4800.0, 600.0, &t, 300.0);
            assert!(buffered.start < plain.start);
            assert!(buffered.end > plain.end);
            assert_eq!(
                plain.start - buffered.start,
                (300.0f64 / 48.0).ceil() as usize
            );
        }

        #[test]
        fn clamps_to_the_track() {
            let t = StrideTrack::new(48.0, 5);
            assert_eq!(visible_range(0.0, 6000.0, &t, 1000.0), 0..5);
        }

        #[test]
        fn never_returns_last_below_first_even_scrolled_far_past_the_end() {
            let t = StrideTrack::new(48.0, 5);
            let r = visible_range(100_000.0, 600.0, &t, 0.0);
            assert!(r.end >= r.start);
            assert_eq!(r, 5..5);
        }

        #[test]
        fn is_empty_for_an_empty_track() {
            assert_eq!(
                visible_range(0.0, 600.0, &StrideTrack::new(48.0, 0), 0.0),
                0..0
            );
        }

        #[test]
        fn degrades_to_the_whole_track_for_a_zero_stride() {
            assert_eq!(
                visible_range(0.0, 600.0, &StrideTrack::new(0.0, 7), 0.0),
                0..7
            );
        }

        #[test]
        fn always_contains_the_centred_index() {
            // If this fails, a consumer can centre on an item it has not rendered: the "active
            // thumb is blank" class of bug a virtualized rail is prone to.
            let t = strip();
            for scroll_left in [0.0, 137.0, 5000.0, 250_000.0, 1_199_000.0] {
                let c = index_at_center(scroll_left, 600.0, &t);
                let r = visible_range(scroll_left, 600.0, &t, 0.0);
                assert!(
                    c >= r.start && c < r.end,
                    "centre {c} outside {r:?} at {scroll_left}"
                );
            }
        }
    }
}
