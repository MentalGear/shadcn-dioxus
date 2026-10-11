use super::super::component::*;
use crate::components::kbd::{Kbd, KbdGroup};
use dioxus::prelude::*;
use dioxus_icons::lucide::Search;

/// A keyboard-shortcut hint in the trailing addon.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupAddon {
                    Search {}
                }
                InputGroupInput {
                    id: "input-group-kbd-command",
                    placeholder: "Jump to a page",
                    "aria-label": "Jump to a page",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    KbdGroup {
                        Kbd { "⌘" }
                        Kbd { "K" }
                    }
                }
            }
        }
    }
}
