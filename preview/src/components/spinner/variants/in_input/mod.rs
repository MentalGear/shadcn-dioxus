use super::super::component::*;
use crate::components::input_group::{
    InputGroup, InputGroupAddon, InputGroupAlign, InputGroupInput, InputGroupText,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            // The group dims to the disabled opacity (`:has(:disabled)`); `aria-disabled` tells assistive tech
            // and axe's colour-contrast rule that the dimmed addon text belongs to an inactive control.
            InputGroup { "aria-disabled": "true",
                InputGroupInput { placeholder: "Send a message...", disabled: true }
                InputGroupAddon { align: InputGroupAlign::End,
                    Spinner { label: "Sending" }
                    InputGroupText { "Sending..." }
                }
            }
        }
    }
}
