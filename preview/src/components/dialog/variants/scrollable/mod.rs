use crate::components::button::{Button, ButtonVariant};

use super::super::component::{Dialog, DialogDescription, DialogHeader, DialogTitle};
use dioxus::prelude::*;

/// shadcn's `dialog-scrollable-content`: a body taller than the viewport allows scrolls inside
/// the panel (`-mx-4 max-h-[50vh] overflow-y-auto px-4`). The region is a plain `div`, focusable
/// so keyboard users can scroll it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Scrollable Content"
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            DialogHeader {
                DialogTitle { "Scrollable Content" }
                DialogDescription { "This is a dialog with scrollable content." }
            }
            div {
                role: "region",
                aria_label: "Scrollable dialog body",
                tabindex: "0",
                max_height: "50vh",
                overflow_y: "auto",
                style: "margin-inline: calc(var(--dx-space-4) * -1); padding-inline: var(--dx-space-4);",
                for i in 0..10 {
                    p { margin: "0 0 1rem", line_height: "1.5",
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat (paragraph {i + 1} of 10)."
                    }
                }
            }
        }
    }
}
