use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

/// A read-only group keeps its selection: the items stay focusable and the arrow keys move focus,
/// but a click, Space or an arrow key does not select another item.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            RadioGroup {
                read_only: true,
                default_value: "standard".to_string(),
                name: "radio-group-read-only-shipping",
                aria_label: "Shipping speed",
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-read-only-standard",
                        value: "standard".to_string(),
                    }
                    FieldLabel { html_for: "radio-group-read-only-standard", "Standard shipping" }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-read-only-express",
                        value: "express".to_string(),
                    }
                    FieldLabel { html_for: "radio-group-read-only-express", "Express shipping" }
                }
            }
        }
    }
}
