use super::super::component::*;
use dioxus::prelude::*;

const ROWS: usize = 10_000;
const COLUMNS: usize = 50;

/// A deterministic "measurement" for a cell, so the demo needs no data set.
fn value(row: usize, column: usize) -> String {
    let seed = (row as u64 + 1).wrapping_mul(2_654_435_761) ^ (column as u64 + 1).wrapping_mul(40_503);
    format!("{:.2}", (seed % 100_000) as f64 / 100.0)
}

#[component]
pub fn Demo() -> Element {
    let mut active = use_signal(|| (0usize, 0usize));
    let (row, column) = active();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-2); width: 100%;",
            VirtualGrid {
                row_count: ROWS,
                column_count: COLUMNS,
                row_height: |_| 36,
                column_width: |column: usize| if column == 0 { 80 } else { 112 },
                sticky_first_column: true,
                render_header: |column: usize| {
                    if column == 0 {
                        rsx! { "Row" }
                    } else {
                        rsx! { "Sensor {column}" }
                    }
                },
                render_cell: |(row, column): (usize, usize)| {
                    if column == 0 {
                        rsx! { "{row + 1}" }
                    } else {
                        rsx! { "{value(row, column)}" }
                    }
                },
                on_active_change: move |cell| active.set(cell),
                "aria-label": "Sensor readings, 10,000 rows by 50 columns",
                style: "height: 24rem;",
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "Active cell: row {row + 1}, column {column + 1}. Click a cell, then use the arrow keys, Home/End, Ctrl+Home/End and PageUp/PageDown."
            }
        }
    }
}
