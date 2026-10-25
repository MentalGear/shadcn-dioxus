use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

/// Each option is a `FieldLabel` wrapping a `Field`, so the whole card is the
/// click target for its radio; `FieldLabel` draws the card (border, padding,
/// checked tint) itself whenever it wraps a `Field`. The `aria_label`s name each
/// radio directly: `Field` is a `role="group"`, and axe does not read a label's
/// name through a group, so the wrapping `<label for>` alone leaves it unnamed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            RadioGroup {
                default_value: "plus".to_string(),
                name: "radio-group-choice-card-plan",
                aria_label: "Billing plan",
                FieldLabel {
                    html_for: "radio-group-choice-card-plus",
                    Field { orientation: FieldOrientation::Horizontal,
                        FieldContent {
                            "Plus"
                            FieldDescription { "For individuals and small teams." }
                        }
                        RadioItem {
                            id: "radio-group-choice-card-plus",
                            aria_label: "Plus",
                            value: "plus".to_string(),
                        }
                    }
                }
                FieldLabel {
                    html_for: "radio-group-choice-card-pro",
                    Field { orientation: FieldOrientation::Horizontal,
                        FieldContent {
                            "Pro"
                            FieldDescription { "For growing businesses." }
                        }
                        RadioItem {
                            id: "radio-group-choice-card-pro",
                            aria_label: "Pro",
                            value: "pro".to_string(),
                        }
                    }
                }
                FieldLabel {
                    html_for: "radio-group-choice-card-enterprise",
                    Field { orientation: FieldOrientation::Horizontal,
                        FieldContent {
                            "Enterprise"
                            FieldDescription { "For large teams with custom needs." }
                        }
                        RadioItem {
                            id: "radio-group-choice-card-enterprise",
                            aria_label: "Enterprise",
                            value: "enterprise".to_string(),
                        }
                    }
                }
            }
        }
    }
}
