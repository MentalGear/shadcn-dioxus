use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Code;

/// shadcn's "custom input": the group's control is any element that wears `dx-input-group-control`,
/// here a plain `textarea` the caller styles. The group still draws the edge, the focus ring and the
/// invalid and disabled states, and the addons sit around it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 24rem;",
            InputGroup {
                InputGroupAddon { align: InputGroupAlign::BlockStart,
                    Code {}
                    InputGroupText { "script.js" }
                }
                textarea {
                    id: "input-group-custom-code",
                    class: "dx-input-group-control",
                    style: "min-height: 5rem; padding: 0.5rem 0.75rem; border: 0; outline: 0; resize: none; font: inherit; font-family: var(--dx-font-mono, monospace); font-size: var(--dx-text-sm); color: var(--dx-foreground); background: none;",
                    "aria-label": "Code",
                    placeholder: "console.log('hello');",
                }
            }
            InputGroup {
                textarea {
                    id: "input-group-custom-invalid",
                    class: "dx-input-group-control",
                    style: "min-height: 3rem; padding: 0.5rem 0.75rem; border: 0; outline: 0; resize: none; font: inherit; font-size: var(--dx-text-sm); color: var(--dx-foreground); background: none;",
                    "aria-label": "Notes",
                    "aria-invalid": "true",
                    "Required",
                }
            }
        }
    }
}
