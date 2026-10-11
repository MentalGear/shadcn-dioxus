use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Copy};

/// Buttons inside addons: an icon-only copy button that confirms, and a text button.
#[component]
pub fn Demo() -> Element {
    let mut copied = use_signal(|| false);
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupInput {
                    id: "input-group-button-link",
                    readonly: true,
                    value: "https://ui.shadcn.com/docs",
                    "aria-label": "Shareable link",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconXs,
                        r#type: "button",
                        "aria-label": if copied() { "Link copied" } else { "Copy link" },
                        onclick: move |_| copied.set(!copied()),
                        if copied() {
                            Check {}
                        } else {
                            Copy {}
                        }
                    }
                }
            }
            InputGroup {
                InputGroupInput {
                    id: "input-group-button-query",
                    placeholder: "Search the docs",
                    "aria-label": "Search the docs",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        r#type: "button",
                        "Search"
                    }
                }
            }
        }
    }
}
