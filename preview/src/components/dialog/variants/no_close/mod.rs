use crate::components::button::{Button, ButtonVariant};

use super::super::component::{Dialog, DialogDescription, DialogHeader, DialogTitle};
use dioxus::prelude::*;

/// shadcn's `dialog-no-close-button`: `<DialogContent showCloseButton={false}>`. Escape and a
/// click on the scrim still dismiss it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "No Close Button"
        }
        Dialog {
            open: open(),
            on_open_change: move |v| open.set(v),
            show_close_button: false,
            DialogHeader {
                DialogTitle { "No Close Button" }
                DialogDescription {
                    "This dialog doesn't have a close button in the top-right corner. Press Escape or click outside to dismiss it."
                }
            }
        }
    }
}
