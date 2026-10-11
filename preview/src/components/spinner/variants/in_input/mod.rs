use super::super::component::*;
use crate::components::input_group::{
    InputGroup, InputGroupAddon, InputGroupAlign, InputGroupInput, InputGroupText,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupInput { placeholder: "Send a message...", disabled: true }
                InputGroupAddon { align: InputGroupAlign::End,
                    Spinner { label: "Sending" }
                    InputGroupText { "Sending..." }
                }
            }
        }
    }
}
