use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { "data-vg-demo": "content-visibility", style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%;",
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "Under the default Auto mode a grid of up to 1,000 cells keeps every cell in the DOM and lets the browser skip the rows that are off-screen: find-in-page, selection and screen readers reach every cell, and the keyboard works as in a windowed grid."
            }
            VirtualGrid {
                row_count: 60usize,
                column_count: 12usize,
                row_height: |_| 36,
                column_width: |column: usize| if column == 0 { 80 } else { 96 },
                sticky_first_column: true,
                render_header: |column: usize| if column == 0 { rsx! { "Row" } } else { rsx! { "Col {column}" } },
                render_cell: |(row, column): (usize, usize)| if column == 0 {
                    rsx! { "{row + 1}" }
                } else {
                    rsx! { "R{row + 1} C{column}" }
                },
                "aria-label": "60 rows by 12 columns, every cell in the DOM",
                style: "height: 16rem;",
            }
            VirtualWrapGrid {
                count: 240usize,
                columns: 6usize,
                estimate_row_height: |_| 64,
                render_item: |index: usize| rsx! {
                    div { style: "box-sizing: border-box; height: 56px; margin-block-end: 8px; padding: var(--dx-space-2); border: 1px solid var(--dx-border); border-radius: var(--dx-radius-md); font-size: var(--dx-text-sm);",
                        "Tile {index + 1}"
                    }
                },
                "aria-label": "240 tiles, every tile in the DOM",
                style: "height: 14rem;",
            }
        }
    }
}
