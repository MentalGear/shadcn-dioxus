//! `Column<T>` and the `use_data_table` hook (backlog row 340): the data model
//! a page describes a table with once. The pure state machine is `model.rs`;
//! this file adds the typed column definition and the signal that holds the
//! state. Same `#[path]` sibling wiring as `state.rs`.

use super::layout::ColumnLayout;
use super::model::{TableState, TableView};
use super::state::SortDirection;
use dioxus::prelude::*;
use dioxus_primitives::data_view::{derive, Field, RowAnchor, View};
use std::rc::Rc;

/// The width in px of a column in the `Grid` layout when [`Column::grid_width`] is not set.
pub const DEFAULT_GRID_COLUMN_WIDTH: u32 = 160;

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

/// One column of a data table: an id, a header, how to render a cell, and
/// (optionally) a typed sort key, filter text, width and alignment. The sort
/// key and filter text are a [`Field`] of `dioxus_primitives::data_view`; the
/// rest is what a table adds to draw it.
///
/// ```ignore
/// Column::new("email", "Email", |p: &Payment| rsx! { "{p.email}" })
///     .sortable(|p| p.email.clone())
///     .filterable(|p| p.email.clone())
///     .align(ColumnAlign::Start)
/// ```
pub struct Column<T: 'static> {
    /// Stable id: what `aria-sort`, the sort state and a header activation
    /// refer to. Unique within a table. Same as `field.id`.
    pub id: String,
    /// The sort key and filter text (`id` is repeated in it).
    pub field: Field<T>,
    /// The header text.
    pub header: String,
    /// Renders this column's cell for a row.
    pub cell: RowFn<T, Element>,
    /// An optional CSS width for the column (`"8rem"`).
    pub width: Option<String>,
    /// The column's width in px in the `Grid` layout, where a virtualized grid needs a number
    /// (`None`: [`DEFAULT_GRID_COLUMN_WIDTH`]).
    pub grid_width: Option<u32>,
    /// Header and cell alignment.
    pub align: ColumnAlign,
    /// Whether the Columns menu offers this column (a visibility toggle and a
    /// keyboard move). `true` unless [`Column::hideable`] says otherwise.
    pub hideable: bool,
}

impl<T: 'static> Column<T> {
    /// A plain, unsorted, unfiltered column.
    pub fn new(
        id: impl Into<String>,
        header: impl Into<String>,
        cell: impl Fn(&T) -> Element + 'static,
    ) -> Self {
        let id = id.into();
        Self {
            field: Field::new(id.clone()),
            id,
            header: header.into(),
            cell: RowFn::new(cell),
            width: None,
            grid_width: None,
            align: ColumnAlign::Start,
            hideable: true,
        }
    }

    /// Makes the column sortable by a typed key (`K: Ord`), so the sort cannot
    /// name a column or type that does not exist and the header gets the sort
    /// button and `aria-sort`.
    #[must_use]
    pub fn sortable<K: Ord>(mut self, key: impl Fn(&T) -> K + 'static) -> Self {
        self.field = self.field.sortable(key);
        self
    }

    /// Makes the table's filter input match this column's text (a
    /// case-insensitive substring; a row matches when any filterable column
    /// does).
    #[must_use]
    pub fn filterable(mut self, text: impl Fn(&T) -> String + 'static) -> Self {
        self.field = self.field.filterable(text);
        self
    }

    /// Sets the column's CSS width.
    #[must_use]
    pub fn width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the column's width in px for the `Grid` layout (the `Table` layout uses
    /// [`width`](Self::width)).
    #[must_use]
    pub fn grid_width(mut self, px: u32) -> Self {
        self.grid_width = Some(px);
        self
    }

    /// Sets header and cell alignment.
    #[must_use]
    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    /// Whether the Columns menu offers this column. `false` keeps it out of the
    /// menu (always shown, never moved by it), like TanStack's `enableHiding: false`.
    #[must_use]
    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }

    /// Whether the header offers a sort button.
    pub fn is_sortable(&self) -> bool {
        self.field.is_sortable()
    }
}

impl<T: 'static> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            field: self.field.clone(),
            header: self.header.clone(),
            cell: self.cell.clone(),
            width: self.width.clone(),
            grid_width: self.grid_width,
            align: self.align,
            hideable: self.hideable,
        }
    }
}

impl<T: 'static> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.header == other.header
            && self.cell == other.cell
            && self.width == other.width
            && self.grid_width == other.grid_width
            && self.align == other.align
            && self.hideable == other.hideable
            && self.field == other.field
    }
}

