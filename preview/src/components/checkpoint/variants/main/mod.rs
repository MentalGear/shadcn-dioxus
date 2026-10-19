use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut last = use_signal(|| "Nothing restored yet");
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-4);",
            Checkpoint {
                label: "Before refactor",
                on_restore: move |_| last.set("Restored: Before refactor"),
            }
            Checkpoint {
                label: "Tests passing",
                on_restore: move |_| last.set("Restored: Tests passing"),
            }
            p { role: "status", style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "{last}"
            }
        }
    }
}
