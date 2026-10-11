use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            role: "search",
            style: "display: flex; align-items: center; gap: 0.5rem; width: 100%; max-width: 24rem;",
            Input {
                r#type: "search",
                placeholder: "Search components",
                "aria-label": "Search components",
                style: "flex: 1; min-width: 0;",
            }
            Button { variant: ButtonVariant::Outline, r#type: "button", "Search" }
        }
    }
}
