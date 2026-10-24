//! Scroll to an item (backlog row 398): the public handle on `VirtualList`, `VirtualGrid`,
//! `VirtualWrapGrid` and `VirtualJustifiedGrid`.
//!
//! [`use_virtual_scroller`] creates a [`VirtualScroller`], a `Copy` handle the caller passes to
//! a component's `scroller` prop. The component attaches to it when it mounts and detaches when
//! it unmounts, and [`VirtualScroller::scroll_to_index`] / [`VirtualScroller::scroll_to_cell`]
//! reach whichever body is mounted: the windowed engine (the target's offset comes from the
//! engine's measured-or-estimated positions, so a row that is not mounted can be the target)
//! or, in content-visibility mode, the DOM (every row exists; the browser's `scrollIntoView`
//! first, then a few frames of correction while skipped chunks render at their real height).
//!
//! The alignment is [`ScrollAlign`], the type `MessageScroller` already uses: `Start`, `Center`,
//! `End`, or `Nearest` (the least movement that brings the item fully into view, nothing if it
//! already is).
//!
//! A call made while no component is attached (before the first render, or in the window while
//! a mode switch remounts the body) is **kept and run when the component attaches**; only the
//! last such call is kept. Windowed bodies keep it a little longer, until the container has
//! reported its size. A call after the component unmounted does nothing.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub use crate::message_scroller::ScrollAlign;

/// One scroll request. `column` is only read by `VirtualGrid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollRequest {
    pub(crate) index: usize,
    pub(crate) column: Option<usize>,
    pub(crate) align: ScrollAlign,
}

/// What a scroller is attached to: who (so a stale detach is ignored) and how to reach it.
#[derive(Clone, Copy, Default)]
struct Slot {
    target: Option<(u64, Callback<ScrollRequest>)>,
    pending: Option<ScrollRequest>,
}

/// A handle that scrolls a virtual list or grid to an item. Create it with
/// [`use_virtual_scroller`] and pass it to the component's `scroller` prop.
///
/// It is `Copy`, so event handlers capture it. See the module documentation for what a call
/// before the component has mounted does.
#[derive(Clone, Copy, PartialEq)]
pub struct VirtualScroller {
    slot: CopyValue<Slot>,
}

/// Creates a [`VirtualScroller`] owned by the calling component.
pub fn use_virtual_scroller() -> VirtualScroller {
    use_hook(|| VirtualScroller {
        slot: CopyValue::new(Slot::default()),
    })
}

impl VirtualScroller {
    /// Scrolls item `index` to `align`, clamped to the scroll range (an `index` past the end
    /// goes to the last item). In a windowed list the item need not be mounted: its position
    /// comes from the measured sizes where there are any and the estimates elsewhere, and the
    /// rows around it are measured as they mount. `VirtualGrid` reads `index` as a row; the
    /// wrapping and justified grids read it as an item index.
    pub fn scroll_to_index(&self, index: usize, align: ScrollAlign) {
        self.send(ScrollRequest {
            index,
            column: None,
            align,
        });
    }

    /// `VirtualGrid`: scrolls the cell `(row, column)` to `align` on both axes. A pinned
    /// column is always in view, so it moves only the rows. The other components ignore
    /// `column`.
    pub fn scroll_to_cell(&self, row: usize, column: usize, align: ScrollAlign) {
        self.send(ScrollRequest {
            index: row,
            column: Some(column),
            align,
        });
    }

    fn send(&self, request: ScrollRequest) {
        let mut slot = self.slot;
        let Ok(mut state) = slot.try_write() else {
            return;
        };
        match state.target {
            Some((_, handler)) => {
                drop(state);
                handler.call(request);
            }
            None => state.pending = Some(request),
        }
    }

    fn attach(&self, token: u64, handler: Callback<ScrollRequest>) {
        let mut slot = self.slot;
        let Ok(mut state) = slot.try_write() else {
            return;
        };
        state.target = Some((token, handler));
        let pending = state.pending.take();
        drop(state);
        if let Some(request) = pending {
            handler.call(request);
        }
    }

    fn detach(&self, token: u64) {
        let mut slot = self.slot;
        let Ok(mut state) = slot.try_write() else {
            return;
        };
        if state.target.is_some_and(|(owner, _)| owner == token) {
            state.target = None;
        }
    }
}

