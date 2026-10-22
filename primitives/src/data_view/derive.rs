//! [`derive`]: the one pipeline `filter -> sort or manual order -> page`.

use super::field::Field;
use super::stages::{paginate, tri_state, SortDirection};
use super::state::ViewState;
use super::text::{contains_query_normalized, normalize_query};
use crate::checkbox::CheckboxState;
use std::cmp::Ordering;
use std::collections::HashMap;

/// An ascending comparator over rows.
pub type RowCompare<'a, T> = dyn Fn(&T, &T) -> Ordering + 'a;

/// What a view shows for one [`ViewState`]: indices into the source rows and
/// counts, never rows. Counts are computed, never stored.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    /// Indices into the source rows of the current page, filtered and sorted.
    pub indices: Vec<usize>,
    /// The key of each of `indices`, in order.
    pub keys: Vec<String>,
    /// Rows left after filtering, before pagination.
    pub filtered_count: usize,
    /// Number of pages (at least 1).
    pub page_count: usize,
    /// The current page, clamped to the last page.
    pub page: usize,
    /// Selected rows that pass the filter, on every page: the count the
    /// footer shows ("`selected_in_view` of `filtered_count`", as shadcn does).
    /// Always `<= filtered_count`.
    pub selected_in_view: usize,
    /// Selected rows in total, including ones the filter hides.
    pub selected_total: usize,
    /// The header checkbox: all, some or none of the current page's rows.
    pub select_all: CheckboxState,
}

/// [`derive`] for rows described by [`Field`]s: the active sort field's
/// comparator, and the filter text of every filterable field.
pub fn derive<T: 'static>(
    rows: &[T],
    state: &ViewState,
    fields: &[Field<T>],
    key: impl Fn(&T) -> String,
) -> View {
    let compare = state
        .sort
        .as_ref()
        .and_then(|(id, _)| fields.iter().find(|f| &f.id == id))
        .and_then(Field::comparator);
    derive_by(
        rows,
        state,
        |row| fields.iter().filter_map(|f| f.text_of(row)).collect(),
        compare,
        key,
    )
}

/// Filters, sorts and paginates `rows` for `state`, returning indices.
///
/// `filter_text` lists the texts the filter may match for a row (a row matches
/// when any contains the query, `contains_query` semantics; no texts means no
/// field is filterable, so every row matches). `compare` is the active sort
/// field's ascending comparator, if any. `key` is the stable row key. With no
/// sort active a manual [`ViewState::order`] arranges the filtered rows
/// (reconciled: a permutation of the filtered keys by construction).
pub fn derive_by<T>(
    rows: &[T],
    state: &ViewState,
    filter_text: impl Fn(&T) -> Vec<String>,
    compare: Option<&RowCompare<T>>,
    key: impl Fn(&T) -> String,
) -> View {
    let needle = normalize_query(&state.filter);
    let mut filtered: Vec<usize> = (0..rows.len())
        .filter(|&i| {
            let texts = filter_text(&rows[i]);
            texts.is_empty()
                || texts
                    .iter()
                    .any(|text| contains_query_normalized(text, &needle))
        })
        .collect();
    let filtered_count = filtered.len();

    // Keys of the filtered rows are needed for the selection count and a manual order only.
    let wants_keys = !state.selected.is_empty() || (state.sort.is_none() && state.order.is_some());
    let filtered_keys: Vec<String> = if wants_keys {
        filtered.iter().map(|&i| key(&rows[i])).collect()
    } else {
        Vec::new()
    };
    let selected_in_view = filtered_keys
        .iter()
        .filter(|k| state.selected.contains(*k))
        .count();

    match (&state.sort, compare, &state.order) {
        (Some((_, direction)), Some(compare), _) => filtered.sort_by(|&a, &b| match direction {
            SortDirection::Ascending => compare(&rows[a], &rows[b]),
            SortDirection::Descending => compare(&rows[b], &rows[a]),
        }),
        (None, _, Some(order)) => {
            let position: HashMap<&str, usize> = order
                .keys()
                .iter()
                .enumerate()
                .rev()
                .map(|(i, k)| (k.as_str(), i))
                .collect();
            let mut keyed: Vec<(usize, &String)> =
                filtered.iter().copied().zip(filtered_keys.iter()).collect();
            keyed.sort_by_key(|(_, k)| position.get(k.as_str()).copied().unwrap_or(usize::MAX));
            filtered = keyed.into_iter().map(|(i, _)| i).collect();
        }
        _ => {}
    }

    let (indices, page_count) = paginate(&filtered, state.page, state.page_size);
    let keys: Vec<String> = indices.iter().map(|&i| key(&rows[i])).collect();
    let selected_here = keys.iter().filter(|k| state.selected.contains(*k)).count();
    View {
        indices,
        filtered_count,
        page_count,
        page: state.page.min(page_count - 1),
        selected_in_view,
        selected_total: state.selected.len(),
        select_all: tri_state(selected_here, keys.len()),
        keys,
    }
}

