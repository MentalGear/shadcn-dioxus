//! `Column<T>` and the `use_data_table` hook (backlog row 340): the data model
//! a page describes a table with once. The pure state machine is `model.rs`;
//! this file adds the typed column definition and the signal that holds the
//! state. Same `#[path]` sibling wiring as `state.rs`.

use super::model::{derive, TableState, TableView};
use super::state::SortDirection;
use dioxus::prelude::*;
use std::cmp::Ordering;
use std::rc::Rc;

/// A shared closure over a row, compared by identity so it can sit in props
/// (`Props` need `PartialEq`; a fresh closure per render compares unequal, so
/// the table re-renders, which is what a data-driven table wants anyway).
pub struct RowFn<T: 'static, R: 'static>(Rc<dyn Fn(&T) -> R>);

impl<T: 'static, R: 'static> RowFn<T, R> {
    /// Wraps `f`.
    pub fn new(f: impl Fn(&T) -> R + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// Calls the closure on `row`.
    pub fn call(&self, row: &T) -> R {
        (self.0)(row)
    }
}

impl<T: 'static, R: 'static> Clone for RowFn<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static, R: 'static> PartialEq for RowFn<T, R> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// How a column's cells and header align.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnAlign {
    /// The start edge (left in LTR); the table default.
    #[default]
    Start,
    /// Centred.
    Center,
    /// The end edge (right in LTR), for numbers.
    End,
}

impl ColumnAlign {
    pub(crate) fn class(self) -> Option<&'static str> {
        match self {
            ColumnAlign::Start => None,
            ColumnAlign::Center => Some("dx-data-table-align-center"),
            ColumnAlign::End => Some("dx-data-table-align-end"),
        }
    }
}

type Comparator<T> = Rc<dyn Fn(&T, &T) -> Ordering>;

/// One column of a data table: an id, a header, how to render a cell, and
/// (optionally) a typed sort key, filter text, width and alignment.
///
/// ```ignore
/// Column::new("email", "Email", |p: &Payment| rsx! { "{p.email}" })
///     .sortable(|p| p.email.clone())
///     .filterable(|p| p.email.clone())
///     .align(ColumnAlign::Start)
/// ```
pub struct Column<T: 'static> {
    /// Stable id: what `aria-sort`, the sort state and a header activation
    /// refer to. Unique within a table.
    pub id: String,
    /// The header text.
    pub header: String,
    /// Renders this column's cell for a row.
    pub cell: RowFn<T, Element>,
    /// The ascending order of this column's sort key; `None` = not sortable.
    pub(crate) compare: Option<Comparator<T>>,
    /// The text the filter matches for a row; `None` = not filtered.
    pub filter_text: Option<RowFn<T, String>>,
    /// An optional CSS width for the column (`"8rem"`).
    pub width: Option<String>,
    /// Header and cell alignment.
    pub align: ColumnAlign,
}

impl<T: 'static> Column<T> {
    /// A plain, unsorted, unfiltered column.
    pub fn new(id: impl Into<String>, header: impl Into<String>, cell: impl Fn(&T) -> Element + 'static) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            cell: RowFn::new(cell),
            compare: None,
            filter_text: None,
            width: None,
            align: ColumnAlign::Start,
        }
    }

    /// Makes the column sortable by a typed key (`K: Ord`), so the sort cannot
    /// name a column or type that does not exist and the header gets the sort
    /// button and `aria-sort`.
    #[must_use]
    pub fn sortable<K: Ord>(mut self, key: impl Fn(&T) -> K + 'static) -> Self {
        self.compare = Some(Rc::new(move |a, b| key(a).cmp(&key(b))));
        self
    }

    /// Makes the table's filter input match this column's text (a
    /// case-insensitive substring; a row matches when any filterable column
    /// does).
    #[must_use]
    pub fn filterable(mut self, text: impl Fn(&T) -> String + 'static) -> Self {
        self.filter_text = Some(RowFn::new(text));
        self
    }

    /// Sets the column's CSS width.
    #[must_use]
    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets header and cell alignment.
    #[must_use]
    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    /// Whether the header offers a sort button.
    pub fn is_sortable(&self) -> bool {
        self.compare.is_some()
    }
}

impl<T: 'static> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            header: self.header.clone(),
            cell: self.cell.clone(),
            compare: self.compare.clone(),
            filter_text: self.filter_text.clone(),
            width: self.width.clone(),
            align: self.align,
        }
    }
}

impl<T: 'static> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.header == other.header
            && self.cell == other.cell
            && self.width == other.width
            && self.align == other.align
            && match (&self.compare, &other.compare) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && self.filter_text == other.filter_text
    }
}

