//! Pinned rows and columns (row 388): a 5,000-row grid whose first two columns are pinned at the
//! inline start, whose last column (a selection checkbox) is pinned at the inline end, and whose
//! header row is pinned unless the switch turns `pin_header` off.
use super::super::component::*;
use crate::components::checkbox::Checkbox;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use crate::components::switch::Switch;
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use std::collections::HashSet;

const ROWS: usize = 5_000;
/// Site, region, 26 weekly readings and the selection column.
const COLUMNS: usize = 29;
const SELECT_COLUMN: usize = COLUMNS - 1;

fn region(row: usize) -> &'static str {
    ["North", "South", "East", "West"][(row * 7) % 4]
}

/// A deterministic weekly reading, so the demo needs no data set.
fn reading(row: usize, column: usize) -> String {
    let seed = (row as u64 + 1).wrapping_mul(2_654_435_761) ^ (column as u64 + 1).wrapping_mul(40_503);
    format!("{:.1}", (seed % 10_000) as f64 / 10.0)
}

#[component]
pub fn Demo() -> Element {
    let mut pin_header = use_signal(|| true);
    // Selection is a set of row numbers as strings (the row key).
    let mut selected = use_signal(HashSet::<String>::new);
    let count = selected.read().len();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%;",
            Field { orientation: FieldOrientation::Horizontal,
                Switch {
                    id: "virtual-grid-pin-header",
                    checked: pin_header(),
                    on_checked_change: move |value| pin_header.set(value),
                }
                FieldLabel { html_for: "virtual-grid-pin-header", "Pin the header row" }
            }
            VirtualGrid {
                row_count: ROWS,
                column_count: COLUMNS,
                row_height: |_| 36,
                column_width: |column: usize| match column {
                    0 => 112,
                    1 => 96,
                    SELECT_COLUMN => 80,
                    _ => 104,
                },
                pin_header: pin_header(),
                pinned_columns_start: 2usize,
                pinned_columns_end: 1usize,
                row_selected: move |key: String| selected.read().contains(&key),
                render_header: |column: usize| match column {
                    0 => rsx! { "Site" },
                    1 => rsx! { "Region" },
                    SELECT_COLUMN => rsx! { "Select" },
                    _ => rsx! { "Week {column - 1}" },
                },
                render_cell: move |(row, column): (usize, usize)| match column {
                    0 => rsx! { "Site {row + 1}" },
                    1 => rsx! { "{region(row)}" },
                    SELECT_COLUMN => rsx! {
                        Checkbox {
                            checked: if selected.read().contains(&row.to_string()) {
                                CheckboxState::Checked
                            } else {
                                CheckboxState::Unchecked
                            },
                            aria_label: "Select site {row + 1}",
                            on_checked_change: move |state: CheckboxState| {
                                selected.with_mut(|set| {
                                    if bool::from(state) {
                                        set.insert(row.to_string());
                                    } else {
                                        set.remove(&row.to_string());
                                    }
                                });
                            },
                        }
                    },
                    _ => rsx! { "{reading(row, column)}" },
                },
                "aria-label": "Weekly readings, 5,000 sites by 29 columns, with the site and region columns pinned at the start and the selection column pinned at the end",
                style: "height: 24rem;",
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "{count} selected. Scroll sideways: Site and Region stay at the start, Select stays at the end, and the header stays on top while it is pinned."
            }
        }
    }
}
