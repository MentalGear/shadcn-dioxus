use crate::components::button::{Button, ButtonVariant};

use super::super::component::{Dialog, DialogDescription, DialogTitle};
use dioxus::prelude::*;

/// A header, a scrolling body and a footer that stays in view while the body scrolls. The body and
/// the footer are plain `div`s (`Dialog` has no header/footer parts): only the body has an
/// `overflow`, so the footer is never inside the scroller.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Sticky footer"
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            button {
                class: "dx-dialog-close",
                r#type: "button",
                aria_label: "Close",
                tabindex: if open() { "0" } else { "-1" },
                onclick: move |_| open.set(false),
                "×"
            }
            DialogTitle { "Terms of service" }
            DialogDescription { "Scroll to read the terms. The buttons stay in view." }
            div {
                role: "region",
                aria_label: "Terms of service text",
                tabindex: "0",
                max_height: "50vh",
                overflow_y: "auto",
                style: "padding-inline-end: 0.5rem;",
                for i in 0..8 {
                    p { margin: "0 0 1rem",
                        "By using the service you agree to keep your account details accurate, to keep your credentials private, and to tell us promptly about any unauthorised use. We may update these terms; we will give notice before a change takes effect (clause {i + 1} of 8)."
                    }
                }
            }
            div { display: "flex", justify_content: "flex-end", gap: "0.5rem",
                Button {
                    r#type: "button",
                    variant: ButtonVariant::Outline,
                    onclick: move |_| open.set(false),
                    "Decline"
                }
                Button { r#type: "button", onclick: move |_| open.set(false), "Accept" }
            }
        }
    }
}
