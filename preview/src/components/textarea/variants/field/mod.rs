use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldLabel};
use dioxus::prelude::*;

/// shadcn's "Field" example: a label above the textarea and a helper line below it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field {
                FieldLabel { html_for: "textarea-field-message", "Your message" }
                Textarea {
                    id: "textarea-field-message",
                    placeholder: "Type your message here.",
                }
                FieldDescription { "Your message will be copied to the support team." }
            }
        }
    }
}
