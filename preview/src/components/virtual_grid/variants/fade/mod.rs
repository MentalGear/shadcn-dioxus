use super::super::component::*;
use dioxus::prelude::*;

/// `fade` on both layouts. The fade is a mask on the grid's own scroll container, driven by its
/// scroll offset, so it follows the scrollbar whatever cells are mounted. A mask also fades the
/// container's border, so the border sits on a wrapper and the grid drops its own.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-4); width: 100%;",
            div {
                "data-demo": "fade-wrap",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
                VirtualWrapGrid {
                    count: 2_000usize,
                    columns: 4usize,
                    estimate_row_height: |_| 72,
                    fade: ScrollFade::Y,
                    render_item: |index: usize| rsx! {
                        div { style: "display: grid; place-items: center; height: 4rem; margin: var(--dx-space-1); border-radius: var(--dx-radius-md); background: var(--dx-muted); color: var(--dx-muted-foreground);",
                            "Tile {index + 1}"
                        }
                    },
                    "aria-label": "Tiles, faded at the scroll edges",
                    style: "height: 16rem; border: none; border-radius: 0;",
                }
            }
            div {
                "data-demo": "fade-grid",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
                VirtualGrid {
                    row_count: 1_000usize,
                    column_count: 30usize,
                    // The sticky header would fade with a top edge, so only the far edges fade.
                    render_header: |column: usize| rsx! { "Column {column + 1}" },
                    render_cell: |(row, column): (usize, usize)| rsx! { "R{row + 1} C{column + 1}" },
                    fade: ScrollFade::Bottom,
                    "aria-label": "Readings, faded at the bottom edge",
                    style: "height: 16rem; border: none; border-radius: 0;",
                }
            }
        }
    }
}
