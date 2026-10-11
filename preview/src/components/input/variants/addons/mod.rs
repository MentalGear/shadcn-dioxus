use crate::components::button::{Button, ButtonVariant};
use crate::components::button_group::ButtonGroup;
use crate::components::input::Input;
use crate::components::input_group::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Search;

/// Addon examples built from `InputGroup`: a search field with a leading icon
/// and a domain field with text addons on both ends. The last example joins a
/// bare `Input` to a `Button` inside a `ButtonGroup`; the input takes the
/// group's seam through the `--dx-group-radius-*` variables (see
/// `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 24rem;",
            InputGroup {
                InputGroupAddon {
                    Search {}
                }
                InputGroupInput {
                    id: "input-addons-search",
                    "aria-label": "Search the docs",
                    placeholder: "Search the docs",
                }
            }
            InputGroup {
                InputGroupAddon { InputGroupText { "https://" } }
                InputGroupInput {
                    id: "input-addons-domain",
                    "aria-label": "Website domain",
                    placeholder: "example.com",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupText { ".com" }
                }
            }
            ButtonGroup { "aria-label": "Newsletter signup",
                Input {
                    id: "input-addons-subscribe",
                    "aria-label": "Email address",
                    placeholder: "you@example.com",
                    r#type: "email",
                    style: "flex: 1;",
                }
                Button { variant: ButtonVariant::Outline, "Subscribe" }
            }
        }
    }
}
