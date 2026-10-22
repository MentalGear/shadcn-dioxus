//! Themed Data Table composition pieces -- shadcn/ui catalog parity
//! (dev-docs/component-backlog.md row 69).
//!
//! Like `Table` (row 68), there is no `dioxus_primitives::data_table`
//! primitive: shadcn/ui's own Data Table is a composition pattern built
//! from `Table` plus existing `Select`/`Checkbox`/`Button`/`Input`, not a
//! new accessible widget with its own ARIA role. This file's three
//! components (`DataTableColumnHeader`, `DataTablePagination`,
//! `DataTableToolbar`) are that same composition, reusable across any
//! table shaped like the demo's payments table -- every element they
//! render comes from `crate::components::{table, button, select, input,
//! checkbox}`'s own themed wrappers (the preview composition rule,
//! dev-docs/preview-composition.md), never a raw element or primitive of
//! their own. The actual sort/filter/pagination *state* is owned by the
//! caller (see `variants/main/mod.rs`); `state.rs` (this folder) is the
//! pure, unit-tested transform logic those state signals are fed through.

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::checkbox::Checkbox;
use crate::components::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowDown, ArrowUp, ArrowUpDown};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

use dioxus_primitives::checkbox::CheckboxState;

// A consumer-facing surface: this crate's own demos use a subset of it.
#[allow(dead_code)]
#[path = "columns.rs"]
mod columns;
#[allow(dead_code)]
#[path = "model.rs"]
mod model;
#[allow(dead_code)]
#[path = "state.rs"]
mod state;

#[allow(unused_imports)]
pub use columns::{use_data_table, Column, ColumnAlign, DataTableModel, DataTableOptions, DataTableState, RowFn};
#[allow(unused_imports)]
pub use model::{derive, TableState, TableView};
#[allow(unused_imports)]
pub use state::{filter_rows, includes_string, paginate, sort_rows, sort_rows_by, tri_state, SortDirection};

/// Props for [`DataTableColumnHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableColumnHeaderProps {
    /// This column's current sort direction, or `None` when a different
    /// column (or no column at all) is the one currently sorted.
    ///
    /// The APG Table pattern's sortable-table example
    /// (`playwright/oracle/reference/7e4034b/content/patterns/table/
    /// examples/sortable-table.html`, "Role, Property, State, and Tabindex
    /// Attributes"): "`aria-sort=\"value\"` ... \[s\]et on the currently
    /// sorted column. When the sorted column is changed, the `aria-sort`
    /// attribute is removed and set on the newly sorted column." This
    /// component instead always *sets* `aria-sort`, valued `"none"` for
    /// every sortable column that isn't the active one (rather than
    /// omitting the attribute the way the reference page's static HTML
    /// does) -- `"none"` is `aria-sort`'s own documented default value, so
    /// an explicit `"none"` and an absent attribute are equivalent to
    /// assistive tech, and explicit is easier for this crate's own
    /// oracle/axe scans to assert against uniformly. See
    /// `playwright/oracle/tier1-apg/sortable-table.spec.ts` for the rule
    /// this is calibrated against.
    pub sorted: Option<SortDirection>,

    /// Fired when the header's sort button is activated (pointer or
    /// keyboard -- it's a real `button`, so both come for free from the
    /// browser). The caller owns which column is active and the
    /// ascending/descending toggle (`SortDirection::toggled`); this
    /// component is purely presentational.
    pub onclick: EventHandler<MouseEvent>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The column's label, e.g. `"Email"`.
    pub children: Element,
}

/// A sortable column header: a [`TableHead`] whose content is a ghost
/// [`Button`] that toggles the column's sort and shows the current
/// direction, with `aria-sort` on the `th` itself (not the button) per the
/// APG Table pattern. See [`DataTableColumnHeaderProps::sorted`] for the
/// exact `aria-sort` value convention.
#[component]
pub fn DataTableColumnHeader(props: DataTableColumnHeaderProps) -> Element {
    let aria_sort = props.sorted.map_or("none", SortDirection::aria_sort);
    let base = attributes!(th { class: "dx-data-table-column-header" });
    // `aria-sort` is required APG Table-pattern state this wrapper computes
    // from `props.sorted`, not a caller default -- owned-wins.
    let owned = attributes!(th { aria_sort });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/data_table/style.css") }
        TableHead {
            attributes: merged,
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                class: "dx-data-table-column-header-button",
                onclick: move |event| props.onclick.call(event),
                {props.children}
                match props.sorted {
                    Some(SortDirection::Ascending) => rsx! {
                        ArrowUp { class: "dx-data-table-sort-icon", size: "16px" }
                    },
                    Some(SortDirection::Descending) => rsx! {
                        ArrowDown { class: "dx-data-table-sort-icon", size: "16px" }
                    },
                    None => rsx! {
                        ArrowUpDown { class: "dx-data-table-sort-icon", size: "16px" }
                    },
                }
            }
        }
    }
}

