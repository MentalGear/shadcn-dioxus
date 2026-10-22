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
    /// [`derive`](super::derive) clamps it for display).
    pub page: usize,
    /// Rows per page.
    pub page_size: usize,
    /// Keys of the selected rows, across every page and filter.
    pub selected: HashSet<String>,
    /// A manual row order, used while no sort is active. Not drawn by any UI yet.
    pub order: Option<KeyOrder>,
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

    /// A header activation: the sorted column flips direction, any other column
    /// becomes the sort, ascending. Returns to page 0.
    pub fn toggle_sort(&mut self, column_id: &str) {
        self.sort = match self.sort.take() {
            Some((id, direction)) if id == column_id => Some((id, direction.toggled())),
            _ => Some((column_id.to_string(), SortDirection::Ascending)),
        };
        self.page = 0;
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
