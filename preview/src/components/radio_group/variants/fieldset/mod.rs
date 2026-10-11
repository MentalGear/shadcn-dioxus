use super::super::component::*;
use crate::components::field::{
    Field, FieldDescription, FieldLabel, FieldLegend, FieldOrientation, FieldSet,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldSet {
                FieldLegend { "Subscription plan" }
                FieldDescription { "Pick how often you would like to be billed." }
                RadioGroup {
                    default_value: "monthly".to_string(),
                    name: "radio-group-fieldset-plan",
                    aria_label: "Subscription plan",
                    Field { orientation: FieldOrientation::Horizontal,
                        RadioItem {
                            id: "radio-group-fieldset-monthly",
                            value: "monthly".to_string(),
                            index: 0usize,
                        }
                        FieldLabel { html_for: "radio-group-fieldset-monthly", "Monthly ($9.99/month)" }
                    }
                    Field { orientation: FieldOrientation::Horizontal,
                        RadioItem {
                            id: "radio-group-fieldset-yearly",
                            value: "yearly".to_string(),
                            index: 1usize,
                        }
                        FieldLabel { html_for: "radio-group-fieldset-yearly", "Yearly ($99.99/year)" }
                    }
                    Field { orientation: FieldOrientation::Horizontal,
                        RadioItem {
                            id: "radio-group-fieldset-lifetime",
                            value: "lifetime".to_string(),
                            index: 2usize,
                        }
                        FieldLabel { html_for: "radio-group-fieldset-lifetime", "Lifetime ($299.99)" }
                    }
                }
            }
        }
    }
}
