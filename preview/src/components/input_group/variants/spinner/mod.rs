use super::super::component::*;
use crate::components::spinner::Spinner;
use dioxus::prelude::*;

/// A spinner in the trailing addon while a value is being saved.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupInput {
                    id: "input-group-spinner-name",
                    value: "Acme Studio",
                    "aria-label": "Workspace name",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupText { "Saving..." }
                    Spinner { label: "Saving workspace name" }
                }
            }
        }
    }
}
