//! The windowed list's engine: every rule of "which rows are mounted, how tall is the canvas,
//! how far must the scroll position move", as plain Rust with no Dioxus types.
//!
//! The component owns one [`Engine`] in a `CopyValue` and treats it as an *external store*:
//! scroll messages, resize events and prop changes mutate it, and a single reactive signal tells
//! the component to render. That signal is written **only when [`Engine::needs_render`] says
//! the output would differ from the last render**, which is what quantises the scroll signal.
//! The earlier design wrote the raw scroll offset into a reactive store on every scroll event,
//! so every event re-rendered the whole list (a vdom walk plus `render_item` for every mounted
//! row) even when the mounted rows had not changed. Here a scroll event costs one `O(log N)`
//! range lookup and, most of the time, nothing else.
//!
//! # What is rendered, and when that changes
//!
//! [`RenderKey`] is the whole render output in miniature: the mounted index range, the offset of
//! its first row, and the canvas height. A render stores the key it drew; a handler publishes a
//! re-render only when the key it would draw now is different.
//!
//! The mounted range is the visible range padded by `buffer` rows each side, with hysteresis:
//! it is *kept* while the visible range stays at least `buffer / 2` rows inside it (or the
//! range already reaches the start or end of the list), and re-centred on the visible range
//! when it does not. So between `buffer / 2` and `buffer` rows are always mounted beyond each
//! edge of the viewport, and a steady scroll re-renders about once per `buffer / 2` rows
//! instead of once per row.
//!
//! # Scrollbar stability
//!
//! While the user is scrolling the canvas height is frozen at the value it had when the scroll
//! began, so rows that are measured on the way (which moves the live total) do not drag the
//! scrollbar thumb out from under the pointer; the live total is applied when scrolling stops.
//! Resizes of rows above the viewport are returned as scroll adjustments (applied at once when
//! idle, deferred until the scroll ends otherwise) so content does not jump.

use std::collections::HashMap;

use super::size_index::{SizeEstimates, SizeIndex};
use super::types::VirtualItem;
use super::utils::default_range_extractor;

/// What a render draws: when this is unchanged, a render would produce the same DOM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderKey {
    /// The mounted rows, inclusive, with overscan; `None` before the viewport is known.
    range: Option<(usize, usize)>,
    /// The offset of the first mounted row (the `translateY` of the rows' wrapper).
    top_offset: u32,
    /// The height of the scroll canvas.
    canvas_height: u32,
}

/// The rows to mount and where to put them.
pub(crate) struct Plan {
    pub(crate) items: Vec<VirtualItem>,
    pub(crate) top_offset: u32,
    /// The extent of the items alone, never `max`ed with the viewport: the viewport is `0` on the
    /// server and measured on the client, so a canvas that depended on it drew a short list
    /// differently in the server's markup and the first client frame (backlog row 368). A consumer
    /// that fills its box does it in CSS (`min-width: 100%` on the columns axis); on the rows
    /// axis the space below the last row is the scroll container's own.
    pub(crate) canvas_height: u32,
}

/// The scroll total held fixed while the user scrolls.
#[derive(Clone, Copy)]
struct FrozenTotal {
    total: u32,
    /// The item count it was taken for; a different count invalidates it.
    count: usize,
}

