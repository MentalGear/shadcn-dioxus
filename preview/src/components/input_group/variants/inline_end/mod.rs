use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Search;

/// Addons at the inline end of the input: a trailing icon, and trailing text.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupInput {
                    id: "input-group-inline-end-search",
                    placeholder: "Find a component",
                    "aria-label": "Find a component",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    Search {}
                }
            }
            InputGroup {
                InputGroupInput {
                    id: "input-group-inline-end-results",
                    placeholder: "Type to filter",
                    "aria-label": "Type to filter",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupText { "12 results" }
                }
            }
        }
    }
}
