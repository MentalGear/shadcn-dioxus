use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

const COUNT: usize = 40;

/// The rows and the order they show: one shared `render_row`, so the two modes differ only in
/// whether the table is told who is at an index.
#[component]
fn Rows(order: Signal<Vec<usize>>, keyed: bool) -> Element {
    let render_row = move |idx: usize| {
        let id = order.read()[idx];
        rsx! {
            TableCell {
                input {
                    r#type: "checkbox",
                    "data-item-check": "{id}",
                    "aria-label": "Mark order {id}",
                }
            }
            TableCell {
                button { r#type: "button", "data-item-button": "{id}", "Open order {id}" }
            }
            TableCell { "${id * 10 + 50}.00" }
        }
    };
    if keyed {
        rsx! {
            TableRows {
                count: COUNT,
                columns: 3,
                virtualize: VirtualizeMode::Windowed,
                item_key: move |idx: usize| format!("order-{}", order.read()[idx]),
                render_row,
            }
        }
    } else {
        rsx! {
            TableRows {
                count: COUNT,
                columns: 3,
                virtualize: VirtualizeMode::Windowed,
                render_row,
            }
        }
    }
}

/// A reorderable table body: `item_key` makes each row's DOM node follow its item, so focus and
/// an uncontrolled checkbox inside a row stay with the order they belong to when the order changes.
#[component]
pub fn Demo() -> Element {
    let mut order = use_signal(|| (0..COUNT).collect::<Vec<usize>>());
    let mut keyed = use_signal(|| true);
    rsx! {
        div { class: "flex flex-col gap-3", "data-table-reorder-demo": "",
            div { class: "flex flex-wrap items-center gap-2",
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "reverse",
                    onclick: move |_| order.with_mut(|o| o.reverse()),
                    "Sort descending"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "rotate",
                    onclick: move |_| order.with_mut(|o| o.rotate_left(1)),
                    "Move first to last"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "key-mode",
                    "aria-pressed": "{keyed()}",
                    onclick: move |_| keyed.set(!keyed()),
                    if keyed() {
                        "item_key: on"
                    } else {
                        "item_key: off"
                    }
                }
            }
            Table {
                container_attributes: vec![
                    Attribute::new("style", "max-height: 320px;", None, false),
                    Attribute::new("tabindex", "0", None, false),
                    Attribute::new("role", "region", None, false),
                    Attribute::new("aria-label", "Orders", None, false),
                ],
                style: "table-layout: fixed;",
                TableHeader { sticky: true,
                    TableRow {
                        TableHead { "Mark" }
                        TableHead { "Order" }
                        TableHead { "Amount" }
                    }
                }
                Rows { order, keyed: keyed() }
            }
        }
    }
}
