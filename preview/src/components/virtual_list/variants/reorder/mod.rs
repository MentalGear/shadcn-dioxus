use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

const COUNT: usize = 200;

/// How many sentences item `id` carries, so its height is a property of the item (0..=15 lines).
fn repeats(id: usize) -> usize {
    let mut x = id.wrapping_mul(2654435761);
    x ^= x >> 16;
    x ^= x >> 8;
    x % 16
}

/// The list and the order it shows: one shared `render_item`, so the two modes differ only in
/// whether the list is told who is at an index.
#[component]
fn Rows(order: Signal<Vec<usize>>, keyed: bool) -> Element {
    let render_item = move |idx: usize| {
        let id = order.read()[idx];
        let text = "Variable content. ".repeat(repeats(id));
        rsx! {
            article { class: "dx-virtual-list-card", "data-item": "{id}",
                h3 { class: "dx-virtual-list-card-title", "Item {id}" }
                div { class: "flex items-center gap-3",
                    button { r#type: "button", "data-item-button": "{id}", "Focus item {id}" }
                    input {
                        r#type: "checkbox",
                        "data-item-check": "{id}",
                        "aria-label": "Mark item {id}",
                    }
                }
                p { "{text}" }
            }
        }
    };
    // The same list twice, once with `item_key` and once without (the default, a row is its slot).
    if keyed {
        rsx! {
            VirtualList {
                count: COUNT,
                buffer: 6usize,
                virtualize: VirtualizeMode::Windowed,
                estimate_size: |_idx| 96,
                item_key: move |idx: usize| format!("item-{}", order.read()[idx]),
                render_item,
            }
        }
    } else {
        rsx! {
            VirtualList {
                count: COUNT,
                buffer: 6usize,
                virtualize: VirtualizeMode::Windowed,
                estimate_size: |_idx| 96,
                render_item,
            }
        }
    }
}

/// A reorderable list: the order is data, `item_key` says who is at an index, so focus, a row's
/// own state (the checkbox) and the measured heights follow the item when the order changes.
#[component]
pub fn Demo() -> Element {
    let mut order = use_signal(|| (0..COUNT).collect::<Vec<usize>>());
    let mut keyed = use_signal(|| true);
    rsx! {
        div { class: "dx-virtual-list-demo", "data-vl-reorder-demo": "",
            style { r#".dx-virtual-list-container {{
  position: relative;
  max-height: 24rem;
  contain: layout paint;
  overflow-y: auto;
}}

.dx-virtual-list-demo {{
  display: flex;
  flex-direction: column;
  margin: 0 auto;
  gap: 0.75rem;
}}

.dx-virtual-list-card {{
  padding: 0.75rem 0.9rem;
  border: 1px solid var(--dx-border);
  border-radius: 0.625rem;
  background: var(--dx-card);
}}

.dx-virtual-list-card-title {{
  margin: 0 0 0.3rem;
  color: var(--dx-foreground);
  font-size: 0.95rem;
  font-weight: 660;
}}

.dx-virtual-list-card p {{
  margin: 0.3rem 0 0;
  color: var(--dx-muted-foreground);
  font-size: 0.875rem;
  line-height: 1.4;
}}"# }
            div { class: "flex flex-wrap items-center gap-2",
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "reverse",
                    onclick: move |_| order.with_mut(|o| o.reverse()),
                    "Reverse the list"
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
            Rows { order, keyed: keyed() }
        }
    }
}
