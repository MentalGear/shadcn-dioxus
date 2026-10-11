use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Toggle { disabled: true, style: "padding-inline: 0.75rem;", "Disabled" }
    }
}
