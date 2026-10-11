use super::super::component::*;
use crate::components::button::Button;
use crate::components::field::{Field, FieldDescription, FieldGroup, FieldLabel};
use crate::components::form::DemoForm;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            DemoForm { id: "input-form-signup",
                FieldGroup {
                    Field {
                        FieldLabel { html_for: "input-form-name", "Full name" }
                        Input {
                            id: "input-form-name",
                            name: "name",
                            placeholder: "Evil Rabbit",
                            required: true,
                        }
                    }
                    Field {
                        FieldLabel { html_for: "input-form-email", "Work email" }
                        Input {
                            id: "input-form-email",
                            name: "email",
                            r#type: "email",
                            placeholder: "rabbit@example.com",
                            required: true,
                        }
                        FieldDescription { "We will never share your email with anyone." }
                    }
                    Field {
                        Button { r#type: "submit", "Create account" }
                    }
                }
            }
        }
    }
}
