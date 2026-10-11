use super::super::component::*;
use crate::components::field::{
    Field, FieldDescription, FieldError, FieldLabel, FieldLegend, FieldOrientation, FieldSet,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldSet {
                FieldLegend { "Notification channel" }
                FieldDescription { "Choose how we reach you about account activity." }
                RadioGroup {
                    name: "radio-group-invalid-channel",
                    aria_label: "Notification channel",
                    aria_invalid: "true",
                    Field { orientation: FieldOrientation::Horizontal, invalid: true,
                        RadioItem {
                            id: "radio-group-invalid-email",
                            value: "email".to_string(),
                            index: 0usize,
                            aria_invalid: "true",
                        }
                        FieldLabel { html_for: "radio-group-invalid-email", "Email" }
                    }
                    Field { orientation: FieldOrientation::Horizontal, invalid: true,
                        RadioItem {
                            id: "radio-group-invalid-sms",
                            value: "sms".to_string(),
                            index: 1usize,
                            aria_invalid: "true",
                        }
                        FieldLabel { html_for: "radio-group-invalid-sms", "Text message" }
                    }
                    Field { orientation: FieldOrientation::Horizontal, invalid: true,
                        RadioItem {
                            id: "radio-group-invalid-push",
                            value: "push".to_string(),
                            index: 2usize,
                            aria_invalid: "true",
                        }
                        FieldLabel { html_for: "radio-group-invalid-push", "Push notification" }
                    }
                }
                FieldError { "Select a notification channel to continue." }
            }
        }
    }
}
