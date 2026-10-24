use super::super::component::*;
use dioxus::prelude::*;

/// Five rows in a box that fits twenty: the canvas is the rows' own extent, so the server's markup
/// and the first client frame agree (row 368). Windowed on purpose: under `Auto` fifteen cells would
/// all be mounted.
#[component]
pub fn Demo() -> Element {
    rsx! {
        VirtualGrid {
            row_count: 5usize,
            virtualize: VirtualizeMode::Windowed,
            column_count: 3usize,
            row_height: |_| 36,
            column_width: |_| 112,
            render_header: |column: usize| rsx! { "Column {column + 1}" },
            render_cell: |(row, column): (usize, usize)| rsx! { "R{row + 1} C{column + 1}" },
            "aria-label": "Three columns, five rows, in a box taller than they are",
            style: "height: 24rem;",
        }
    }
}
