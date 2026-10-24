use super::super::component::*;
use dioxus::prelude::*;

const ROWS: usize = 2_000;
const COLUMNS: usize = 8;
const ITEMS: usize = 3_000;
const CITIES: [&str; 7] = ["Lisbon", "Osaka", "Quito", "Tromsø", "Accra", "Hobart", "Recife"];

/// The text a cell shows, which is also what find-in-page searches.
fn cell_text(row: usize, column: usize) -> String {
    match column {
        0 => format!("Order {:05}", row + 1),
        1 => CITIES[(row * 3 + 1) % CITIES.len()].to_string(),
        _ => format!("{}.{:02}", (row * 31 + column * 7) % 900 + 100, (row + column) % 100),
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { "data-vg-demo": "find-in-page", style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%;",
            style { r#"[data-vg-demo="find-in-page"] [data-find-match] {{
  animation: dx-virtual-grid-find-match 2s ease-out;
}}

@keyframes dx-virtual-grid-find-match {{
  from {{ background-color: var(--dx-accent); }}
}}"# }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "Experimental: both grids keep every unmounted cell's text in the DOM as a hidden=\"until-found\" placeholder. Search the page (Ctrl+F) for \"Order 01777\" or \"Photo 2468\": the browser reveals the placeholder, and the grid brings the real cell into view and flashes it."
            }
            VirtualGrid {
                row_count: ROWS,
                column_count: COLUMNS,
                virtualize: VirtualizeMode::Windowed,
                row_height: |_| 36,
                column_width: |column: usize| if column == 0 { 120 } else { 112 },
                pinned_columns_start: 1usize,
                render_header: |column: usize| match column {
                    0 => rsx! { "Order" },
                    1 => rsx! { "City" },
                    _ => rsx! { "Reading {column - 1}" },
                },
                render_cell: |(row, column): (usize, usize)| rsx! { "{cell_text(row, column)}" },
                text_value: |(row, column): (usize, usize)| cell_text(row, column),
                "aria-label": "Searchable orders, 2,000 rows by 8 columns",
                style: "height: 18rem;",
            }
            VirtualWrapGrid {
                count: ITEMS,
                columns: 5usize,
                virtualize: VirtualizeMode::Windowed,
                // Deliberately below the real row (72: the item's 56 + 8 margin, plus the grid's own 8px
                // of row padding): a match must land on the real item, not on its estimate.
                estimate_row_height: |_| 64,
                render_item: |index: usize| rsx! {
                    div { style: "box-sizing: border-box; height: 56px; margin-block-end: 8px; padding: var(--dx-space-2); border: 1px solid var(--dx-border); border-radius: var(--dx-radius-md); font-size: var(--dx-text-sm);",
                        "Photo {index + 1}"
                    }
                },
                text_value: |index: usize| format!("Photo {}", index + 1),
                "aria-label": "Searchable gallery, 3,000 items",
                style: "height: 16rem;",
            }
        }
    }
}
