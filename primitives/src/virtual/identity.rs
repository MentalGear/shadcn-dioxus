//! Item identity for the windowed renderers (backlog row 361).
//!
//! A renderer is `count + render(index)`; on its own nothing says *who* is at an index, so a
//! sort or a reorder leaves DOM nodes, the pinned focus row, the grid's active cell and the
//! measured sizes on the slot while the item moves. The optional `item_key` prop (`Callback<usize,
//! String>`, default: the index) says who, and everything that must survive a change of the view
//! is keyed through this module:
//!
//! - [`render_key`]: the one place a rendered element's `key` is made;
//! - [`key_hash`]: what the engine's measured sizes are stored under (a `u64` of the key, the
//!   index itself without an `item_key`, so a default caller sees no change);
//! - [`resolve_index`]: where a remembered item is now (the pinned row, the active cell).

use std::hash::{Hash, Hasher};

use dioxus::prelude::Callback;

/// The `key` of the element that renders item `idx`: the caller's `item_key` when given, else the
/// index. Every rendered key of a windowed renderer goes through here (`check-render-keys.sh`).
pub(crate) fn render_key(item_key: Option<Callback<usize, String>>, idx: usize) -> String {
    match item_key {
        Some(key) => key(idx),
        None => idx.to_string(),
    }
}

/// The engine's key of an item key: a stable (fixed-seed) hash.
pub(crate) fn key_hash(key: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// The engine keys of items `0..count`, in index order. `None` without an `item_key`: the engine
/// then keys by index.
pub(crate) fn engine_keys(
    item_key: Option<Callback<usize, String>>,
    count: usize,
) -> Option<std::rc::Rc<[u64]>> {
    item_key.map(|f| (0..count).map(|i| key_hash(&f(i))).collect())
}

/// Where the item remembered as `key` is now, given the index it last had (`hint`). `O(1)` while
/// the item has not moved, one scan over `0..count` after a reorder (then the caller stores the
/// result as the new hint). Without an `item_key` an item is its index. `None` when the item is
/// gone.
pub(crate) fn resolve_index(
    item_key: Option<Callback<usize, String>>,
    key: Option<&str>,
    hint: usize,
    count: usize,
) -> Option<usize> {
    let (Some(item_key), Some(key)) = (item_key, key) else {
        return Some(hint);
    };
    if hint < count && item_key(hint) == key {
        return Some(hint);
    }
    (0..count).find(|&i| item_key(i) == key)
}

/// The engine keys of the rows of a wrapping grid: row `r` holds items `r * columns..`, and a
/// row's measured height is a property of what it holds, so a row is keyed by the (order-free)
/// combination of its items' key hashes. A reorder inside a row keeps the row's size, an item
/// moving to another row takes its height with the row it joins, and a change of `columns`
/// needs no separate "forget the measurements".
pub(crate) fn engine_row_keys(
    item_key: Option<Callback<usize, String>>,
    count: usize,
    columns: usize,
) -> Option<std::rc::Rc<[u64]>> {
    let item_key = item_key?;
    let columns = columns.max(1);
    Some(
        (0..count.div_ceil(columns))
            .map(|row| {
                (row * columns..((row + 1) * columns).min(count))
                    .fold(0u64, |acc, i| acc.wrapping_add(key_hash(&item_key(i))))
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::with_runtime;
    use dioxus::prelude::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// The key-OFF half of `playwright/item-key.spec.ts` ("without item_key a row is its slot"),
    /// proved at the engine: with no `item_key` the identity of an element IS its index, so after
    /// any reorder the DOM node, its state and its measured size stay on the slot.
    #[test]
    fn without_an_item_key_a_row_is_its_slot() {
        for idx in [0, 3, 196, 199] {
            assert_eq!(render_key(None, idx), idx.to_string());
        }
        assert!(engine_keys(None, 200).is_none());
        assert!(engine_row_keys(None, 200, 4).is_none());
        // The remembered item is "the one at that slot", wherever the data went.
        assert_eq!(resolve_index(None, Some("task-3"), 3, 200), Some(3));
        assert_eq!(resolve_index(None, Some("task-3"), 196, 200), Some(196));
    }

    /// The key-ON half: the key is the item's, so after a reverse the item that was at slot 3 is at
    /// slot 196 and everything keyed by it (the DOM node, the pinned row) goes there.
    #[test]
    fn with_an_item_key_a_row_follows_its_item() {
        with_runtime(|| {
            let order = Rc::new(RefCell::new((0..200usize).collect::<Vec<_>>()));
            let key = {
                let order = order.clone();
                Callback::new(move |idx: usize| format!("task-{}", order.borrow()[idx]))
            };
            assert_eq!(render_key(Some(key), 3), "task-3");
            let before = engine_keys(Some(key), 200).unwrap();
            order.borrow_mut().reverse();
            assert_eq!(render_key(Some(key), 3), "task-196");
            assert_eq!(render_key(Some(key), 196), "task-3");
            assert_eq!(resolve_index(Some(key), Some("task-3"), 3, 200), Some(196));
            // An item's engine key never changes; only the slot it sits in does.
            let after = engine_keys(Some(key), 200).unwrap();
            assert_eq!(before[3], after[196]);
            assert_ne!(before[3], after[3]);
            // A key that is gone resolves to nothing.
            assert_eq!(resolve_index(Some(key), Some("task-999"), 3, 200), None);
        });
    }
}
