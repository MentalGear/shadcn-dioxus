use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` on an alert dialog: the destructive action is never the default focus. A
/// keyboard open lands on Cancel (the safe choice) and a mouse open on the panel; touch and pen
/// open on the panel as always.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| open.set(true),
            "Show Focus Alert"
        }
        AlertDialog {
            open: open(),
            on_open_change: move |v| open.set(v),
            initial_focus: Callback::new(|m: OpenModality| match m {
                OpenModality::Keyboard => FocusTarget::selector("#alert-focus-cancel"),
                _ => FocusTarget::Panel,
            }),
            AlertDialogTitle { "Discard draft" }
            AlertDialogDescription { "Focus starts on the safe choice." }
            AlertDialogActions {
                AlertDialogAction { "Discard" }
                AlertDialogCancel { id: "alert-focus-cancel", "Cancel" }
            }
        }
    }
}
