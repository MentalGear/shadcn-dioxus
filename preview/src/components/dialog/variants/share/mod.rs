use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::label::Label;

use super::super::component::{
    Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use dioxus::prelude::*;

/// shadcn's `dialog-close-button`: the default corner X, plus a `DialogClose` in a start-aligned
/// footer (`<DialogFooter className="sm:justify-start">`).
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Share"
        }
        Dialog {
            open: open(),
            on_open_change: move |v| open.set(v),
            style: "max-width: min(28rem, calc(100% - 2rem));",
            DialogHeader {
                DialogTitle { "Share link" }
                DialogDescription { "Anyone who has this link will be able to view this." }
            }
            div { display: "flex", align_items: "center", gap: "0.5rem",
                div { display: "grid", flex: "1 1 0%", gap: "0.5rem",
                    Label {
                        html_for: "dialog-share-link",
                        style: "position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap;",
                        "Link"
                    }
                    Input {
                        id: "dialog-share-link",
                        readonly: true,
                        initial_value: "https://ui.shadcn.com/docs/installation",
                    }
                }
            }
            DialogFooter { style: "justify-content: flex-start;",
                DialogClose {
                    r#as: |attributes| rsx! {
                        Button { attributes, "Close" }
                    },
                }
            }
        }
    }
}