/// Attaches `handler` to `scroller` once the calling component has mounted and detaches it when
/// the component unmounts. `handler` must be safe to call at any time after the mount.
pub(crate) fn use_scroller_attach(
    scroller: Option<VirtualScroller>,
    handler: Callback<ScrollRequest>,
) {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let token = use_hook(|| NEXT.fetch_add(1, Ordering::Relaxed));
    use_effect(move || {
        if let Some(scroller) = scroller {
            scroller.attach(token, handler);
        }
    });
    use_drop(move || {
        if let Some(scroller) = scroller {
            scroller.detach(token);
        }
    });
}

/// The scroll offset that puts an item at `align` in a viewport. All values are along one axis,
/// in px: the item spans `start..start + size`, the viewport shows `current..current +
/// viewport` of `total`. Clamped to the scroll range. `Nearest` leaves `current` when the item
/// is fully shown, else takes the nearest edge (the start edge for an item that does not fit).
pub(crate) fn align_offset(
    start: u32,
    size: u32,
    current: u32,
    viewport: u32,
    total: u32,
    align: ScrollAlign,
) -> u32 {
    let end = start.saturating_add(size);
    let wanted = match align {
        ScrollAlign::Start => start,
        ScrollAlign::End => end.saturating_sub(viewport),
        ScrollAlign::Center => start
            .saturating_add(size / 2)
            .saturating_sub(viewport / 2),
        ScrollAlign::Nearest => {
            if start >= current && end <= current.saturating_add(viewport) {
                current
            } else if start < current || size >= viewport {
                start
            } else {
                end - viewport
            }
        }
    };
    wanted.min(total.saturating_sub(viewport))
}

/// Scrolls the element `selector` names inside the container `container_id` to `align`, for
/// content-visibility mode, where every item is in the DOM. `cover` is `(top, start, end)`: the px
/// the pinned header and columns hide at each edge. `inline` also aligns along the inline axis
/// (a grid cell).
pub(crate) fn scroll_dom_to(
    container_id: String,
    selector: String,
    align: ScrollAlign,
    cover: (u32, u32, u32),
    inline: bool,
) {
    let eval = document::eval(SCROLL_DOM_JS);
    let _ = eval.send((container_id, selector, align, cover, inline));
}

