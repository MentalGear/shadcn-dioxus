use super::super::component::*;
use crate::components::input_group::{InputGroup, InputGroupAddon, InputGroupInput};
use dioxus::prelude::*;
use crate::components::icons::Search;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Empty { style: "width: 100%; max-width: 28rem;",
            EmptyHeader {
                EmptyTitle { "404 - Page not found" }
                EmptyDescription {
                    "The page you're looking for doesn't exist. Try searching for what you need below."
                }
            }
            EmptyContent {
                InputGroup { style: "width: 100%; max-width: 20rem;",
                    InputGroupAddon {
                        Search {}
                    }
                    InputGroupInput {
                        placeholder: "Search pages...",
                        aria_label: "Search pages",
                    }
                }
            }
        }
    }
}
