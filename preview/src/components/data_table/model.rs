//! The pure state machine and derivation behind `use_data_table` (backlog row
//! 340): no signals, no rendering, so every transition is a plain unit test.
//!
//! [`TableState`] is what a table remembers (sort, filter text, page, page
//! size, selected row keys) and its methods are the only way it changes, so the
//! rules "sorting, filtering or resizing the page returns to page 1" and "the
//! page is clamped to the last page" live here once. [`derive`] turns the full
//! row list plus a state into the [`TableView`] a table renders, built on the
//! helpers of `state.rs` (`filter_rows`, `sort_rows_by`, `paginate`,
//! `tri_state`). The module wiring is the same `#[path]` sibling as `state.rs`.

use super::state::{filter_rows, includes_string, paginate, sort_rows_by, tri_state, SortDirection};
use dioxus_primitives::checkbox::CheckboxState;
use std::cmp::Ordering;
use std::collections::HashSet;

/// An ascending comparator over rows.
pub type RowCompare<'a, T> = dyn Fn(&T, &T) -> Ordering + 'a;

/// What a data table remembers between renders.
#[derive(Debug, Clone, PartialEq)]
pub struct TableState {
    /// The sorted column's id and direction, or `None` for source order.
    pub sort: Option<(String, SortDirection)>,
    /// The filter text, as typed.
    pub filter: String,
    /// 0-based current page (may exceed the last page after the rows shrink;
    /// [`derive`] clamps it for display).
    pub page: usize,
    /// Rows per page.
    pub page_size: usize,
    /// Keys of the selected rows, across every page and filter.
    pub selected: HashSet<String>,
}

impl TableState {
    /// A fresh state on page 0 with `page_size` rows per page.
    pub fn new(page_size: usize, sort: Option<(String, SortDirection)>) -> Self {
        Self { sort, filter: String::new(), page: 0, page_size: page_size.max(1), selected: HashSet::new() }
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
    pub fn set_many_selected<'a>(&mut self, keys: impl IntoIterator<Item = &'a str>, selected: bool) {
        for key in keys {
            self.set_selected(key, selected);
        }
    }
}

/// What a table renders for one [`TableState`].
#[derive(Debug, Clone, PartialEq)]
pub struct TableView<T> {
    /// The rows of the current page, sorted and filtered.
    pub rows: Vec<T>,
    /// The key of each of `rows`, in order.
    pub keys: Vec<String>,
    /// Rows left after filtering, before pagination.
    pub filtered_count: usize,
    /// Number of pages (at least 1).
    pub page_count: usize,
    /// The current page, clamped to the last page.
    pub page: usize,
    /// Selected rows across every page.
    pub selected_count: usize,
    /// The header checkbox: all, some or none of the current page's rows.
    pub select_all: CheckboxState,
}

