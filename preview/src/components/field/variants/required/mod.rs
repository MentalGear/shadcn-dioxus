use super::super::component::*;
use crate::components::input::Input;
use dioxus::prelude::*;

/// `FieldLabel { required }` draws the marker; the input carries its own `required`. The second field has
/// neither, for comparison.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldGroup {
                Field {
                    FieldLabel { html_for: "field-required-email", required: true, "Email" }
                    Input {
                        id: "field-required-email",
                        r#type: "email",
                        placeholder: "you@example.com",
                        required: true,
                    }
                    FieldDescription { "We send the receipt here." }
                }
                Field {
                    FieldLabel { html_for: "field-required-nickname", "Nickname" }
                    Input { id: "field-required-nickname", placeholder: "Optional" }
                }
            }
        }
    }
}
