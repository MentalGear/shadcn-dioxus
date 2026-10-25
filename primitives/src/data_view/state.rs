//! [`ViewState`]: what a collection view remembers, and the only ways it
//! changes. Pure data with pure transitions, so every rule is a plain unit
//! test.

use super::stages::SortDirection;
use std::collections::{HashMap, HashSet};

/// A manual order of row keys (drag-to-reorder), applied when no sort is
/// active. It may list keys that no longer exist and miss keys that are new:
/// [`KeyOrder::reconcile`] always yields a duplicate-free permutation of the
/// keys that exist, so the view never depends on the caller keeping it in step.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyOrder(Vec<String>);

impl KeyOrder {
    /// An order that lists `keys` first to last.
    pub fn new(keys: impl IntoIterator<Item = String>) -> Self {
        Self(keys.into_iter().collect())
    }

    /// The listed keys, first to last (may be stale; see [`Self::reconcile`]).
    pub fn keys(&self) -> &[String] {
        &self.0
    }

    /// `current` (in source order) arranged by this order: listed keys in the
    /// listed order, missing ones dropped, new ones appended in source order.
    /// Idempotent: `reconcile` of a reconciled order's keys is the same list.
    pub fn reconcile<'a>(&self, current: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        let mut keys: Vec<&'a str> = current.into_iter().collect();
        let position: HashMap<&str, usize> = self
            .0
            .iter()
            .enumerate()
            .rev()
            .map(|(i, k)| (k.as_str(), i))
            .collect();
        // Stable: keys with no listed position keep their source order, after the listed ones.
        keys.sort_by_key(|k| position.get(k).copied().unwrap_or(usize::MAX));
        keys
    }

    /// `items` arranged by this order, where `key` names each item's key: the
    /// same rule as [`Self::reconcile`] (listed keys in the listed order,
    /// unlisted items after them in source order, stable for equal keys) but
    /// over the items themselves, so a duplicate key cannot lose one.
    pub fn arrange<T>(
        &self,
        items: impl IntoIterator<Item = T>,
        key: impl Fn(&T) -> &str,
    ) -> Vec<T> {
        let position: HashMap<&str, usize> = self
            .0
            .iter()
            .enumerate()
            .rev()
            .map(|(i, k)| (k.as_str(), i))
            .collect();
        let mut items: Vec<T> = items.into_iter().collect();
        items.sort_by_key(|item| position.get(key(item)).copied().unwrap_or(usize::MAX));
        items
    }

    /// How many keys the order lists.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the order lists no key.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The listed position of `key`.
    pub fn position(&self, key: &str) -> Option<usize> {
        self.0.iter().position(|k| k == key)
    }

    /// Moves `key` so it sits at listed position `to` (clamped to the last
    /// position) and the keys between shift by one. Returns `false`, changing
    /// nothing, when `key` is not listed. The inverse of a move is
    /// `move_key(key, old_position)`.
    pub fn move_key(&mut self, key: &str, to: usize) -> bool {
        let Some(from) = self.position(key) else {
            return false;
        };
        let to = to.min(self.0.len() - 1);
        let item = self.0.remove(from);
        self.0.insert(to, item);
        true
    }

    /// Moves `key` by `delta` positions (negative = earlier), stopping at the
    /// ends rather than wrapping. `false` when `key` is not listed.
    pub fn move_by(&mut self, key: &str, delta: isize) -> bool {
        let Some(from) = self.position(key) else {
            return false;
        };
        let to = from.saturating_add_signed(delta);
        self.move_key(key, to)
    }

    /// Moves `key` to sit directly before `anchor` (the row-order move once a
    /// filter hides rows: the anchor is the next *visible* key, so hidden keys
    /// keep their relative place). `false`, changing nothing, when either key
    /// is not listed or they are the same key.
    pub fn move_before(&mut self, key: &str, anchor: &str) -> bool {
        if key == anchor {
            return false;
        }
        let (Some(from), Some(_)) = (self.position(key), self.position(anchor)) else {
            return false;
        };
        let item = self.0.remove(from);
        let at = self.position(anchor).unwrap_or(self.0.len());
        self.0.insert(at, item);
        true
    }

    /// Moves `key` to sit directly after `anchor`, the mirror of
    /// [`Self::move_before`] (the row-order move when the moved row becomes
    /// the last visible one). `false`, changing nothing, when either key is not
    /// listed or they are the same key.
    pub fn move_after(&mut self, key: &str, anchor: &str) -> bool {
        if key == anchor {
            return false;
        }
        let (Some(from), Some(_)) = (self.position(key), self.position(anchor)) else {
            return false;
        };
        let item = self.0.remove(from);
        let at = self.position(anchor).map_or(self.0.len(), |p| p + 1);
        self.0.insert(at, item);
        true
    }

    /// Drops `key` from the order. `true` when it was listed.
    pub fn remove(&mut self, key: &str) -> bool {
        match self.position(key) {
            Some(at) => {
                self.0.remove(at);
                true
            }
            None => false,
        }
    }

    /// The listed keys, owned.
    pub fn into_keys(self) -> Vec<String> {
        self.0
    }
}

