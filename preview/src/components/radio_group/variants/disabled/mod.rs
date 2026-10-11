use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            RadioGroup {
                default_value: "standard".to_string(),
                name: "radio-group-disabled-shipping",
                aria_label: "Shipping speed",
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-disabled-standard",
                        value: "standard".to_string(),
                        index: 0usize,
                    }
                    FieldLabel { html_for: "radio-group-disabled-standard", "Standard shipping" }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-disabled-express",
                        value: "express".to_string(),
                        index: 1usize,
                    }
                    FieldLabel { html_for: "radio-group-disabled-express", "Express shipping" }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-disabled-overnight",
                        value: "overnight".to_string(),
                        index: 2usize,
                        disabled: true,
                    }
                    FieldLabel { html_for: "radio-group-disabled-overnight", "Overnight shipping (unavailable)" }
                }
            }
        }
    }
}