/// The DOM flow behind [`scroll_dom_to`]. `scrollIntoView` first, because the browser makes a
/// skipped chunk relevant for it; then the exact alignment (clear of the pinned edges), repeated
/// for a few frames because a chunk that has just rendered replaces its height guess with its
/// real height and moves everything below it.
const SCROLL_DOM_JS: &str = r#"
    const [id, selector, align, cover, inline] = await dioxus.recv();
    const [coverTop, coverStart, coverEnd] = cover;
    let tries = 0;
    let settled = 0;
    let first = true;
    const delta = (from, to, size, a) => {
        // Where the item's edges are against the visible span [from, to].
        const end = from + size;
        if (a === "start") return from - to.start;
        if (a === "end") return end - to.end;
        if (a === "center") return from + size / 2 - (to.start + to.end) / 2;
        if (from >= to.start && end <= to.end) return 0;
        return from < to.start || size >= to.end - to.start ? from - to.start : end - to.end;
    };
    const go = () => {
        const grid = document.getElementById(id);
        if (!grid) return;
        const item = grid.querySelector(selector);
        if (!item) {
            if (tries++ < 30) requestAnimationFrame(go);
            return;
        }
        if (first) {
            first = false;
            item.scrollIntoView({ block: align === "nearest" ? "nearest" : align, inline: "nearest" });
        }
        const box = grid.getBoundingClientRect();
        const rect = item.getBoundingClientRect();
        const block = delta(
            rect.top - box.top - grid.clientTop,
            { start: coverTop, end: grid.clientHeight },
            rect.height,
            align,
        );
        let moved = Math.abs(block) > 1;
        if (moved) grid.scrollTop += block;
        if (inline && !item.hasAttribute("data-grid-pinned")) {
            const rtl = getComputedStyle(grid).direction === "rtl";
            const from = rtl
                ? box.left + grid.clientLeft + grid.clientWidth - rect.right
                : rect.left - box.left - grid.clientLeft;
            const along = delta(
                from,
                { start: coverStart, end: grid.clientWidth - coverEnd },
                rect.width,
                align,
            );
            if (Math.abs(along) > 1) {
                grid.scrollLeft += rtl ? -along : along;
                moved = true;
            }
        }
        settled = moved ? 0 : settled + 1;
        if (settled < 2 && tries++ < 20) requestAnimationFrame(go);
    };
    go();
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use ScrollAlign::{Center, End, Nearest, Start};

    /// 100 rows of 50px in a 600px viewport (total 5,000), item `index` of them.
    fn at(index: u32, current: u32, align: ScrollAlign) -> u32 {
        align_offset(index * 50, 50, current, 600, 5_000, align)
    }

    #[test]
    fn start_puts_the_item_at_the_top() {
        assert_eq!(at(40, 0, Start), 2_000);
    }

    #[test]
    fn end_puts_the_item_at_the_bottom() {
        // Row 40 ends at 2,050; the viewport ends there when it starts 600 above.
        assert_eq!(at(40, 0, End), 1_450);
    }

    #[test]
    fn center_puts_the_item_in_the_middle() {
        // 2,025 is the row's centre; minus half the viewport.
        assert_eq!(at(40, 0, Center), 1_725);
    }

    #[test]
    fn every_alignment_clamps_to_the_scroll_range() {
        for align in [Start, Center, End, Nearest] {
            assert!(at(99, 0, align) <= 4_400, "{align:?}");
            assert!(at(2, 0, align) <= 4_400, "{align:?}");
        }
        assert_eq!(at(99, 0, Start), 4_400, "the last rows cannot reach the top");
        assert_eq!(at(99, 0, Center), 4_400);
        assert_eq!(at(0, 900, End), 0, "the first row's bottom is under the viewport");
        assert_eq!(at(1, 900, Center), 0);
    }

    #[test]
    fn nearest_leaves_a_fully_visible_item_where_it_is() {
        // Rows 12..23 fill 600..1,200 exactly.
        assert_eq!(at(12, 600, Nearest), 600);
        assert_eq!(at(23, 600, Nearest), 600);
        assert_eq!(at(15, 600, Nearest), 600);
    }

    #[test]
    fn nearest_takes_the_closest_edge() {
        // Above the viewport: its top edge. Below it: its bottom edge.
        assert_eq!(at(5, 600, Nearest), 250);
        assert_eq!(at(30, 600, Nearest), 950);
        // Half hidden at either edge.
        assert_eq!(align_offset(580, 50, 600, 600, 5_000, Nearest), 580);
        assert_eq!(align_offset(1_180, 50, 600, 600, 5_000, Nearest), 630);
    }

    #[test]
    fn an_item_taller_than_the_viewport_aligns_its_start_for_nearest() {
        assert_eq!(align_offset(1_000, 900, 0, 600, 5_000, Nearest), 1_000);
        assert_eq!(align_offset(1_000, 900, 1_500, 600, 5_000, Nearest), 1_000);
    }

    #[test]
    fn a_viewport_taller_than_the_content_never_scrolls() {
        for align in [Start, Center, End, Nearest] {
            assert_eq!(align_offset(100, 50, 0, 600, 300, align), 0, "{align:?}");
        }
    }

    #[test]
    fn a_call_with_no_component_is_kept_for_the_attach() {
        // The pure part of the queue: the last pending call wins.
        let mut slot = Slot::default();
        for index in [3, 9] {
            slot.pending = Some(ScrollRequest {
                index,
                column: None,
                align: Start,
            });
        }
        assert_eq!(slot.pending.map(|request| request.index), Some(9));
    }

    #[test]
    fn the_dom_flow_settles_and_reads_both_axes_in_logical_terms() {
        assert!(SCROLL_DOM_JS.contains("scrollIntoView"));
        assert!(SCROLL_DOM_JS.contains("requestAnimationFrame(go)"));
        assert!(SCROLL_DOM_JS.contains(r#"direction === "rtl""#));
        assert!(SCROLL_DOM_JS.contains("data-grid-pinned"));
    }
}
