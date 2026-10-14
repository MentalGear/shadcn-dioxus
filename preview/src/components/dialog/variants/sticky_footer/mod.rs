use crate::components::button::{Button, ButtonVariant};

use super::super::component::{
    Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use dioxus::prelude::*;

/// shadcn's `dialog-sticky-footer`: a header, a scrolling body and a `DialogFooter` that stays in
/// view while the body scrolls. Only the body has an `overflow`, so the footer is never inside
/// the scroller.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Sticky Footer"
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            DialogHeader {
                DialogTitle { "Sticky Footer" }
                DialogDescription {
                    "This dialog has a sticky footer that stays visible while the content scrolls."
                }
            }
            div {
                role: "region",
                aria_label: "Sticky footer dialog body",
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
            DialogFooter {
                DialogClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}