/// Props for [`DataTableToolbar`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableToolbarProps {
    /// The filter input's current (controlled) value.
    pub value: ReadSignal<String>,

    /// Fired on every keystroke with the input's new value.
    pub oninput: Callback<String>,

    /// Placeholder text, e.g. `"Filter emails..."`.
    #[props(default)]
    pub placeholder: String,

    /// Accessible name for the filter input -- required, since a bare
    /// placeholder is not an accessible name (it disappears once there is
    /// a value, and isn't exposed to every assistive technology the same
    /// way in the first place).
    pub aria_label: String,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A toolbar holding a text filter [`Input`](crate::components::input::Input).
#[component]
pub fn DataTableToolbar(props: DataTableToolbarProps) -> Element {
    let base = attributes!(div { class: "dx-data-table-toolbar", "data-slot": "data-table-toolbar" });
    let merged = merge_attributes(vec![base, props.attributes]);
    let value = (props.value)();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/data_table/style.css") }
        div { ..merged,
            crate::components::input::Input {
                class: "dx-data-table-filter",
                "data-slot": "data-table-filter",
                r#type: "text",
                placeholder: props.placeholder.clone(),
                aria_label: props.aria_label.clone(),
                value: "{value}",
                oninput: move |event: FormEvent| props.oninput.call(event.value()),
            }
        }
    }
}

/// Props for [`DataTablePagination`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTablePaginationProps {
    /// Number of rows currently selected (across every page, not just the
    /// one visible).
    pub selected_count: usize,

    /// Number of rows in the filtered result set (before pagination).
    pub total_count: usize,

    /// 0-based index of the current page.
    pub page: usize,

    /// Total number of pages (always >= 1 -- see `state::paginate`).
    pub page_count: usize,

    /// Rows shown per page.
    pub page_size: usize,

    /// The rows-per-page choices offered in the `Select`.
    pub page_size_options: Vec<usize>,

    /// Fired with the newly chosen page size.
    pub on_page_size_change: Callback<usize>,

    /// Fired when "Previous" is activated. Never fired while already on
    /// the first page -- the button is `disabled` there instead.
    pub on_previous: Callback<()>,

    /// Fired when "Next" is activated. Never fired while already on the
    /// last page -- the button is `disabled` there instead.
    pub on_next: Callback<()>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The Data Table's footer bar: a "x of y row(s) selected" summary, a
/// rows-per-page [`Select`](crate::components::select::Select), a "Page n
/// of m" readout, and Previous/Next [`Button`]s, disabled at either end.
#[component]
pub fn DataTablePagination(props: DataTablePaginationProps) -> Element {
    let base = attributes!(div { class: "dx-data-table-pagination", "data-slot": "data-table-pagination" });
    let merged = merge_attributes(vec![base, props.attributes]);
    let at_start = props.page == 0;
    let at_end = props.page + 1 >= props.page_count;
    let page_size = props.page_size;
    let on_page_size_change = props.on_page_size_change;
    let on_previous = props.on_previous;
    let on_next = props.on_next;

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/data_table/style.css") }
        div { ..merged,
            div { class: "dx-data-table-pagination-selected",
                "{props.selected_count} of {props.total_count} row(s) selected."
            }
            div { class: "dx-data-table-pagination-controls",
                div { class: "dx-data-table-pagination-size",
                    span { class: "dx-data-table-pagination-size-label", aria_hidden: "true", "Rows per page" }
                    crate::components::select::Select::<usize> {
                        "data-slot": "data-table-page-size",
                        trigger_aria_label: "Rows per page",
                        default_value: page_size,
                        item_to_label: |size: usize| size.to_string(),
                        // Static trigger text: the default `SelectValue` prints "Select an option" until the options
                        // register, then "5" (~18 px narrower), which re-flowed the footer (CLS 0.0131 > 0.01).
                        trigger: rsx! {
                            span { "{page_size}" }
                        },
                        on_value_change: move |value: Option<usize>| {
                            if let Some(value) = value {
                                on_page_size_change.call(value);
                            }
                        },
                        for (i , option) in props.page_size_options.iter().copied().enumerate() {
                            crate::components::select::SelectOption::<usize> {
                                key: "{option}",
                                index: i,
                                value: option,
                                text_value: "{option}",
                                "{option}"
                            }
                        }
                    }
                }
                div { class: "dx-data-table-pagination-page", "Page {props.page + 1} of {props.page_count}" }
                div { class: "dx-data-table-pagination-nav",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: at_start,
                        onclick: move |_| on_previous.call(()),
                        "Previous"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: at_end,
                        onclick: move |_| on_next.call(()),
                        "Next"
                    }
                }
            }
        }
    }
}

