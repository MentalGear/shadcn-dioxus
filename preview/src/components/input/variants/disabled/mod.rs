use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldLabel};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Field {
                FieldLabel { html_for: "input-disabled-email", "Email" }
                Input {
                    id: "input-disabled-email",
                    r#type: "email",
                    placeholder: "Email",
                    disabled: true,
                }
                FieldDescription { "This field is currently disabled." }
            }
        }
    }
}
