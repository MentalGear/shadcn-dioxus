use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowUp;

/// Block addons: a full-width row below (`BlockEnd`) or above (`BlockStart`) the control, the
/// shape of a chat composer's toolbar.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "BlockEnd: a row below the control" }
                InputGroup {
                    InputGroupInput {
                        id: "input-group-block-end-message",
                        placeholder: "Ask anything",
                        "aria-label": "Message",
                    }
                    InputGroupAddon { align: InputGroupAlign::BlockEnd,
                        InputGroupText { "Enter to send" }
                        Button {
                            style: "margin-inline-start: auto;",
                            size: ButtonSize::IconSm,
                            r#type: "button",
                            "aria-label": "Send message",
                            ArrowUp {}
                        }
                    }
                }
            }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "BlockStart: a row above the control" }
                InputGroup {
                    InputGroupAddon { align: InputGroupAlign::BlockStart,
                        InputGroupText { "Reply to Ada" }
                        Button {
                            style: "margin-inline-start: auto;",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Xs,
                            r#type: "button",
                            "Cancel"
                        }
                    }
                    InputGroupInput {
                        id: "input-group-block-start-reply",
                        placeholder: "Write a reply",
                        "aria-label": "Reply",
                    }
                }
            }
        }
    }
}