/// Props for [`DataTable`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps<T: Clone + PartialEq + 'static> {
    /// The full data (every page, before filtering). Pass a `Vec<T>` or a
    /// signal; the table re-derives when it changes.
    pub rows: ReadSignal<Vec<T>>,

    /// The column definitions, in display order: one `Column` per header and
    /// cell, so the two cannot drift apart.
    pub columns: Vec<Column<T>>,

    /// A stable key for a row (a database id, not an index). Selection is
    /// kept by key, so it survives sorting, filtering and paging.
    pub row_key: RowFn<T, String>,

    /// Adds a leading checkbox column with a header "select all" for the
    /// current page.
    #[props(default)]
    pub selectable: bool,

    /// Shows a filter input with this placeholder, matching every column
    /// marked `filterable`. `None` hides the toolbar.
    #[props(default)]
    pub filter_placeholder: Option<String>,

    /// The filter input's accessible name; required in practice whenever
    /// `filter_placeholder` is set.
    #[props(default = "Filter rows".to_string())]
    pub filter_label: String,

    /// The rows-per-page choices. Empty (the default) shows every row and no
    /// pagination bar; otherwise the first choice is the initial page size.
    #[props(default)]
    pub page_sizes: Vec<usize>,

    /// The initially sorted column id and direction.
    #[props(default)]
    pub default_sort: Option<(String, SortDirection)>,

    /// What the body says when no row matches.
    #[props(default = "No results.".to_string())]
    pub empty_message: String,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A data-driven table: describe the columns once and the component renders
/// the toolbar, the [`Table`] with sortable headers (`aria-sort` on the `th`),
/// row selection with an all/some/none header checkbox, and the pagination
/// bar, on top of [`use_data_table`]. Use the hook directly for a custom
/// layout.
#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(props: DataTableProps<T>) -> Element {
    let options = DataTableOptions {
        page_size: props.page_sizes.first().copied().unwrap_or(usize::MAX),
        sort: props.default_sort.clone(),
    };
    let model = use_data_table(props.rows, &props.columns, &props.row_key, options);
    let state = model.state;
    let view = &model.view;
    let column_count = props.columns.len() + usize::from(props.selectable);
    let page_keys = view.keys.clone();
    let page_count = view.page_count;
    let base = attributes!(div { class: "dx-data-table", "data-slot": "data-table", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

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
            Table {
                TableHeader {
                    TableRow {
                        if props.selectable {
                            TableHead {
                                Checkbox {
                                    checked: view.select_all,
                                    aria_label: "Select all",
                                    on_checked_change: move |next: CheckboxState| {
                                        state.set_many_selected(&page_keys, bool::from(next));
                                    },
                                }
                            }
                        }
                        for column in props.columns.iter() {
                            {column_header(column, &model)}
                        }
                    }
                }
                TableBody {
                    if view.rows.is_empty() {
                        TableRow {
                            TableCell { colspan: column_count, "{props.empty_message}" }
                        }
                    }
                    for (row , key) in view.rows.iter().zip(view.keys.iter()) {
                        TableRow { key: "{key}", selected: model.is_selected(key),
                            if props.selectable {
                                TableCell {
                                    Checkbox {
                                        checked: if model.is_selected(key) {
                                            CheckboxState::Checked
                                        } else {
                                            CheckboxState::Unchecked
                                        },
                                        aria_label: "Select row {key}",
                                        on_checked_change: {
                                            let key = key.clone();
                                            move |next: CheckboxState| state.set_selected(&key, bool::from(next))
                                        },
                                    }
                                }
                            }
                            for column in props.columns.iter() {
                                TableCell {
                                    class: column.align.class(),
                                    {column.cell.call(row)}
                                }
                            }
                        }
                    }
                }
            }
            if !props.page_sizes.is_empty() {
                DataTablePagination {
                    selected_count: view.selected_count,
                    total_count: view.filtered_count,
                    page: view.page,
                    page_count,
                    page_size: model.page_size,
                    page_size_options: props.page_sizes.clone(),
                    on_page_size_change: move |size| state.set_page_size(size),
                    on_previous: move |_| state.previous_page(),
                    on_next: move |_| state.next_page(page_count),
                }
            }
        }
    }
}

/// One header cell: the sort button when the column is sortable, else plain text.
fn column_header<T: Clone + 'static>(column: &Column<T>, model: &DataTableModel<T>) -> Element {
    let state = model.state;
    let id = column.id.clone();
    let align = column.align.class();
    let style = column.width.as_ref().map(|w| format!("width: {w};"));
    if column.is_sortable() {
        rsx! {
            DataTableColumnHeader {
                key: "{column.id}",
                sorted: model.sorted(&column.id),
                onclick: move |_| state.toggle_sort(&id),
                class: align,
                style,
                "{column.header}"
            }
        }
    } else {
        rsx! {
            TableHead { key: "{column.id}", class: align, style, "{column.header}" }
        }
    }
}