/// Filters, sorts and paginates `rows` for `state`.
///
/// `filter_text` lists the texts the filter may match for a row (a row matches
/// when any contains the query, [`includes_string`] semantics; no texts means
/// no column is filterable, so every row matches). `compare` is the active
/// sort column's ascending comparator, if any. `key` is the stable row key.
pub fn derive<T: Clone>(
    rows: &[T],
    state: &TableState,
    filter_text: impl Fn(&T) -> Vec<String>,
    compare: Option<&RowCompare<T>>,
    key: impl Fn(&T) -> String,
) -> TableView<T> {
    let filtered = filter_rows(rows, |row| {
        let texts = filter_text(row);
        texts.is_empty() || texts.iter().any(|text| includes_string(text, &state.filter))
    });
    let sorted = match (&state.sort, compare) {
        (Some((_, direction)), Some(compare)) => sort_rows_by(&filtered, *direction, compare),
        _ => filtered.clone(),
    };
    let (page_rows, page_count) = paginate(&sorted, state.page, state.page_size);
    let keys: Vec<String> = page_rows.iter().map(&key).collect();
    let selected_here = keys.iter().filter(|k| state.selected.contains(*k)).count();
    TableView {
        filtered_count: filtered.len(),
        page_count,
        page: state.page.min(page_count - 1),
        selected_count: state.selected.len(),
        select_all: tri_state(selected_here, keys.len()),
        rows: page_rows,
        keys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Row {
        id: &'static str,
        name: &'static str,
        n: i32,
    }

    const ROWS: &[Row] = &[
        Row { id: "a", name: "carol", n: 3 },
        Row { id: "b", name: "alice", n: 1 },
        Row { id: "c", name: "bob", n: 2 },
        Row { id: "d", name: "dana", n: 4 },
        Row { id: "e", name: "erin", n: 5 },
    ];

    fn view(state: &TableState) -> TableView<Row> {
        let by_n = |a: &Row, b: &Row| a.n.cmp(&b.n);
        derive(ROWS, state, |r| vec![r.name.to_string()], Some(&by_n), |r| r.id.to_string())
    }

    fn ids(v: &TableView<Row>) -> Vec<&str> {
        v.rows.iter().map(|r| r.id).collect()
    }

    fn state() -> TableState {
        TableState::new(2, None)
    }

    #[test]
    fn unsorted_view_keeps_source_order_and_paginates() {
        let v = view(&state());
        assert_eq!(ids(&v), vec!["a", "b"]);
        assert_eq!((v.page_count, v.filtered_count, v.page), (3, 5, 0));
    }

    #[test]
    fn toggle_sort_new_column_is_ascending_same_column_flips() {
        let mut s = state();
        s.toggle_sort("n");
        assert_eq!(s.sort, Some(("n".into(), SortDirection::Ascending)));
        assert_eq!(ids(&view(&s)), vec!["b", "c"]);
        s.toggle_sort("n");
        assert_eq!(s.sort, Some(("n".into(), SortDirection::Descending)));
        assert_eq!(ids(&view(&s)), vec!["e", "d"]);
        s.toggle_sort("name");
        assert_eq!(s.sort, Some(("name".into(), SortDirection::Ascending)));
    }

    #[test]
    fn sorting_filtering_and_page_size_return_to_page_zero() {
        let mut s = state();
        s.set_page(2, 3);
        s.toggle_sort("n");
        assert_eq!(s.page, 0);
        s.set_page(2, 3);
        s.set_filter("a");
        assert_eq!(s.page, 0);
        s.set_page(1, 3);
        s.set_page_size(4);
        assert_eq!(s.page, 0);
        assert_eq!(s.page_size, 4);
    }

    #[test]
    fn page_size_is_floored_at_one() {
        let mut s = state();
        s.set_page_size(0);
        assert_eq!(s.page_size, 1);
        assert_eq!(TableState::new(0, None).page_size, 1);
    }

    #[test]
    fn filter_is_a_case_insensitive_substring_match_and_trims() {
        let mut s = state();
        s.set_filter("  LI ");
        let v = view(&s);
        assert_eq!(ids(&v), vec!["b"]);
        assert_eq!(v.filtered_count, 1);
        assert_eq!(v.page_count, 1);
        s.set_filter("");
        assert_eq!(view(&s).filtered_count, 5);
        s.set_filter("zzz");
        let v = view(&s);
        assert!(v.rows.is_empty());
        assert_eq!(v.page_count, 1);
    }

    #[test]
    fn no_filterable_column_means_the_filter_matches_everything() {
        let mut s = state();
        s.set_filter("zzz");
        let v = derive(ROWS, &s, |_| Vec::new(), None, |r| r.id.to_string());
        assert_eq!(v.filtered_count, 5);
    }

    #[test]
    fn next_and_previous_stop_at_the_ends() {
        let mut s = state();
        s.previous_page();
        assert_eq!(s.page, 0);
        s.next_page(3);
        s.next_page(3);
        s.next_page(3);
        assert_eq!(s.page, 2);
        assert_eq!(ids(&view(&s)), vec!["e"]);
        s.previous_page();
        assert_eq!(s.page, 1);
    }

    #[test]
    fn a_page_past_the_end_is_clamped_in_the_view() {
        let mut s = state();
        s.page = 9;
        let v = view(&s);
        assert_eq!(v.page, 2);
        assert_eq!(ids(&v), vec!["e"]);
    }

    #[test]
    fn selection_survives_sort_filter_and_page_changes() {
        let mut s = state();
        s.set_selected("d", true);
        s.toggle_sort("n");
        s.set_filter("a");
        s.next_page(2);
        s.set_page_size(5);
        assert_eq!(view(&s).selected_count, 1);
        assert!(s.selected.contains("d"));
        s.set_selected("d", false);
        assert_eq!(view(&s).selected_count, 0);
    }

    #[test]
    fn header_checkbox_covers_the_current_page_only() {
        let mut s = state();
        assert_eq!(view(&s).select_all, CheckboxState::Unchecked);
        s.set_selected("a", true);
        assert_eq!(view(&s).select_all, CheckboxState::Indeterminate);
        let keys = view(&s).keys;
        s.set_many_selected(keys.iter().map(String::as_str), true);
        let v = view(&s);
        assert_eq!(v.select_all, CheckboxState::Checked);
        assert_eq!(v.selected_count, 2);
        // Page 2 is untouched: unchecked there, count still spans pages.
        s.next_page(3);
        let v = view(&s);
        assert_eq!((v.select_all, v.selected_count), (CheckboxState::Unchecked, 2));
        // Deselecting the page's rows clears exactly those.
        s.previous_page();
        let keys = view(&s).keys;
        s.set_many_selected(keys.iter().map(String::as_str), false);
        assert_eq!(view(&s).selected_count, 0);
    }

    #[test]
    fn an_empty_page_is_not_all_selected() {
        let mut s = state();
        s.set_filter("zzz");
        assert_eq!(view(&s).select_all, CheckboxState::Unchecked);
    }
}
