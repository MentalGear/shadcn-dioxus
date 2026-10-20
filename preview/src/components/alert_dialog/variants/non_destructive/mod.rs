use crate::components::button::{Button, ButtonVariant};

use super::super::component::*;
use dioxus::prelude::*;

/// `variant: AlertDialogActionVariant::Primary` is the confirm button of an ordinary prompt, so a
/// dialog that is not destructive does not paint its action red.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut published = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Publish changes"
        }
        AlertDialog { open: open(), on_open_change: move |v| open.set(v),
            AlertDialogTitle { "Publish this page?" }
            AlertDialogDescription {
                "Your changes go live for everyone straight away. You can unpublish at any time."
            }
            AlertDialogActions {
                AlertDialogCancel { "Not yet" }
                AlertDialogAction {
                    variant: AlertDialogActionVariant::Primary,
                    on_click: move |_| published.set(true),
                    "Publish"
                }
            }
        }
        if published() {
            p { style: "margin-top: 1rem; font-weight: 600;", "Page published." }
        }
    }
}
