//! The `Grid` layout of [`DataTable`](super::DataTable) (backlog row 342): the same columns, the
//! same state (`use_data_table_index`, one `dioxus_primitives::data_view::derive`), drawn through
//! `VirtualGrid` so only the rows in view exist. Same `#[path]` sibling wiring as `columns.rs`.
//!
//! It reads rows by position from the derived indices (`View::indices`) and never copies them,
//! because a 50,000-row table cannot afford a clone of the data per render. The grid's hooks
//! (`column_sort` for `aria-sort`, `row_key` and `row_selected` keyed by the row's identity, a
//! checkbox in a cell) are the whole integration.

use super::columns::{use_data_table_index, DataTableOptions, DEFAULT_GRID_COLUMN_WIDTH};
use super::{Column, DataTableProps, DataTableSortButton, DataTableToolbar};
use crate::components::checkbox::Checkbox;
use crate::components::virtual_grid::{VirtualGrid, VirtualizeMode};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The height in px of a data row: a `Table` row's own height, 8px padding twice around one
/// 20px line plus the 1px row border (measured in `data_table.spec.ts`, which compares the two).
pub(super) const GRID_ROW_HEIGHT: u32 = 37;

/// The header row's height in px: `--dx-table-head-height` (2.5rem at the default root size).
pub(super) const GRID_HEADER_HEIGHT: u32 = 40;

/// The `Grid` layout: toolbar, a windowed grid, a one-line summary. No pagination: it scrolls.
#[component]
pub(super) fn DataTableGrid<T: Clone + PartialEq + 'static>(props: DataTableProps<T>) -> Element {
    let options = DataTableOptions {
        page_size: usize::MAX,
        sort: props.default_sort.clone(),
        hidden_columns: props.hidden_columns.clone(),
        selected: Vec::new(),
    };
    let model = use_data_table_index(props.rows, &props.columns, &props.row_key, options);
    let state = model.state;
    let derived = model.view;
    let rows = props.rows;
    let selectable = props.selectable;
    let offset = usize::from(selectable);
    let visible: Vec<Column<T>> = model
        .visible_columns(&props.columns)
        .into_iter()
        .cloned()
        .collect();
    let column_count = visible.len() + offset;
    let (filtered_count, selected_in_view, select_all) = {
        let view = derived.read();
        (view.filtered_count, view.selected_in_view, view.select_all)
    };
    let sort = model.sort.clone();
    let header_sort = model.sort.clone();

    let base =
        attributes!(div { class: "dx-data-table", "data-slot": "data-table", "data-dx-root": "", "data-layout": "grid" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    let widths: Vec<u32> = visible
        .iter()
        .map(|c| c.grid_width.unwrap_or(DEFAULT_GRID_COLUMN_WIDTH))
        .collect();
    let column_width = move |column: usize| {
        if selectable && column == 0 {
            48
        } else {
            widths.get(column - offset).copied().unwrap_or(DEFAULT_GRID_COLUMN_WIDTH)
        }
    };

    let sort_ids: Vec<String> = visible.iter().map(|c| c.id.clone()).collect();
    let column_sort = move |column: usize| {
        let id = sort_ids.get(column.checked_sub(offset)?)?;
        sort.as_ref()
            .filter(|(sorted, _)| sorted == id)
            .map(|(_, direction)| *direction)
    };

    let header_columns = visible.clone();
    let render_header = move |column: usize| -> Element {
        if selectable && column == 0 {
            return rsx! {
                Checkbox {
                    checked: select_all,
                    aria_label: "Select all",
                    on_checked_change: move |next: CheckboxState| {
                        let keys = derived.read().keys.clone();
                        state.set_many_selected(&keys, bool::from(next));
                    },
                }
            };
        }
        let Some(col) = header_columns.get(column - offset) else {
            return rsx! {};
        };
        let id = col.id.clone();
        let label = col.header.clone();
        let align = col.align.class();
        if col.is_sortable() {
            let sorted = header_sort.clone().filter(|(s, _)| *s == id).map(|(_, d)| d);
            rsx! {
                div { class: align,
                    DataTableSortButton { sorted, onclick: move |_| state.toggle_sort(&id), "{label}" }
                }
            }
        } else {
            rsx! {
                div { class: align, "{label}" }
            }
        }
    };

    let cell_columns = visible.clone();
    let render_cell = move |(row, column): (usize, usize)| -> Element {
        let (index, key) = {
            let view = derived.read();
            match (view.indices.get(row), view.keys.get(row)) {
                (Some(&index), Some(key)) => (index, key.clone()),
                _ => return rsx! {},
            }
        };
        if selectable && column == 0 {
            return rsx! {
                Checkbox {
                    checked: if state.is_selected(&key) { CheckboxState::Checked } else { CheckboxState::Unchecked },
                    aria_label: "Select row {key}",
                    on_checked_change: move |next: CheckboxState| state.set_selected(&key, bool::from(next)),
                }
            };
        }
        let Some(col) = cell_columns.get(column - offset) else {
            return rsx! {};
        };
        let rows = rows.read();
        let Some(data) = rows.get(index) else {
            return rsx! {};
        };
        let content = col.cell.call(data);
        match col.align.class() {
            Some(align) => rsx! {
                div { class: align, {content} }
            },
            None => content,
        }
    };

    let label = props.grid_label.clone();
    let height = props.grid_height.clone();
    let empty = (filtered_count == 0).then(|| props.empty_message.clone());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/data_table/style.css") }
        div { ..merged,
            if let Some(placeholder) = props.filter_placeholder.clone() {
                DataTableToolbar {
                    value: model.filter.clone(),
                    oninput: move |value: String| state.set_filter(value),
                    placeholder,
                    aria_label: props.filter_label.clone(),
                }
            }
            VirtualGrid {
                row_count: filtered_count,
                // Windowed at every count: a filter that crossed `Auto`'s threshold would re-mount
                // the body in the other mode mid-typing.
                virtualize: VirtualizeMode::Windowed,
                column_count,
                row_height: |_| GRID_ROW_HEIGHT,
                header_height: GRID_HEADER_HEIGHT,
                column_width,
                row_key: move |row: usize| derived.read().keys.get(row).cloned().unwrap_or_default(),
                row_selected: move |key: String| state.is_selected(&key),
                column_sort,
                render_header,
                render_cell,
                "aria-label": "{label}",
                style: "height: {height};",
            }
            if let Some(message) = empty {
                p { class: "dx-data-table-empty", role: "status", "{message}" }
            }
            div { class: "dx-data-table-pagination", "data-slot": "data-table-summary",
                div { class: "dx-data-table-pagination-selected",
                    "{selected_in_view} of {filtered_count} row(s) selected."
                }
            }
        }
    }
}
