use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldLabel};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Field {
                FieldLabel { html_for: "input-required-handle",
                    "Handle"
                    span {
                        "aria-hidden": "true",
                        style: "color: var(--dx-destructive-text);",
                        " *"
                    }
                }
                Input {
                    id: "input-required-handle",
                    placeholder: "@maxleiter",
                    required: true,
                }
                FieldDescription { "Required. Shown on your public profile." }
            }
        }
    }
}
