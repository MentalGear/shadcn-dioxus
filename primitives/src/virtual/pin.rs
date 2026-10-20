//! Keep the focused item rendered: the strategy piece both windowed components share.
//!
//! A windowed list or grid unmounts what scrolls out of its window, and an element that is
//! unmounted while it holds focus loses it (focus falls to `<body>`, and a grid loses its only
//! tab stop). So the item that holds focus is "pinned": drawn even when it is far outside the
//! window. [`with_pinned`] adds it to the window in index order, so keyed rendering keeps its DOM
//! node (moving it into the window when a scroll reaches it rather than re-creating it).
//!
//! The grids place every item absolutely (`top: start`), so a pinned item simply sits at its own
//! position. `VirtualList` stacks its window in normal flow inside a translated wrapper (so a row
//! whose estimate was wrong still sits right below its neighbour); a pinned row in that flow
//! would push its neighbours, so [`pin_in_flow`] also says where to draw it absolutely when it is
//! not contiguous with the window. Same parent, same key: when the window reaches it, it only
//! loses its `position: absolute` and joins the flow, without remounting.

use super::types::VirtualItem;

/// `items` with `pinned` added in index order when it is not already there. The window stays a
/// sorted, duplicate-free list, so keyed rendering moves the pinned item's DOM node into the
/// window (rather than re-creating it) when a scroll reaches it.
pub(crate) fn with_pinned(
    mut items: Vec<VirtualItem>,
    pinned: Option<VirtualItem>,
) -> Vec<VirtualItem> {
    if let Some(pinned) = pinned {
        if let Err(at) = items.binary_search_by_key(&pinned.index(), VirtualItem::index) {
            items.insert(at, pinned);
        }
    }
    items
}

/// A flow-layout window with a pinned item: the items to draw, and, when the pinned item is not
/// contiguous with the window (`window` is `first..=last` of consecutive indices), its index and
/// its offset in px from the top of the window's wrapper, which starts at `top_offset`. A
/// detached item is drawn with `position: absolute; top: <offset>px` so it never moves the rows
/// in flow; the offset is negative for an item above the window.
pub(crate) fn pin_in_flow(
    items: Vec<VirtualItem>,
    pinned: Option<VirtualItem>,
    top_offset: u32,
) -> (Vec<VirtualItem>, Option<(usize, i64)>) {
    let Some(pinned) = pinned else {
        return (items, None);
    };
    let contiguous = match (items.first(), items.last()) {
        (Some(first), Some(last)) => (first.index()..=last.index()).contains(&pinned.index()),
        _ => false,
    };
    let detached = (!contiguous).then(|| {
        (
            pinned.index(),
            i64::from(pinned.start()) - i64::from(top_offset),
        )
    });
    (with_pinned(items, Some(pinned)), detached)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(i: usize) -> VirtualItem {
        VirtualItem::new(i, i, i as u32 * 10, 10)
    }

    fn indices(items: &[VirtualItem]) -> Vec<usize> {
        items.iter().map(VirtualItem::index).collect()
    }

    #[test]
    fn with_pinned_keeps_the_list_sorted_and_unique() {
        let window: Vec<_> = (10..=12).map(item).collect();
        assert_eq!(
            indices(&with_pinned(window.clone(), None)),
            vec![10, 11, 12]
        );
        assert_eq!(
            indices(&with_pinned(window.clone(), Some(item(11)))),
            vec![10, 11, 12]
        );
        assert_eq!(
            indices(&with_pinned(window.clone(), Some(item(2)))),
            vec![2, 10, 11, 12]
        );
        assert_eq!(
            indices(&with_pinned(window, Some(item(90)))),
            vec![10, 11, 12, 90]
        );
    }

    #[test]
    fn a_pinned_item_inside_the_window_stays_in_flow() {
        let window: Vec<_> = (10..=12).map(item).collect();
        let (items, detached) = pin_in_flow(window, Some(item(11)), 100);
        assert_eq!(indices(&items), vec![10, 11, 12]);
        assert_eq!(detached, None);
    }

    #[test]
    fn a_pinned_item_outside_the_window_is_detached_at_its_own_position() {
        let window: Vec<_> = (10..=12).map(item).collect();
        // Below: row 90 starts at 900px, the wrapper at 100px.
        let (items, detached) = pin_in_flow(window.clone(), Some(item(90)), 100);
        assert_eq!(indices(&items), vec![10, 11, 12, 90]);
        assert_eq!(detached, Some((90, 800)));
        // Above: row 2 starts at 20px, 80px above the wrapper.
        let (items, detached) = pin_in_flow(window.clone(), Some(item(2)), 100);
        assert_eq!(indices(&items), vec![2, 10, 11, 12]);
        assert_eq!(detached, Some((2, -80)));
        // The neighbour just past the window is not contiguous either: it would sit in flow
        // only once the window grows to include it.
        let (_, detached) = pin_in_flow(window, Some(item(13)), 100);
        assert_eq!(detached, Some((13, 30)));
    }

    #[test]
    fn an_empty_window_detaches_the_pinned_item_and_no_pin_changes_nothing() {
        let (items, detached) = pin_in_flow(Vec::new(), Some(item(5)), 0);
        assert_eq!(indices(&items), vec![5]);
        assert_eq!(detached, Some((5, 50)));
        let window: Vec<_> = (0..3).map(item).collect();
        let (items, detached) = pin_in_flow(window, None, 0);
        assert_eq!((indices(&items), detached), (vec![0, 1, 2], None));
    }
}
