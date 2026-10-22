use super::super::component::*;
use crate::components::checkbox::Checkbox;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use std::collections::HashSet;

const ROWS: usize = 10_000;

/// The columns after the checkbox column: header, and a key into the row's data.
const COLUMNS: [&str; 3] = ["Order", "Status", "Total"];

/// A deterministic order for item `id`, so the demo needs no data set.
fn order_number(id: usize) -> usize {
    (id * 7_919) % ROWS
}

fn status(id: usize) -> &'static str {
    ["Paid", "Pending", "Failed", "Refunded"][(id * 31) % 4]
}

fn total_cents(id: usize) -> usize {
    ((id as u64 + 1).wrapping_mul(2_654_435_761) % 100_000) as usize
}

/// Item ids in display order for the sort state (`column` is 1-based after the checkbox column).
fn sorted_ids(sort: Option<(usize, SortDirection)>) -> Vec<usize> {
    let mut ids: Vec<usize> = (0..ROWS).collect();
    if let Some((column, direction)) = sort {
        ids.sort_by(|a, b| {
            let ordering = match column {
                1 => order_number(*a).cmp(&order_number(*b)),
                2 => status(*a).cmp(status(*b)).then(a.cmp(b)),
                _ => total_cents(*a).cmp(&total_cents(*b)),
            };
            match direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            }
        });
    }
    ids
}

#[component]
pub fn Demo() -> Element {
    let mut sort = use_signal(|| None::<(usize, SortDirection)>);
    // Selection is a set of item ids (the row key), so it follows a row through a sort.
    let mut selected = use_signal(HashSet::<String>::new);
    let ids = use_memo(move || sorted_ids(sort()));
    let count = selected.read().len();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-2); width: 100%;",
            VirtualGrid {
                row_count: ROWS,
                column_count: 4usize,
                row_height: |_| 36,
                column_width: |column: usize| match column {
                    0 => 48,
                    1 => 140,
                    _ => 120,
                },
                row_key: move |row: usize| format!("{}", ids.read()[row]),
                row_selected: move |key: String| selected.read().contains(&key),
                column_sort: move |column: usize| {
                    sort().filter(|(sorted, _)| *sorted == column).map(|(_, direction)| direction)
                },
                render_header: move |column: usize| {
                    if column == 0 {
                        rsx! { span { style: "position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%);", "Choose" } }
                    } else {
                        let label = COLUMNS[column - 1];
                        rsx! {
                            button {
                                r#type: "button",
                                style: "padding: 0; border: 0; background: none; color: inherit; font: inherit; cursor: pointer;",
                                onclick: move |_| {
                                    let next = match sort() {
                                        Some((sorted, direction)) if sorted == column => direction.toggled(),
                                        _ => SortDirection::Ascending,
                                    };
                                    sort.set(Some((column, next)));
                                },
                                "{label}"
                                match sort() {
                                    Some((sorted, SortDirection::Ascending)) if sorted == column => " \u{2191}",
                                    Some((sorted, SortDirection::Descending)) if sorted == column => " \u{2193}",
                                    _ => "",
                                }
                            }
                        }
                    }
                },
                render_cell: move |(row, column): (usize, usize)| {
                    let id = ids.read()[row];
                    match column {
                        0 => rsx! {
                            Checkbox {
                                checked: if selected.read().contains(&id.to_string()) {
                                    CheckboxState::Checked
                                } else {
                                    CheckboxState::Unchecked
                                },
                                aria_label: "Choose order {order_number(id) + 1}",
                                on_checked_change: move |state: CheckboxState| {
                                    selected.with_mut(|set| {
                                        if bool::from(state) {
                                            set.insert(id.to_string());
                                        } else {
                                            set.remove(&id.to_string());
                                        }
                                    });
                                },
                            }
                        },
                        1 => rsx! { "ORD-{order_number(id) + 1:05}" },
                        2 => rsx! { "{status(id)}" },
                        _ => rsx! { "${total_cents(id) / 100}.{total_cents(id) % 100:02}" },
                    }
                },
                "aria-label": "Orders, 10,000 rows, sortable and selectable",
                style: "height: 24rem;",
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "{count} selected. Tab reaches one checkbox; the arrow keys move between cells and Space toggles the focused row."
            }
        }
    }
}
