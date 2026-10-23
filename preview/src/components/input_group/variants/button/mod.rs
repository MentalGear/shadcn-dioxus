use super::super::component::*;
use crate::components::button::ButtonVariant;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Copy, Eye, EyeOff, X};

/// `InputGroupButton` in each size: an icon-only copy button that confirms (`IconXs`), a text button with
/// an icon (`Xs`, the default), a secondary text button (`Sm`) and a password reveal (`IconSm`).
#[component]
pub fn Demo() -> Element {
    let mut copied = use_signal(|| false);
    let mut query = use_signal(|| "dioxus".to_string());
    let mut revealed = use_signal(|| false);
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
                    InputGroupButton {
                        size: InputGroupButtonSize::IconXs,
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
                    id: "input-group-button-filter",
                    value: "{query}",
                    oninput: move |event: FormEvent| query.set(event.value()),
                    "aria-label": "Filter",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupButton { onclick: move |_| query.set(String::new()),
                        X {}
                        "Clear"
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
                    InputGroupButton {
                        variant: ButtonVariant::Secondary,
                        size: InputGroupButtonSize::Sm,
                        "Search"
                    }
                }
            }
            InputGroup {
                InputGroupInput {
                    id: "input-group-button-password",
                    r#type: if revealed() { "text" } else { "password" },
                    value: "correct-horse",
                    "aria-label": "Password",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    InputGroupButton {
                        size: InputGroupButtonSize::IconSm,
                        "aria-label": if revealed() { "Hide password" } else { "Show password" },
                        onclick: move |_| revealed.set(!revealed()),
                        if revealed() {
                            EyeOff {}
                        } else {
                            Eye {}
                        }
                    }
                }
            }
        }
    }
}
