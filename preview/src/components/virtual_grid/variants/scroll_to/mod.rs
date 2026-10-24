//! Scroll to a cell (row 398): a 10,000 x 50 grid with a pinned header and a pinned first column,
//! and a handle from `use_virtual_scroller` that aligns any cell, mounted or not.
use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use dioxus::prelude::*;

const ROWS: usize = 10_000;
const COLUMNS: usize = 50;
const ROW: u32 = 36;
const COLUMN: u32 = 120;
const ALIGNS: [(ScrollAlign, &str); 4] = [
    (ScrollAlign::Start, "Start"),
    (ScrollAlign::Center, "Center"),
    (ScrollAlign::End, "End"),
    (ScrollAlign::Nearest, "Nearest"),
];

#[component]
pub fn Demo() -> Element {
    let scroller = use_virtual_scroller();
    let mut align = use_signal(|| ScrollAlign::Start);
    let mut row = use_signal(|| "5000".to_string());
    let mut column = use_signal(|| "30".to_string());

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%;", "data-vg-demo": "scroll-to",
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                Input {
                    r#type: "number",
                    min: "1",
                    "aria-label": "Row number",
                    "data-vg-row-input": "",
                    value: "{row}",
                    style: "width: 7rem;",
                    oninput: move |event: FormEvent| row.set(event.value()),
                }
                Input {
                    r#type: "number",
                    min: "1",
                    "aria-label": "Column number",
                    "data-vg-column-input": "",
                    value: "{column}",
                    style: "width: 7rem;",
                    oninput: move |event: FormEvent| column.set(event.value()),
                }
                Button {
                    "data-vg-go": "",
                    onclick: move |_| {
                        if let (Ok(row), Ok(column)) = (
                            row.peek().trim().parse::<usize>(),
                            column.peek().trim().parse::<usize>(),
                        ) {
                            scroller.scroll_to_cell(
                                row.saturating_sub(1),
                                column.saturating_sub(1),
                                align(),
                            );
                        }
                    },
                    "Scroll to cell"
                }
            }
            div {
                role: "group",
                "aria-label": "Alignment",
                style: "display: flex; flex-wrap: wrap; gap: var(--dx-space-2);",
                for (value, name) in ALIGNS {
                    Button {
                        key: "{name}",
                        variant: if align() == value { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                        "aria-pressed": (align() == value).to_string(),
                        onclick: move |_| align.set(value),
                        "{name}"
                    }
                }
            }
            VirtualGrid {
                row_count: ROWS,
                column_count: COLUMNS,
                row_height: |_| ROW,
                column_width: |_| COLUMN,
                pinned_columns_start: 1usize,
                scroller,
                render_header: |column: usize| rsx! { "Column {column + 1}" },
                render_cell: |(row, column): (usize, usize)| rsx! { "{row + 1}, {column + 1}" },
                "aria-label": "10,000 rows by 50 columns, with the header and the first column pinned",
                style: "height: 24rem;",
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                "The cell lands clear of the pinned header and column at the chosen alignment; a cell in the pinned column moves only the rows."
            }
        }
    }
}