/// What a table renders for one [`ViewState`]: a [`View`] with the page's rows
/// cloned out (only the page, never the filtered or sorted set).
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
    /// Selected rows that pass the filter; see [`View::selected_in_view`].
    pub selected_in_view: usize,
    /// Selected rows in total, including ones the filter hides.
    pub selected_total: usize,
    /// Deprecated alias of [`selected_total`](Self::selected_total).
    #[deprecated(
        note = "use `selected_in_view` for the footer count, `selected_total` for every selected row"
    )]
    pub selected_count: usize,
    /// The header checkbox: all, some or none of the current page's rows.
    pub select_all: CheckboxState,
}

impl<T: Clone> TableView<T> {
    /// The page of `rows` that `view` selects.
    pub fn from_view(rows: &[T], view: &View) -> Self {
        #[allow(deprecated)]
        Self {
            rows: view.indices.iter().map(|&i| rows[i].clone()).collect(),
            keys: view.keys.clone(),
            filtered_count: view.filtered_count,
            page_count: view.page_count,
            page: view.page,
            selected_in_view: view.selected_in_view,
            selected_total: view.selected_total,
            selected_count: view.selected_total,
            select_all: view.select_all,
        }
    }
}

/// [`derive_by`] with the page's rows cloned into a [`TableView`] (the Data
/// Table's original `derive`).
pub fn derive_rows<T: Clone>(
    rows: &[T],
    state: &ViewState,
    filter_text: impl Fn(&T) -> Vec<String>,
    compare: Option<&RowCompare<T>>,
    key: impl Fn(&T) -> String,
) -> TableView<T> {
    TableView::from_view(rows, &derive_by(rows, state, filter_text, compare, key))
}

#[cfg(test)]
mod tests {
    use super::super::state::{KeyOrder, TableState};
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Row {
        id: &'static str,
        name: &'static str,
        n: i32,
    }

    const ROWS: &[Row] = &[
        Row {
            id: "a",
            name: "carol",
            n: 3,
        },
        Row {
            id: "b",
            name: "alice",
            n: 1,
        },
        Row {
            id: "c",
            name: "bob",
            n: 2,
        },
        Row {
            id: "d",
            name: "dana",
            n: 4,
        },
        Row {
            id: "e",
            name: "erin",
            n: 5,
        },
    ];

    fn view(state: &TableState) -> TableView<Row> {
        let by_n = |a: &Row, b: &Row| a.n.cmp(&b.n);
        derive_rows(
            ROWS,
            state,
            |r| vec![r.name.to_string()],
            Some(&by_n),
            |r| r.id.to_string(),
        )
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
        let v = derive_rows(ROWS, &s, |_| Vec::new(), None, |r| r.id.to_string());
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
        assert_eq!(view(&s).selected_total, 1);
        assert!(s.selected.contains("d"));
        s.set_selected("d", false);
        assert_eq!(view(&s).selected_total, 0);
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
        assert_eq!(v.selected_total, 2);
        // Page 2 is untouched: unchecked there, the in-view count still spans pages.
        s.next_page(3);
        let v = view(&s);
        assert_eq!(
            (v.select_all, v.selected_in_view),
            (CheckboxState::Unchecked, 2)
        );
        // Deselecting the page's rows clears exactly those.
        s.previous_page();
        let keys = view(&s).keys;
        s.set_many_selected(keys.iter().map(String::as_str), false);
        assert_eq!(view(&s).selected_total, 0);
    }

    #[test]
    fn an_empty_page_is_not_all_selected() {
        let mut s = state();
        s.set_filter("zzz");
        assert_eq!(view(&s).select_all, CheckboxState::Unchecked);
    }

    #[test]
    fn the_count_is_selected_in_view_never_more_than_filtered() {
        let mut s = TableState::new(5, None);
        for id in ["a", "b", "d"] {
            s.set_selected(id, true);
        }
        s.set_filter("dana");
        let v = view(&s);
        assert_eq!(
            (v.selected_in_view, v.filtered_count, v.selected_total),
            (1, 1, 3)
        );
        s.set_filter("zzz");
        let v = view(&s);
        assert_eq!(
            (v.selected_in_view, v.filtered_count, v.selected_total),
            (0, 0, 3)
        );
        #[allow(deprecated)]
        {
            assert_eq!(v.selected_count, 3);
        }
    }

    #[test]
    fn a_manual_order_applies_without_a_sort_and_is_reconciled() {
        let mut s = TableState::new(10, None);
        // "zz" is gone from the data, "e" and "a" are listed, the rest are new.
        s.order = Some(KeyOrder::new(["e", "zz", "a"].map(String::from)));
        assert_eq!(ids(&view(&s)), vec!["e", "a", "b", "c", "d"]);
        // A sort wins over the manual order.
        s.toggle_sort("n");
        assert_eq!(ids(&view(&s)), vec!["b", "c", "a", "d", "e"]);
    }

    #[test]
    fn fields_drive_the_same_pipeline() {
        let fields = [
            Field::<Row>::new("name").filterable(|r| r.name.to_string()),
            Field::<Row>::new("n").sortable(|r| r.n),
        ];
        let mut s = TableState::new(10, None);
        s.toggle_sort("n");
        s.toggle_sort("n");
        s.set_filter("a");
        let v = derive(ROWS, &s, &fields, |r| r.id.to_string());
        assert_eq!(v.keys, vec!["d", "a", "b"]);
        assert_eq!(v.indices, vec![3, 0, 1]);
    }
}