/// The initial state of [`use_data_table`]. Read once, on the first render.
#[derive(Debug, Clone, PartialEq)]
pub struct DataTableOptions {
    /// Rows per page; `usize::MAX` shows everything on one page.
    pub page_size: usize,
    /// The initially sorted column and direction.
    pub sort: Option<(String, SortDirection)>,
}

impl Default for DataTableOptions {
    fn default() -> Self {
        Self { page_size: 10, sort: None }
    }
}

/// The mutating half of a data table: `Copy`, so event handlers capture it
/// freely. Every method is a [`TableState`] transition.
#[derive(Clone, Copy)]
pub struct DataTableState {
    state: Signal<TableState>,
}

impl DataTableState {
    /// The current state.
    pub fn get(&self) -> TableState {
        self.state.read().clone()
    }

    /// A header activation for `column_id`; see [`TableState::toggle_sort`].
    pub fn toggle_sort(&self, column_id: &str) {
        let mut state = self.state;
        state.with_mut(|s| s.toggle_sort(column_id));
    }

    /// Sets the filter text.
    pub fn set_filter(&self, text: impl Into<String>) {
        let mut state = self.state;
        let text = text.into();
        state.with_mut(|s| s.set_filter(text));
    }

    /// Sets rows per page.
    pub fn set_page_size(&self, page_size: usize) {
        let mut state = self.state;
        state.with_mut(|s| s.set_page_size(page_size));
    }

    /// One page back.
    pub fn previous_page(&self) {
        let mut state = self.state;
        state.with_mut(TableState::previous_page);
    }

    /// One page forward, stopping at the last of `page_count`.
    pub fn next_page(&self, page_count: usize) {
        let mut state = self.state;
        state.with_mut(|s| s.next_page(page_count));
    }

    /// Selects or deselects one row.
    pub fn set_selected(&self, key: &str, selected: bool) {
        let mut state = self.state;
        state.with_mut(|s| s.set_selected(key, selected));
    }

    /// Selects or deselects every key (the header checkbox).
    pub fn set_many_selected(&self, keys: &[String], selected: bool) {
        let mut state = self.state;
        state.with_mut(|s| s.set_many_selected(keys.iter().map(String::as_str), selected));
    }
}

/// What [`use_data_table`] returns: the mutators plus this render's view.
pub struct DataTableModel<T: 'static> {
    /// The mutating half; `Copy`.
    pub state: DataTableState,
    /// The derived rows and counts for this render (subscribes to the state
    /// and to `rows`).
    pub view: TableView<T>,
    /// The sorted column and its direction, for `aria-sort`.
    pub sort: Option<(String, SortDirection)>,
    /// The filter text.
    pub filter: String,
    /// Rows per page.
    pub page_size: usize,
    selected: std::collections::HashSet<String>,
}

impl<T: 'static> DataTableModel<T> {
    /// This column's sort direction, `None` when another column is sorted.
    pub fn sorted(&self, column_id: &str) -> Option<SortDirection> {
        self.sort.as_ref().filter(|(id, _)| id == column_id).map(|(_, d)| *d)
    }

    /// Whether the row with `key` is selected.
    pub fn is_selected(&self, key: &str) -> bool {
        self.selected.contains(key)
    }
}

/// Owns a data table's sort, filter, page and selection state and derives what
/// to render: `rows` is the full data, `columns` say how each column sorts and
/// filters, `row_key` gives a row a stable identity for selection.
///
/// `options` is read on the first render only. Render the returned
/// [`DataTableModel::view`] into `Table` yourself, or hand the same inputs to
/// the `DataTable` component, which does exactly that.
pub fn use_data_table<T: Clone + 'static>(
    rows: ReadSignal<Vec<T>>,
    columns: &[Column<T>],
    row_key: &RowFn<T, String>,
    options: DataTableOptions,
) -> DataTableModel<T> {
    let state = use_signal(|| TableState::new(options.page_size, options.sort));
    let current = state.read().clone();
    let compare = current
        .sort
        .as_ref()
        .and_then(|(id, _)| columns.iter().find(|c| &c.id == id))
        .and_then(|c| c.compare.clone());
    let all = rows.read();
    let view = derive(
        &all,
        &current,
        |row| columns.iter().filter_map(|c| c.filter_text.as_ref().map(|f| f.call(row))).collect(),
        compare.as_deref(),
        |row| row_key.call(row),
    );
    DataTableModel {
        state: DataTableState { state },
        view,
        sort: current.sort,
        filter: current.filter,
        page_size: current.page_size,
        selected: current.selected,
    }
}
