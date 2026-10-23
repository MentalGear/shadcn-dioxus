use super::super::component::*;
use dioxus::prelude::*;

/// A demo caption: a title line and a one-sentence description.
#[component]
fn Caption(title: &'static str, children: Element) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.125rem;",
            p { style: "margin: 0; color: var(--dx-foreground); font-size: var(--dx-text-sm); font-weight: 500;",
                "{title}"
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                {children}
            }
        }
    }
}

/// `fade` on both layouts: `Y` on the wrapping gallery, `Both` on the two-axis data grid. The fade is a mask on the grid's own scroll container, driven by its
/// scroll offset, so it follows the scrollbar whatever cells are mounted. A mask also fades the
/// container's border, so the border sits on a wrapper and the grid drops its own.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-4); width: 100%;",
            Caption {
                title: "Wrapping gallery",
                "ScrollFade::Y on a VirtualWrapGrid: it scrolls vertically, so the top and bottom edges fade."
            }
            div {
                "data-demo": "fade-wrap",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
                VirtualWrapGrid {
                    count: 2_000usize,
                    columns: 4usize,
                    // A tile (4rem + 2 x 4px margin) plus the row's 8px bottom padding: exact, so the canvas is the
                    // same height before and after the rows are measured.
                    estimate_row_height: |_| 80,
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
            Caption {
                title: "Two-axis grid",
                "ScrollFade::Both on a VirtualGrid that scrolls rows and columns: all four edges fade, each only while there is more content past it."
            }
            div {
                "data-demo": "fade-grid",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
                // Rows AND columns scroll, so all four edges fade (each only while there is more past it).
                // No sticky header: the mask covers everything inside the container, header included.
                VirtualGrid {
                    row_count: 1_000usize,
                    column_count: 30usize,
                    render_cell: |(row, column): (usize, usize)| rsx! { "R{row + 1} C{column + 1}" },
                    fade: ScrollFade::Both,
                    "aria-label": "Readings, faded at all four edges",
                    style: "height: 16rem; border: none; border-radius: 0;",
                }
            }
        }
    }
}