/// The initial state of [`use_data_table`]. Read once, on the first render.
#[derive(Debug, Clone, PartialEq)]
pub struct DataTableOptions {
    /// Rows per page; `usize::MAX` shows everything on one page.
    pub page_size: usize,
    /// The initially sorted column and direction.
    pub sort: Option<(String, SortDirection)>,
    /// Ids of the columns that start hidden (the Columns menu shows them again).
    pub hidden_columns: Vec<String>,
    /// Keys of the rows that start selected. Applied when the state is
    /// created, so the server markup and the client's first frame agree
    /// (setting a selection from a hook after the first read would not).
    pub selected: Vec<String>,
}

impl Default for DataTableOptions {
    fn default() -> Self {
        Self {
            page_size: 10,
            sort: None,
            hidden_columns: Vec::new(),
            selected: Vec::new(),
        }
    }
}

/// The mutating half of a data table: `Copy`, so event handlers capture it
/// freely. Every method is a [`TableState`] transition.
#[derive(Clone, Copy)]
pub struct DataTableState {
    state: Signal<TableState>,
    layout: Signal<ColumnLayout>,
    all_keys: Callback<(), Vec<String>>,
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

    /// Whether the row with `key` is selected. Subscribes the caller to the state, so a render
    /// that asks (the `Grid` layout's `row_selected` hook) re-runs when the selection changes.
    pub fn is_selected(&self, key: &str) -> bool {
        self.state.read().selected.contains(key)
    }

    /// Selects or deselects one row.
    pub fn set_selected(&self, key: &str, selected: bool) {
        let mut state = self.state;
        state.with_mut(|s| s.set_selected(key, selected));
    }

    /// The manual row move behind the drag handle: row `key` lands next to the
    /// visible row `anchor`, in the order of every row of the data. Only
    /// meaningful while no sort is active (a sort wins over a manual order).
    /// `false` when nothing moved.
    pub fn move_row(&self, key: &str, anchor: RowAnchor<'_>) -> bool {
        let mut state = self.state;
        let keys = self.all_keys.call(());
        state.with_mut(|s| s.move_row(keys.iter().map(String::as_str), key, anchor))
    }

    /// The current column layout (order and hidden columns).
    pub fn layout(&self) -> ColumnLayout {
        self.layout.read().clone()
    }

    /// Shows or hides the column `id`.
    pub fn set_column_hidden(&self, id: &str, hidden: bool) {
        let mut layout = self.layout;
        layout.with_mut(|l| l.set_hidden(id, hidden));
    }

    /// Moves the column `id` one place (`delta` -1 earlier, 1 later) among
    /// `listed`, the columns of `all` the Columns menu offers; see
    /// [`ColumnLayout::move_listed`]. The new position among `listed`, or
    /// `None` when nothing moved.
    pub fn move_column(
        &self,
        all: &[&str],
        listed: &[&str],
        id: &str,
        delta: isize,
    ) -> Option<usize> {
        let mut layout = self.layout;
        layout.with_mut(|l| l.move_listed(all, listed, id, delta))
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
    /// Which columns are shown and in what order.
    pub layout: ColumnLayout,
    selected: std::collections::HashSet<String>,
}

impl<T: 'static> DataTableModel<T> {
    /// This column's sort direction, `None` when another column is sorted.
    pub fn sorted(&self, column_id: &str) -> Option<SortDirection> {
        sorted_in(self.sort.as_ref(), column_id)
    }

    /// The columns to render, in layout order, hidden ones left out.
    pub fn visible_columns<'a>(&self, columns: &'a [Column<T>]) -> Vec<&'a Column<T>> {
        visible_in(&self.layout, columns)
    }

    /// Whether the row with `key` is selected.
    pub fn is_selected(&self, key: &str) -> bool {
        self.selected.contains(key)
    }
}

/// What [`use_data_table_index`] returns: the same state as a [`DataTableModel`], but the rows
/// are an index, not a copy. For a layout that reads rows by position from a long list (the
/// `Grid` layout over tens of thousands of rows), where cloning every row on each render would
/// cost more than the window it draws.
pub struct DataTableIndex<T: 'static> {
    /// The mutating half; `Copy`.
    pub state: DataTableState,
    /// The derived indices into `rows`, their keys and the counts. A memo: read it where the
    /// row is rendered, never clone it per render.
    pub view: Memo<View>,
    /// The sorted column and its direction, for `aria-sort`.
    pub sort: Option<(String, SortDirection)>,
    /// The filter text.
    pub filter: String,
    /// Which columns are shown and in what order.
    pub layout: ColumnLayout,
    marker: std::marker::PhantomData<fn(&T)>,
}

