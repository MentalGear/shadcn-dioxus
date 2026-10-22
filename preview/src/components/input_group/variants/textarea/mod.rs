use super::super::component::*;
use crate::components::button::{Button, ButtonSize};
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowUp;

const LIMIT: usize = 120;

/// shadcn's textarea example: a multi-line control with a footer (a character count and a send
/// button) in a `BlockEnd` addon.
#[component]
pub fn Demo() -> Element {
    let mut text = use_signal(String::new);
    let count = text().chars().count();
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            InputGroup {
                InputGroupTextarea {
                    id: "input-group-textarea-message",
                    placeholder: "Write a message...",
                    "aria-label": "Message",
                    maxlength: "{LIMIT}",
                    oninput: move |e: FormEvent| text.set(e.value()),
                }
                InputGroupAddon { align: InputGroupAlign::BlockEnd,
                    InputGroupText { id: "input-group-textarea-count", "{count} / {LIMIT}" }
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
    }
}
