use super::super::component::*;
use crate::components::checkbox::Checkbox;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.75rem;",
            Checkbox { id: "label-checkbox-terms", name: "label-checkbox-terms" }
            Label { html_for: "label-checkbox-terms", "Accept terms and conditions" }
        }
    }
}