/// Where a moved row lands: directly before or directly after a neighbour that
/// is visible (on the page, through the filter), so rows the view hides keep
/// their relative place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowAnchor<'a> {
    /// Directly before this row.
    Before(&'a str),
    /// Directly after this row.
    After(&'a str),
}

/// What a collection view remembers between renders.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewState {
    /// The sorted field's id and direction, or `None` for source order (or the
    /// manual [`order`](Self::order) when there is one).
    pub sort: Option<(String, SortDirection)>,
    /// The filter text, as typed.
    pub filter: String,
    /// 0-based current page (may exceed the last page after the rows shrink;
    /// [`derive`](super::derive()) clamps it for display).
    pub page: usize,
    /// Rows per page.
    pub page_size: usize,
    /// Keys of the selected rows, across every page and filter.
    pub selected: HashSet<String>,
    /// A manual row order, used while no sort is active ([`move_row`](Self::move_row); the Data Table's drag handle column writes it).
    pub order: Option<KeyOrder>,
}

/// Which reported parts of a [`ViewState`] a transition changed
/// ([`ViewState::changes_since`]): what a view diffs once around each
/// transition to fire its `on_sort_change`/`on_page_change`/`on_selection_change`
/// callbacks, so every mutator reports without one of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ViewChanges {
    /// The sorted field or its direction changed (including to or from no sort).
    pub sort: bool,
    /// The 0-based page changed.
    pub page: bool,
    /// The set of selected keys changed.
    pub selection: bool,
}

/// The Data Table's name for [`ViewState`] (backlog row 340).
pub type TableState = ViewState;

impl ViewState {
    /// A fresh state on page 0 with `page_size` rows per page.
    pub fn new(page_size: usize, sort: Option<(String, SortDirection)>) -> Self {
        Self {
            sort,
            filter: String::new(),
            page: 0,
            page_size: page_size.max(1),
            selected: HashSet::new(),
            order: None,
        }
    }

    /// A header activation, TanStack's default cycle: another column becomes the
    /// sort, ascending; the sorted column goes ascending -> descending -> no sort
    /// (source order, or the manual [`order`](Self::order)). Returns to page 0.
    pub fn toggle_sort(&mut self, column_id: &str) {
        self.sort = match self.sort.take() {
            Some((id, SortDirection::Ascending)) if id == column_id => {
                Some((id, SortDirection::Descending))
            }
            Some((id, SortDirection::Descending)) if id == column_id => None,
            _ => Some((column_id.to_string(), SortDirection::Ascending)),
        };
        self.page = 0;
    }

    /// Clears the sort (source order, or the manual [`order`](Self::order)) and
    /// returns to page 0.
    pub fn clear_sort(&mut self) {
        self.sort = None;
        self.page = 0;
    }

    /// What changed from `before` to `self` among the parts a view reports to
    /// its owner (sort, page, selection); see [`ViewChanges`].
    pub fn changes_since(&self, before: &ViewState) -> ViewChanges {
        ViewChanges {
            sort: self.sort != before.sort,
            page: self.page != before.page,
            selection: self.selected != before.selected,
        }
    }

    /// Sets the filter text and returns to page 0.
    pub fn set_filter(&mut self, text: impl Into<String>) {
        self.filter = text.into();
        self.page = 0;
    }

