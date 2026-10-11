use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal, invalid: true,
                FieldContent {
                    FieldLabel { html_for: "switch-invalid-terms", "Accept terms and conditions" }
                    FieldDescription {
                        "You must accept the terms before creating an account."
                    }
                    FieldError { "Accept the terms to continue." }
                }
                Switch { id: "switch-invalid-terms", aria_invalid: "true" }
            }
        }
    }
}