pub(crate) struct Engine {
    index: SizeIndex,
    scroll_offset: u32,
    viewport: u32,
    is_scrolling: bool,
    /// Scroll adjustments already applied since the current scroll began.
    scroll_adjustments: i32,
    /// Adjustments owed by rows that resized above the viewport while scrolling.
    deferred_adjustments: i32,
    frozen_total: Option<FrozenTotal>,
    /// The mounted range (inclusive), the hysteresis state.
    rendered: Option<(usize, usize)>,
    /// The key of the last render.
    drawn: Option<RenderKey>,
    /// The `buffer` of the last render (a buffer change always re-renders, so this is current).
    buffer: usize,
    /// For each measured row, its exact size minus the whole px the index holds for it (see
    /// `resize_item_exact`), and the sum of those. Offsets stay on whole px, so the canvas is
    /// padded by the rounded-up sum: it always contains the content it holds.
    fractions: HashMap<usize, f64>,
    fraction_sum: f64,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self {
            index: SizeIndex::new(SizeEstimates::evaluate(0, None), HashMap::new()),
            scroll_offset: 0,
            viewport: 0,
            is_scrolling: false,
            scroll_adjustments: 0,
            deferred_adjustments: 0,
            frozen_total: None,
            fractions: HashMap::new(),
            fraction_sum: 0.0,
            rendered: None,
            drawn: None,
            buffer: 8,
        }
    }

    /// Adopt new `count` / estimates. `O(1)` when nothing changed; `O(N)` when it did (the one
    /// place the engine walks every row), keeping every measured size.
    pub(crate) fn sync(&mut self, estimates: &SizeEstimates) {
        if self.index.sync(estimates) {
            self.rendered = None;
            let count = self.index.count();
            self.fractions.retain(|&i, _| i < count);
            self.fraction_sum = self.fractions.values().sum();
        }
    }

    /// Handle a scroll message from the container. Returns the correction to apply when a
    /// scroll ends (the sum of the resizes deferred while it ran).
    pub(crate) fn scroll(&mut self, offset: u32, viewport: u32, is_scrolling: bool) -> Option<i32> {
        let was_scrolling = self.is_scrolling;
        let mut correction = None;

        // A new scroll starts: reset adjustments and freeze the canvas height.
        if is_scrolling && !was_scrolling {
            self.scroll_adjustments = 0;
            self.deferred_adjustments = 0;
            self.frozen_total = Some(FrozenTotal {
                total: self.index.total(),
                count: self.index.count(),
            });
        }

        // The scroll ends: unfreeze, and owe the deferred adjustments.
        if !is_scrolling && was_scrolling {
            self.frozen_total = None;
            if self.deferred_adjustments != 0 {
                correction = Some(self.deferred_adjustments);
                self.deferred_adjustments = 0;
            }
        }

        self.scroll_offset = offset;
        self.is_scrolling = is_scrolling;
        self.viewport = viewport;
        correction
    }

    /// Record that the container's scroll position was moved to `offset` by a correction.
    pub(crate) fn set_scroll_offset(&mut self, offset: u32) {
        self.scroll_offset = offset;
    }

    /// The last scroll offset reported by the container.
    pub(crate) fn scroll_offset(&self) -> u32 {
        self.scroll_offset
    }

    /// `resize_item` for an item measured at `exact` px (a fraction of a px is common). The index
    /// keeps the rounded whole px, so offsets stay faithful; the remainder is remembered so the
    /// canvas can cover the content (three 66.1875px rows are 198.56px, not 198).
    pub(crate) fn resize_item_exact(&mut self, index: usize, exact: f64) -> Option<i32> {
        let adjustment = self.resize_item(index, super::bridge::measured_px(exact));
        if let Some(item) = self.index.item(index) {
            let stored = self.index.measured_size(index).unwrap_or(item.size());
            let exact = if exact.is_nan() { 0.0 } else { exact.max(1.0) };
            let fraction = exact - f64::from(stored);
            let old = if fraction.abs() < 1e-9 {
                self.fractions.remove(&index)
            } else {
                self.fractions.insert(index, fraction)
            };
            self.fraction_sum += fraction - old.unwrap_or(0.0);
        }
        adjustment
    }

    /// Px the canvas adds to the index's total so it holds its rows' exact extent.
    fn canvas_padding(&self) -> u32 {
        // 0.01 px of float noise never earns a whole px.
        (self.fraction_sum - 0.01).max(0.0).ceil() as u32
    }

    /// Handle a row's new measured height in whole px. Returns the scroll adjustment to apply,
    /// if any. Private: renderers measure through `resize_item_exact`, which keeps the fraction.
    ///
    /// `O(log N)`: one position lookup and one tree update, with no copy of the geometry.
    fn resize_item(&mut self, index: usize, new_size: u32) -> Option<i32> {
        let item = self.index.item(index)?;
        let cached = self.index.measured_size(index);

        // Already measured: ignore changes of 2px or less.
        if let Some(cached) = cached {
            if (new_size as i32 - cached as i32).abs() <= 2 {
                return None;
            }
        }

        let old_size = cached.unwrap_or(item.size());
        let delta = new_size as i32 - old_size as i32;
        if delta == 0 {
            return None;
        }
        // Sub-pixel rounding is still cached, but moves nothing.
        let significant = delta.abs() > 1;

        // Only rows above the viewport shift what the reader is looking at.
        let adjusted_scroll = (self.scroll_offset as i32 + self.scroll_adjustments).max(0) as u32;
        let above_viewport = item.start() < adjusted_scroll;

        self.index.set_measured(index, new_size);

        if significant && above_viewport {
            if self.is_scrolling {
                self.deferred_adjustments += delta;
            } else {
                self.scroll_adjustments += delta;
                return Some(delta);
            }
        }
        None
    }

    /// The total scroll size; the frozen value while scrolling.
    fn total_size(&self) -> u32 {
        let total = match self.frozen_total {
            Some(frozen) if frozen.count == self.index.count() => frozen.total,
            _ => self.index.total(),
        };
        total.saturating_add(self.canvas_padding())
    }

    /// The rows in view, `(first, last)` inclusive, from the current scroll state. `None` until
    /// the viewport is known or when the list is empty.
    fn visible(&self) -> Option<(usize, usize)> {
        let count = self.index.count();
        if count == 0 || self.viewport == 0 {
            return None;
        }
        if count == 1 {
            return Some((0, 0));
        }
        // Clamp a stale offset (the list shrank, or the container is over-scrolled).
        let max_offset = self.index.total().saturating_sub(self.viewport);
        let offset = self.scroll_offset.min(max_offset);
        let first = self.index.index_at_or_before(offset);
        let last = self
            .index
            .index_ending_at_or_after(offset.saturating_add(self.viewport))
            .max(first);
        Some((first, last))
    }

    /// Re-centre the mounted range if the visible rows have come within half a buffer of its
    /// edge. See the module doc ("What is rendered, and when that changes").
    fn refresh(&mut self) {
        let Some((first, last)) = self.visible() else {
            self.rendered = None;
            return;
        };
        let count = self.index.count();
        let slack = self.buffer / 2;
        // The row after the last visible one is mounted too (it is the half-visible neighbour
        // of the old walk and part of the established range arithmetic).
        let last = (last + 1).min(count - 1);

        let keep = self.rendered.is_some_and(|(lo, hi)| {
            hi < count
                && lo <= first
                && last <= hi
                && (lo == 0 || first >= lo + slack)
                && (hi == count - 1 || last + slack <= hi)
        });
        if !keep {
            let range = default_range_extractor(first..last, self.buffer, count);
            self.rendered = Some((*range.start(), *range.end()));
        }
    }

    fn key(&self) -> RenderKey {
        RenderKey {
            range: self.rendered,
            top_offset: self
                .rendered
                .map_or(0, |(first, _)| self.index.start(first)),
            canvas_height: self.total_size(),
        }
    }

    /// Whether rendering now would draw something different from the last render. Handlers
    /// call this after mutating the engine and write the render signal only when it is `true`.
    pub(crate) fn needs_render(&mut self) -> bool {
        self.refresh();
        Some(self.key()) != self.drawn
    }

    /// The rows to draw now. Called by the render, which thereby records what it drew.
    pub(crate) fn plan(&mut self, buffer: usize) -> Plan {
        self.buffer = buffer;
        self.refresh();
        let key = self.key();
        self.drawn = Some(key);
        let items = key
            .range
            .map(|(first, last)| (first..=last).filter_map(|i| self.index.item(i)).collect())
            .unwrap_or_default();
        Plan {
            items,
            top_offset: key.top_offset,
            canvas_height: key.canvas_height,
        }
    }

    /// The bottom spacer, in px, for a renderer whose mounted items sit in flow at their exact
    /// size (`TableRows`' `<tr>`s), where the canvas's rounded-up padding would add a blank px:
    /// the exact extent below the last mounted item, i.e. the canvas without its padding but
    /// with every remainder, less `end` and the remainders the mounted items already occupy.
    /// So a short body whose rows are all mounted is exactly its rows' height (row 369).
    pub(crate) fn flow_bottom(&self, plan: &Plan) -> f64 {
        let end = plan
            .items
            .last()
            .map_or(plan.top_offset, |last| last.start() + last.size());
        let mounted: f64 = plan
            .items
            .iter()
            .filter_map(|item| self.fractions.get(&item.index()))
            .sum();
        let whole = plan.canvas_height.saturating_sub(self.canvas_padding());
        let bottom = f64::from(whole) + self.fraction_sum - f64::from(end) - mounted;
        // Float noise below 0.01 px is no spacer.
        if bottom < 0.01 {
            0.0
        } else {
            bottom
        }
    }

    /// Item `index` at its current position, whether or not it is mounted: what a consumer
    /// needs to draw one extra item outside the window (the grid's focused cell).
    pub(crate) fn item(&self, index: usize) -> Option<VirtualItem> {
        self.index.item(index)
    }

    /// Whether any item has been measured: until one is, the canvas height is the estimates' sum,
    /// the same on the server as on the client's first frame.
    pub(crate) fn has_measurements(&self) -> bool {
        self.index.has_measured()
    }

    /// Drop every measured size, keeping the scroll state: for a consumer whose item geometry
    /// changed meaning (a wrapping grid whose column count changed, so row `i` holds other
    /// items). The next [`sync`](Self::sync) rebuilds the index from the estimates alone.
    pub(crate) fn forget_measurements(&mut self) {
        self.index = SizeIndex::new(SizeEstimates::evaluate(0, None), HashMap::new());
        self.fractions.clear();
        self.fraction_sum = 0.0;
        self.rendered = None;
    }

    /// The scroll offset that centres item `index` in the viewport, clamped to the scroll range:
    /// the core's `scroll_to(index)` (row 154 layer 1). `None` past the end. Before the viewport
    /// is known it is the item's start.
    pub(crate) fn scroll_target(&self, index: usize) -> Option<u32> {
        let item = self.index.item(index)?;
        let centre = (item.start() + item.size() / 2).saturating_sub(self.viewport / 2);
        let max = self.index.total().saturating_sub(self.viewport);
        Some(centre.min(max))
    }

    /// How many items the viewport shows now (at least 1 once the viewport is known, 0 before):
    /// the step of PageUp / PageDown.
    pub(crate) fn visible_len(&self) -> usize {
        self.visible().map_or(0, |(first, last)| last - first + 1)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(count: usize, size: u32) -> SizeEstimates {
        let estimate: &dyn Fn(usize) -> u32 = &move |_| size;
        SizeEstimates::evaluate(count, Some(estimate))
    }

    /// An engine over `count` rows of `size` px with a 600px viewport at `offset`, already drawn.
    fn engine(count: usize, size: u32, offset: u32, buffer: usize) -> Engine {
        let mut engine = Engine::new();
        engine.sync(&uniform(count, size));
        engine.scroll(offset, 600, false);
        engine.plan(buffer);
        engine
    }

    fn mounted(plan: &Plan) -> Vec<usize> {
        plan.items.iter().map(VirtualItem::index).collect()
    }

    #[test]
    fn nothing_is_mounted_until_the_viewport_is_known() {
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        let plan = engine.plan(8);
        assert!(plan.items.is_empty());
        assert_eq!(plan.canvas_height, 5000);
    }

    #[test]
    fn mounts_the_visible_rows_plus_the_buffer() {
        // 600px / 50px = rows 0..=11 visible, plus the half-visible neighbour, plus 8 below.
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        engine.scroll(0, 600, false);
        let plan = engine.plan(8);
        assert_eq!(mounted(&plan), (0..=20).collect::<Vec<_>>());
        assert_eq!(plan.top_offset, 0);
        // And in the middle: rows 20..=31 visible.
        engine.scroll(1000, 600, false);
        let plan = engine.plan(8);
        assert_eq!(mounted(&plan), (12..=40).collect::<Vec<_>>());
        assert_eq!(plan.top_offset, 600);
    }

    #[test]
    fn range_clamps_a_stale_scroll_offset_after_the_count_shrinks() {
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        engine.scroll(10_000, 600, false);
        engine.plan(0);
        engine.sync(&uniform(4, 50));
        let plan = engine.plan(0);
        assert_eq!(mounted(&plan), vec![0, 1, 2, 3]);
    }

    #[test]
    fn the_canvas_covers_fractional_rows_while_offsets_stay_on_whole_px() {
        let mut engine = Engine::new();
        engine.sync(&SizeEstimates::evaluate(3, Some(&|_| 68)));
        let _ = engine.scroll(0, 400, false);
        for i in 0..3 {
            engine.resize_item_exact(i, 66.1875);
        }
        // Offsets: 66px apart (nearest), not drifting by the fraction.
        assert_eq!(engine.item(2).unwrap().start(), 132);
        // Canvas: 3 x 66.1875 = 198.5625 px of content, so 199, never 198.
        assert_eq!(engine.plan(8).canvas_height, 199);
        // Whole-px rows add nothing.
        let mut whole = Engine::new();
        whole.sync(&SizeEstimates::evaluate(3, Some(&|_| 68)));
        let _ = whole.scroll(0, 400, false);
        for i in 0..3 {
            whole.resize_item_exact(i, 66.0);
        }
        assert_eq!(whole.plan(8).canvas_height, 198);
    }

    #[test]
    fn flow_bottom_holds_the_exact_rest_and_nothing_for_a_fully_mounted_body() {
        // A short body: all three rows mounted in flow at 66.1875px, so no spacer at all (the
        // canvas's padding px would be a blank px under the last row).
        let mut short = Engine::new();
        short.sync(&SizeEstimates::evaluate(3, Some(&|_| 68)));
        let _ = short.scroll(0, 400, false);
        for i in 0..3 {
            short.resize_item_exact(i, 66.1875);
        }
        let plan = short.plan(8);
        assert_eq!(plan.items.len(), 3);
        assert_eq!(short.flow_bottom(&plan), 0.0);
        // A long body: top spacer + the mounted rows at their exact size + the bottom spacer is
        // the exact extent of every row.
        let mut long = Engine::new();
        long.sync(&uniform(100, 50));
        let _ = long.scroll(0, 100, false);
        for i in 0..4 {
            long.resize_item_exact(i, 66.1875);
        }
        let plan = long.plan(0);
        let mounted: f64 = plan
            .items
            .iter()
            .map(|item| if item.index() < 4 { 66.1875 } else { 50.0 })
            .sum();
        let flow = f64::from(plan.top_offset) + mounted + long.flow_bottom(&plan);
        assert!(
            (flow - (4.0 * 66.1875 + 96.0 * 50.0)).abs() < 1e-9,
            "{flow}"
        );
    }

    #[test]
    fn forget_measurements_drops_the_fractions_too() {
        // The grid forgets its rows on a column change; a stale remainder would pad the canvas
        // of rows that are no longer measured.
        let mut engine = Engine::new();
        engine.sync(&SizeEstimates::evaluate(3, Some(&|_| 68)));
        let _ = engine.scroll(0, 400, false);
        for i in 0..3 {
            engine.resize_item_exact(i, 66.1875);
        }
        engine.forget_measurements();
        engine.sync(&SizeEstimates::evaluate(3, Some(&|_| 68)));
        assert_eq!(engine.plan(8).canvas_height, 204);
    }

    #[test]
    fn frozen_total_is_ignored_when_the_count_changes() {
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        engine.scroll(1000, 600, true);
        assert_eq!(engine.plan(8).canvas_height, 5000);
        engine.sync(&uniform(4, 50));
        assert_eq!(engine.plan(8).canvas_height, 200); // the rows' extent, not the viewport's
    }

    #[test]
    fn a_short_list_has_one_canvas_before_and_after_the_viewport_is_known() {
        // Row 368: the server has no viewport (0), the client measures one; four rows of 50px
        // are 200px on both, whatever the box that holds them.
        let mut engine = Engine::new();
        engine.sync(&uniform(4, 50));
        let server = engine.plan(8).canvas_height;
        engine.scroll(0, 600, false);
        assert_eq!((server, engine.plan(8).canvas_height), (200, 200));
    }

    #[test]
    fn the_canvas_height_is_frozen_while_scrolling_and_released_after() {
        let mut engine = engine(100, 50, 0, 8);
        engine.scroll(400, 600, true);
        assert_eq!(engine.plan(8).canvas_height, 5000);
        // A row measured mid-scroll moves the live total, not the canvas: no render is owed.
        engine.resize_item(60, 150);
        assert_eq!(engine.index.total(), 5100);
        assert!(!engine.needs_render());
        assert_eq!(engine.plan(8).canvas_height, 5000);
        // When the scroll ends the canvas takes the live total, and a render is owed.
        engine.scroll(400, 600, false);
        assert!(engine.needs_render());
        assert_eq!(engine.plan(8).canvas_height, 5100);
    }

    #[test]
    fn resizing_a_row_below_the_viewport_adjusts_nothing() {
        let mut engine = engine(100, 50, 0, 8);
        assert_eq!(engine.resize_item(50, 100), None);
        assert_eq!(engine.index.size(50), 100);
    }

    #[test]
    fn resizing_a_row_above_the_viewport_returns_the_scroll_adjustment() {
        let mut engine = engine(100, 50, 1000, 8);
        assert_eq!(engine.resize_item(5, 100), Some(50));
        // The adjustment is remembered: the next row's "above the viewport" test uses it.
        assert_eq!(engine.scroll_adjustments, 50);
    }

    #[test]
    fn resizes_while_scrolling_are_deferred_until_the_scroll_ends() {
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        engine.scroll(1000, 600, true);
        assert_eq!(
            engine.resize_item(5, 100),
            None,
            "no adjustment during a scroll"
        );
        assert_eq!(
            engine.resize_item(3, 80),
            None,
            "no adjustment during a scroll"
        );
        let correction = engine.scroll(1000, 600, false);
        assert_eq!(correction, Some(80), "50 + 30");
    }

    #[test]
    fn rows_below_the_viewport_owe_no_deferred_adjustment() {
        let mut engine = Engine::new();
        engine.sync(&uniform(100, 50));
        engine.scroll(1000, 600, true);
        assert_eq!(engine.resize_item(50, 100), None);
        assert_eq!(engine.scroll(1000, 600, false), None);
    }

    #[test]
    fn small_remeasures_are_ignored() {
        let mut engine = engine(100, 50, 1000, 8);
        assert_eq!(engine.resize_item(5, 100), Some(50));
        assert_eq!(
            engine.resize_item(5, 102),
            None,
            "within 2px of the cached size"
        );
        assert_eq!(engine.index.size(5), 100);
        assert_eq!(engine.resize_item(5, 103), Some(3));
        assert_eq!(engine.index.size(5), 103);
    }

    #[test]
    fn a_scroll_inside_the_slack_does_not_render() {
        // Uniform 50px rows, buffer 12: after the first render the mounted range reaches 12 rows
        // past each edge, and the range is only re-centred once fewer than 6 rows remain.
        let mut engine = engine(10_000, 50, 5_000, 12);
        let mut renders = 0;
        for offset in (5_001..=5_000 + 6 * 50 - 1).step_by(7) {
            engine.scroll(offset, 600, true);
            if engine.needs_render() {
                renders += 1;
                engine.plan(12);
            }
        }
        assert_eq!(
            renders, 0,
            "5 rows of scroll consume less than the 6-row slack"
        );
        // Crossing the slack renders once, and re-centres.
        engine.scroll(5_000 + 7 * 50, 600, true);
        assert!(engine.needs_render());
    }

    #[test]
    fn a_long_scroll_renders_once_per_half_buffer_not_once_per_event() {
        let events: Vec<u32> = (0..=2_000).map(|i| 5_000 + i * 5).collect(); // 10,000px in 5px steps
        let mut engine = engine(10_000, 50, 5_000, 12);
        let mut renders = 0;
        for offset in &events {
            engine.scroll(*offset, 600, true);
            if engine.needs_render() {
                renders += 1;
                engine.plan(12);
            }
        }
        // 10,000px = 200 rows; one render per ~6 rows of slack is ~33. Never one per event.
        assert!(
            (20..=45).contains(&renders),
            "{renders} renders for {} events",
            events.len()
        );
    }

    #[test]
    fn scroll_target_centres_the_item_and_clamps_to_the_scroll_range() {
        // 100 rows of 50px, 600px viewport: row 40 (2000..2050) centred is 2025 - 300.
        let engine = engine(100, 50, 0, 8);
        assert_eq!(engine.scroll_target(40), Some(1725));
        // Near the ends the target is clamped to 0 and to total - viewport.
        assert_eq!(engine.scroll_target(2), Some(0));
        assert_eq!(engine.scroll_target(99), Some(4400));
        assert_eq!(engine.scroll_target(100), None);
    }

    #[test]
    fn forget_measurements_keeps_the_scroll_state_and_drops_the_sizes() {
        let mut engine = engine(100, 50, 1000, 8);
        engine.resize_item(30, 150);
        engine.forget_measurements();
        engine.sync(&uniform(100, 50));
        assert_eq!(engine.item(31).map(|i| i.start()), Some(1550));
        assert_eq!(engine.scroll_offset(), 1000);
        assert_eq!(engine.visible_len(), 12);
    }

    #[test]
    fn the_visible_rows_are_always_mounted() {
        // Property: whatever the scroll path, every row in view is in the mounted range once a
        // render has caught up. Mixed row heights, jumps and creeps.
        let estimate: &dyn Fn(usize) -> u32 = &|i| 20 + ((i * 37) % 130) as u32;
        let mut engine = Engine::new();
        engine.sync(&SizeEstimates::evaluate(5_000, Some(estimate)));
        engine.scroll(0, 640, false);
        engine.plan(10);
        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut offset = 0u32;
        for _ in 0..3_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            offset = match state % 10 {
                0 => (state >> 8) as u32 % 400_000, // a jump
                1..=4 => offset.saturating_add((state >> 8) as u32 % 40), // creep down
                _ => offset.saturating_sub((state >> 8) as u32 % 40), // creep up
            };
            engine.scroll(offset, 640, true);
            if engine.needs_render() {
                engine.plan(10);
            }
            let (first, last) = engine.visible().unwrap();
            let (lo, hi) = engine.rendered.unwrap();
            assert!(
                lo <= first && last <= hi,
                "visible {first}..={last} outside mounted {lo}..={hi}"
            );
        }
    }

    #[test]
    fn a_zero_buffer_mounts_only_what_is_needed() {
        let mut engine = engine(1_000, 50, 0, 0);
        let first = mounted(&engine.plan(0));
        assert_eq!(first, (0..=12).collect::<Vec<_>>());
        // 1px later row 12 comes into view and its neighbour 13 is needed: one render. Rows then
        // keep arriving at the far edge one per 50px; row 0 leaving at the top owes nothing.
        engine.scroll(1, 600, true);
        assert!(engine.needs_render());
        assert_eq!(mounted(&engine.plan(0)), (0..=13).collect::<Vec<_>>());
        engine.scroll(50, 600, true);
        assert!(!engine.needs_render());
        engine.scroll(51, 600, true);
        assert!(engine.needs_render());
        assert_eq!(mounted(&engine.plan(0)), (1..=14).collect::<Vec<_>>());
    }

    #[test]
    fn measuring_a_visible_row_re_renders_only_when_the_geometry_moves() {
        let mut engine = engine(100, 50, 0, 8);
        // A row measured exactly at its estimate changes nothing.
        assert_eq!(engine.resize_item(3, 50), None);
        assert!(!engine.needs_render());
        // A real change moves the total, so the canvas height: one render.
        engine.resize_item(4, 90);
        assert!(engine.needs_render());
    }

    #[test]
    fn a_resize_of_the_viewport_re_renders_when_more_rows_fit() {
        let mut engine = engine(1_000, 50, 0, 4);
        engine.scroll(0, 1_200, false);
        assert!(engine.needs_render());
        let plan = engine.plan(4);
        assert_eq!(*mounted(&plan).last().unwrap(), 24 + 4);
    }

    #[test]
    fn a_reordered_list_keeps_each_items_measured_size() {
        let keyed = |order: &[u64]| {
            uniform(order.len(), 50).with_keys(Some(order.iter().copied().collect()))
        };
        let mut engine = Engine::new();
        engine.sync(&keyed(&[1, 2, 3, 4]));
        engine.scroll(0, 100, false);
        engine.plan(0);
        // Item 1 (at the top) measures 130 px, item 4 stays estimated.
        engine.resize_item(0, 130);
        assert_eq!(engine.index.total(), 130 + 150);
        // Reversed: the 130 px item is last, nothing is owed and the total is unchanged.
        engine.sync(&keyed(&[4, 3, 2, 1]));
        assert_eq!(engine.index.total(), 280);
        assert_eq!(engine.index.size(0), 50);
        assert_eq!(engine.index.size(3), 130);
        assert_eq!(engine.index.start(3), 150);
    }
}
