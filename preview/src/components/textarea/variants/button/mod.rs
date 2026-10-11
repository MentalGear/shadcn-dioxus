use super::super::component::*;
use crate::components::button::Button;
use crate::components::form::DemoForm;
use dioxus::prelude::*;

/// shadcn's "Button" example: a message composer with its submit button beneath the textarea.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            DemoForm {
                id: "textarea-button-form",
                style: "display: grid; gap: 0.5rem;",
                Textarea {
                    id: "textarea-button-message",
                    name: "message",
                    "aria-label": "Message",
                    placeholder: "Type your message here.",
                }
                Button { r#type: "submit", "Send message" }
            }
        }
    }
}
