use super::super::component::*;
use crate::components::input::Input;
use dioxus::prelude::*;

/// shadcn's "Text" example: a `ButtonGroupText` chip flanking an input, once with a unit before it and once
/// with a suffix after it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 24rem;",
            ButtonGroup { "aria-label": "Price", style: "width: 100%;",
                ButtonGroupText { "$" }
                Input {
                    id: "button-group-text-amount",
                    placeholder: "0.00",
                    "aria-label": "Amount",
                    style: "flex: 1;",
                }
                ButtonGroupText { "USD" }
            }
            ButtonGroup { "aria-label": "Website", style: "width: 100%;",
                ButtonGroupText { "https://" }
                Input {
                    id: "button-group-text-domain",
                    placeholder: "example",
                    "aria-label": "Domain",
                    style: "flex: 1;",
                }
                ButtonGroupText { ".com" }
            }
        }
    }
}