impl<T: 'static> DataTableIndex<T> {
    /// This column's sort direction, `None` when another column is sorted.
    pub fn sorted(&self, column_id: &str) -> Option<SortDirection> {
        sorted_in(self.sort.as_ref(), column_id)
    }

    /// The columns to render, in layout order, hidden ones left out.
    pub fn visible_columns<'a>(&self, columns: &'a [Column<T>]) -> Vec<&'a Column<T>> {
        visible_in(&self.layout, columns)
    }
}

fn sorted_in(sort: Option<&(String, SortDirection)>, column_id: &str) -> Option<SortDirection> {
    sort.filter(|(id, _)| id == column_id).map(|(_, d)| *d)
}

fn visible_in<'a, T: 'static>(layout: &ColumnLayout, columns: &'a [Column<T>]) -> Vec<&'a Column<T>> {
    let ids = layout.visible(columns.iter().map(|c| c.id.as_str()));
    ids.into_iter()
        .filter_map(|id| columns.iter().find(|c| c.id == id))
        .collect()
}

/// Owns a data table's sort, filter, page and selection state and derives what
/// to render: `rows` is the full data, `columns` say how each column sorts and
/// filters, `row_key` gives a row a stable identity for selection.
///
/// `options` is read on the first render only. The derivation
/// (`dioxus_primitives::data_view::derive`) is memoised on the state, the rows
/// and the columns, and works on indices: only the current page's rows are
/// cloned. Render the returned [`DataTableModel::view`] into `Table` yourself,
/// or hand the same inputs to the `DataTable` component, which does exactly that.
pub fn use_data_table<T: Clone + 'static>(
    rows: ReadSignal<Vec<T>>,
    columns: &[Column<T>],
    row_key: &RowFn<T, String>,
    options: DataTableOptions,
) -> DataTableModel<T> {
    let index = use_data_table_index(rows, columns, row_key, options);
    let current = index.state.state.read().clone();
    let view = TableView::from_view(&rows.read(), &index.view.read());
    DataTableModel {
        state: index.state,
        view,
        sort: index.sort,
        filter: index.filter,
        page_size: current.page_size,
        layout: index.layout,
        selected: current.selected,
    }
}

/// [`use_data_table`] without the row copies: the derived [`View`] (indices and keys) stays in a
/// memo and the caller reads rows from `rows` by index. The state and its options are the same;
/// give `page_size: usize::MAX` for a layout that scrolls instead of paging.
pub fn use_data_table_index<T: Clone + 'static>(
    rows: ReadSignal<Vec<T>>,
    columns: &[Column<T>],
    row_key: &RowFn<T, String>,
    options: DataTableOptions,
) -> DataTableIndex<T> {
    let state = use_signal(|| {
        let mut initial = TableState::new(options.page_size, options.sort);
        initial.set_many_selected(options.selected.iter().map(String::as_str), true);
        initial
    });
    let layout =
        use_signal(|| ColumnLayout::with_hidden(options.hidden_columns.iter().map(String::as_str)));
    let fields: Vec<Field<T>> = columns.iter().map(|c| c.field.clone()).collect();
    let row_key_for_keys = row_key.clone();
    let row_key = row_key.clone();
    let derived: Memo<View> = use_memo(use_reactive!(|(fields, row_key)| {
        derive(&rows.read(), &state.read(), &fields, |row| {
            row_key.call(row)
        })
    }));
    // Every row's key, for a manual move only (read when a row is dropped, never while rendering).
    let mut key_fn = use_signal(|| row_key_for_keys.clone());
    if *key_fn.peek() != row_key_for_keys {
        key_fn.set(row_key_for_keys);
    }
    let all_keys = use_callback(move |()| {
        rows.peek()
            .iter()
            .map(|r| key_fn.peek().call(r))
            .collect::<Vec<String>>()
    });
    let sort = state.read().sort.clone();
    let filter = state.read().filter.clone();
    let layout_now = layout.read().clone();
    DataTableIndex {
        state: DataTableState {
            state,
            layout,
            all_keys,
        },
        view: derived,
        sort,
        filter,
        layout: layout_now,
        marker: std::marker::PhantomData,
    }
}
