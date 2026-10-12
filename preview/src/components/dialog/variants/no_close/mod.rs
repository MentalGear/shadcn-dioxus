use crate::components::button::{Button, ButtonVariant};

use super::super::component::{Dialog, DialogDescription, DialogTitle};
use dioxus::prelude::*;

/// A dialog without the corner close button. The X is an opt-in child of `Dialog` (the `main`
/// variant writes it by hand), so leaving it out is the whole variant; Escape and a click on the
/// scrim still dismiss it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "No close button"
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            DialogTitle { "No close button" }
            DialogDescription {
                "This dialog has no close button in the top-right corner. Press Escape or click outside to dismiss it."
            }
        }
    }
}
