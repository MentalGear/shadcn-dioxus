use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldError, FieldLabel};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Field { invalid: true,
                FieldLabel { html_for: "input-invalid-address", "Email address" }
                Input {
                    id: "input-invalid-address",
                    r#type: "email",
                    "aria-invalid": "true",
                    value: "max@example",
                }
                FieldDescription { "We will only use this to send your receipts." }
                FieldError { "Enter a valid email address, like max@example.com." }
            }
        }
    }
}
