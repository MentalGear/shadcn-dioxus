use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

const NAMES: [(&str, &str); 4] = [
    ("alpha", "Alpha"),
    ("bravo", "Bravo"),
    ("charlie", "Charlie"),
    ("delta", "Delta"),
];

/// The order lives here, in the app: the list shows `order` and reports every
/// move through `on_reorder`, so the app can persist it, and may rewrite it
/// itself (the Reverse button).
#[component]
pub fn Demo() -> Element {
    let mut order = use_signal(|| NAMES.map(|(key, _)| key.to_string()).to_vec());
    let items = NAMES
        .map(|(key, label)| {
            rsx! {
                span { key: "{key}", "{label}" }
            }
        })
        .to_vec();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 12px; align-items: flex-start;",
            DragAndDropList {
                items,
                order: ReadSignal::from(order),
                on_reorder: move |next: Vec<String>| order.set(next),
                aria_label: "Controlled list",
            }
            p { "data-testid": "reported-order", "{order.read().join(\",\")}" }
            Button {
                variant: ButtonVariant::Outline,
                r#type: "button",
                onclick: move |_| {
                    let mut next = order();
                    next.reverse();
                    order.set(next);
                },
                "Reverse"
            }
        }
    }
}
