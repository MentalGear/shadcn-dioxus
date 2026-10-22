use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

/// Pressed state follows an item's `value`, not its position: two groups over one list (an
/// uncontrolled multiple group and a controlled single one) keep the same items pressed when the list
/// is reversed or an item is inserted before them.
#[component]
pub fn Demo() -> Element {
    let mut items = use_signal(|| {
        ["bold", "italic", "underline"]
            .map(String::from)
            .to_vec()
    });
    let mut inserted = use_signal(|| 0usize);
    let mut chosen = use_signal(|| "italic".to_string());
    let value = use_memo(move || Some(HashSet::from([chosen()])));
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3);",
            div { style: "display: flex; gap: var(--dx-space-2);",
                Button {
                    variant: ButtonVariant::Outline,
                    r#type: "button",
                    onclick: move |_| items.write().reverse(),
                    "Reverse order"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    r#type: "button",
                    onclick: move |_| {
                        let n = inserted() + 1;
                        inserted.set(n);
                        items.write().insert(0, format!("new{n}"));
                    },
                    "Insert first"
                }
            }
            ToggleGroup {
                allow_multiple_pressed: true,
                default_value: HashSet::from(["italic".to_string()]),
                aria_label: "Reorderable multiple",
                for (index , item) in items().into_iter().enumerate() {
                    ToggleItem { key: "{item}", index, value: item.clone(), aria_label: "multi {item}", "{item}" }
                }
            }
            ToggleGroup {
                value,
                on_value_change: move |next: HashSet<String>| {
                    if let Some(key) = next.into_iter().next() {
                        chosen.set(key);
                    }
                },
                aria_label: "Reorderable single",
                for (index , item) in items().into_iter().enumerate() {
                    ToggleItem { key: "{item}", index, value: item.clone(), aria_label: "single {item}", "{item}" }
                }
            }
            p { "data-testid": "toggle-group-chosen", "Chosen: {chosen}" }
        }
    }
}
