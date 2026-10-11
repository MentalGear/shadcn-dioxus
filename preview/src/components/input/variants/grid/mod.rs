use super::super::component::*;
use crate::components::field::{Field, FieldLabel};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem; width: 100%; max-width: 28rem;",
            Field {
                FieldLabel { html_for: "input-grid-first-name", "First name" }
                Input { id: "input-grid-first-name", placeholder: "Max" }
            }
            Field {
                FieldLabel { html_for: "input-grid-last-name", "Last name" }
                Input { id: "input-grid-last-name", placeholder: "Leiter" }
            }
        }
    }
}
