use super::super::component::*;
use dioxus::prelude::*;

/// Static text addons: a prefix, a suffix, and both around one input.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupAddon { InputGroupText { "+1" } }
                InputGroupInput {
                    id: "input-group-text-phone",
                    r#type: "tel",
                    placeholder: "(555) 000-0000",
                    "aria-label": "Phone number",
                }
            }
            InputGroup {
                InputGroupInput {
                    id: "input-group-text-price",
                    placeholder: "0.00",
                    "aria-label": "Price",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupText { "USD" }
                }
            }
            InputGroup {
                InputGroupAddon { InputGroupText { "https://" } }
                InputGroupInput {
                    id: "input-group-text-domain",
                    placeholder: "example",
                    "aria-label": "Domain name",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupText { ".com" }
                }
            }
        }
    }
}
