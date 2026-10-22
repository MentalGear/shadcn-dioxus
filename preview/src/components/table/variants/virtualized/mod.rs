use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

/// The row counts the buttons switch between: either side of `Auto`'s default threshold (1,000).
const COUNTS: [usize; 3] = [20_000, 1_001, 1_000];

const STATUSES: [&str; 4] = ["Paid", "Pending", "Unpaid", "Refunded"];

#[component]
pub fn Demo() -> Element {
    let mut count = use_signal(|| COUNTS[0]);

    rsx! {
        div { class: "flex flex-col gap-3", "data-table-virtual-demo": "",
            div { class: "flex flex-wrap items-center gap-2",
                for n in COUNTS {
                    Button {
                        key: "{n}",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| count.set(n),
                        "{n} rows"
                    }
                }
            }
            Table {
                container_attributes: vec![
                    Attribute::new("style", "max-height: 320px;", None, false),
                    // A scroll container must be reachable by keyboard.
                    Attribute::new("tabindex", "0", None, false),
                    Attribute::new("role", "region", None, false),
                    Attribute::new("aria-label", "Invoices", None, false),
                ],
                // Fixed columns: the server renders no rows, so automatic layout would resize them.
                style: "table-layout: fixed;",
                TableHeader { sticky: true,
                    TableRow {
                        TableHead { "Invoice" }
                        TableHead { "Status" }
                        TableHead { "Amount" }
                    }
                }
                TableRows {
                    count: count,
                    columns: 3,
                    // A fixed selection: the third row.
                    row_selected: |idx: usize| idx == 2,
                    render_row: move |idx: usize| rsx! {
                        TableCell { "INV{idx + 1:06}" }
                        TableCell { "{STATUSES[idx % STATUSES.len()]}" }
                        TableCell { "${(idx % 97) * 10 + 50}.00" }
                    },
                }
            }
        }
    }
}
