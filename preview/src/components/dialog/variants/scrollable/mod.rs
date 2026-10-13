use crate::components::button::{Button, ButtonVariant};

use super::super::component::{Dialog, DialogDescription, DialogTitle};
use dioxus::prelude::*;

/// A dialog whose body is longer than the viewport allows: the panel stays put and only the body
/// scrolls. The scrolling region is a plain `div` (`Dialog` has no scroll part); it is focusable so
/// keyboard users can scroll it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Scrollable content"
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
            DialogTitle { "Scrollable content" }
            DialogDescription { "This is a dialog with scrollable content." }
            div {
                role: "region",
                aria_label: "Scrollable dialog body",
                tabindex: "0",
                max_height: "50vh",
                overflow_y: "auto",
                style: "padding-inline-end: 0.5rem;",
                for i in 0..8 {
                    p { margin: "0 0 1rem",
                        "Every dashboard starts as a handful of charts and ends as a wall of them. Group related metrics under one heading, keep the most important number at the top of each group, and let the rest scroll below the fold. Revisit the layout after the first week of real use (paragraph {i + 1} of 8)."
                    }
                }
            }
        }
    }
}
