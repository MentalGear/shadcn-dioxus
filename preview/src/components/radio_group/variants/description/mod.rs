use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            RadioGroup {
                default_value: "default".to_string(),
                name: "radio-group-description-spacing",
                aria_label: "Row spacing",
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-description-default",
                        value: "default".to_string(),
                        index: 0usize,
                    }
                    FieldContent {
                        FieldLabel { html_for: "radio-group-description-default", "Default spacing" }
                        FieldDescription { "Balanced rows for everyday browsing." }
                    }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-description-comfortable",
                        value: "comfortable".to_string(),
                        index: 1usize,
                    }
                    FieldContent {
                        FieldLabel { html_for: "radio-group-description-comfortable", "Comfortable" }
                        FieldDescription { "Extra room around every row." }
                    }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    RadioItem {
                        id: "radio-group-description-compact",
                        value: "compact".to_string(),
                        index: 2usize,
                    }
                    FieldContent {
                        FieldLabel { html_for: "radio-group-description-compact", "Compact" }
                        FieldDescription { "Fit more rows on screen at once." }
                    }
                }
            }
        }
    }
}