    /// Sets rows per page (floored at 1) and returns to page 0.
    pub fn set_page_size(&mut self, page_size: usize) {
        self.page_size = page_size.max(1);
        self.page = 0;
    }

    /// Goes to `page`, clamped to `0..page_count`.
    pub fn set_page(&mut self, page: usize, page_count: usize) {
        self.page = page.min(page_count.saturating_sub(1));
    }

    /// One page back; stays on page 0.
    pub fn previous_page(&mut self) {
        self.page = self.page.saturating_sub(1);
    }

    /// One page forward; stays on the last of `page_count` pages.
    pub fn next_page(&mut self, page_count: usize) {
        self.set_page(self.page + 1, page_count);
    }

    /// Selects or deselects one row.
    pub fn set_selected(&mut self, key: &str, selected: bool) {
        if selected {
            self.selected.insert(key.to_string());
        } else {
            self.selected.remove(key);
        }
    }

    /// The manual row move (a drag handle or its keyboard twin): `key` lands
    /// next to `anchor` in the order of `all_keys`, every key of the source rows
    /// (the stored order is first reconciled against them, so it is always a
    /// permutation of the keys that exist). The page,
    /// filter and selection are untouched. `false`, changing nothing, when
    /// `key` or the anchor is not among `all_keys` or they are the same row.
    pub fn move_row<'a>(
        &mut self,
        all_keys: impl IntoIterator<Item = &'a str>,
        key: &str,
        anchor: RowAnchor<'_>,
    ) -> bool {
        let stored = self.order.clone().unwrap_or_default();
        let mut order = KeyOrder::new(stored.reconcile(all_keys).into_iter().map(str::to_string));
        let moved = match anchor {
            RowAnchor::Before(anchor) => order.move_before(key, anchor),
            RowAnchor::After(anchor) => order.move_after(key, anchor),
        };
        if moved {
            self.order = Some(order);
        }
        moved
    }

    /// Selects or deselects every key in `keys` (the header checkbox: the rows
    /// of the current page).
    pub fn set_many_selected<'a>(
        &mut self,
        keys: impl IntoIterator<Item = &'a str>,
        selected: bool,
    ) {
        for key in keys {
            self.set_selected(key, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(keys: &[&str]) -> KeyOrder {
        KeyOrder::new(keys.iter().map(|k| k.to_string()))
    }

    fn listed(order: &KeyOrder) -> Vec<&str> {
        order.keys().iter().map(String::as_str).collect()
    }

    #[test]
    fn move_key_shifts_the_keys_between() {
        let mut o = order(&["a", "b", "c", "d"]);
        assert!(o.move_key("a", 2));
        assert_eq!(listed(&o), ["b", "c", "a", "d"]);
        assert!(o.move_key("d", 0));
        assert_eq!(listed(&o), ["d", "b", "c", "a"]);
        // Past the end clamps to the last position.
        assert!(o.move_key("d", 99));
        assert_eq!(listed(&o), ["b", "c", "a", "d"]);
        // A missing key changes nothing.
        assert!(!o.move_key("zz", 0));
        assert_eq!(listed(&o), ["b", "c", "a", "d"]);
    }

    #[test]
    fn move_by_stops_at_the_ends() {
        let mut o = order(&["a", "b", "c"]);
        assert!(o.move_by("a", -1));
        assert_eq!(listed(&o), ["a", "b", "c"]);
        assert!(o.move_by("a", 1));
        assert_eq!(listed(&o), ["b", "a", "c"]);
        assert!(o.move_by("a", 9));
        assert_eq!(listed(&o), ["b", "c", "a"]);
    }

    #[test]
    fn move_before_lands_ahead_of_the_anchor_whatever_the_direction() {
        let mut o = order(&["a", "b", "c", "d"]);
        assert!(o.move_before("a", "d"));
        assert_eq!(listed(&o), ["b", "c", "a", "d"]);
        assert!(o.move_before("d", "b"));
        assert_eq!(listed(&o), ["d", "b", "c", "a"]);
        assert!(!o.move_before("d", "d"));
        assert!(!o.move_before("d", "zz"));
        assert_eq!(listed(&o), ["d", "b", "c", "a"]);
    }

    #[test]
    fn arrange_follows_reconcile_and_keeps_duplicates() {
        let o = order(&["c", "a", "gone"]);
        let items = ["a", "b", "c", "a"];
        assert_eq!(o.arrange(items, |k| k), ["c", "a", "a", "b"]);
        assert_eq!(o.reconcile(items), ["c", "a", "a", "b"]);
    }

    #[test]
    fn move_after_lands_behind_the_anchor_whatever_the_direction() {
        let mut o = order(&["a", "b", "c", "d"]);
        assert!(o.move_after("a", "c"));
        assert_eq!(listed(&o), ["b", "c", "a", "d"]);
        assert!(o.move_after("d", "b"));
        assert_eq!(listed(&o), ["b", "d", "c", "a"]);
        assert!(!o.move_after("d", "d"));
        assert!(!o.move_after("d", "zz"));
        assert_eq!(listed(&o), ["b", "d", "c", "a"]);
    }

    #[test]
    fn move_row_reconciles_first_and_leaves_the_rest_of_the_view_alone() {
        let mut state = ViewState::new(2, None);
        state.page = 1;
        state.set_selected("c", true);
        let all = ["a", "b", "c", "d"];
        // No stored order yet: source order is the starting point.
        assert!(state.move_row(all, "d", RowAnchor::Before("a")));
        assert_eq!(listed(state.order.as_ref().unwrap()), ["d", "a", "b", "c"]);
        // A key that is new since, and one that is gone, are reconciled away.
        let all = ["a", "b", "c", "e"];
        assert!(state.move_row(all, "a", RowAnchor::After("c")));
        assert_eq!(listed(state.order.as_ref().unwrap()), ["b", "c", "a", "e"]);
        // A move that cannot happen changes nothing.
        let before = state.clone();
        assert!(!state.move_row(all, "zz", RowAnchor::After("c")));
        assert!(!state.move_row(all, "a", RowAnchor::Before("a")));
        assert_eq!(state, before);
        assert_eq!(state.page, 1);
        assert!(state.selected.contains("c"));
    }

    #[test]
    fn toggle_sort_cycles_ascending_descending_none_and_clear_sort_clears() {
        let mut state = ViewState::new(2, None);
        state.page = 1;
        state.toggle_sort("n");
        assert_eq!(state.sort, Some(("n".into(), SortDirection::Ascending)));
        assert_eq!(state.page, 0);
        state.toggle_sort("n");
        assert_eq!(state.sort, Some(("n".into(), SortDirection::Descending)));
        state.page = 1;
        state.toggle_sort("n");
        assert_eq!(state.sort, None);
        assert_eq!(state.page, 0);
        // A descending sort on another column restarts at ascending on the new one.
        state.toggle_sort("a");
        state.toggle_sort("a");
        state.toggle_sort("n");
        assert_eq!(state.sort, Some(("n".into(), SortDirection::Ascending)));
        state.page = 1;
        state.clear_sort();
        assert_eq!((state.sort.clone(), state.page), (None, 0));
    }

    #[test]
    fn changes_since_names_each_reported_part() {
        let before = ViewState::new(2, None);
        let mut after = before.clone();
        assert_eq!(after.changes_since(&before), ViewChanges::default());
        after.set_filter("x");
        assert_eq!(after.changes_since(&before), ViewChanges::default());
        after.page = 3;
        after.toggle_sort("n");
        let changed = after.changes_since(&before);
        assert!(changed.sort && !changed.page && !changed.selection);
        after.set_page(2, 5);
        after.set_selected("a", true);
        assert_eq!(
            after.changes_since(&before),
            ViewChanges {
                sort: true,
                page: true,
                selection: true
            }
        );
        // Selecting a selected key again changes nothing.
        let settled = after.clone();
        after.set_selected("a", true);
        assert_eq!(after.changes_since(&settled), ViewChanges::default());
    }

    #[test]
    fn remove_drops_one_key() {
        let mut o = order(&["a", "b", "c"]);
        assert!(o.remove("b"));
        assert!(!o.remove("b"));
        assert_eq!(listed(&o), ["a", "c"]);
        assert_eq!(o.len(), 2);
    }
}
