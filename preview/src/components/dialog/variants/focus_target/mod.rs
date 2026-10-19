use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;

use super::super::component::{Dialog, DialogDescription, DialogHeader, DialogTitle};
use dioxus::prelude::*;
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` and `final_focus` as functions of the open modality (Base UI's
/// `initialFocus(openType)`): a keyboard open lands on the second field, a mouse open on the
/// Cancel button, a touch open on the panel (no software keyboard) and a pen open names a
/// selector that matches nothing, so the built-in policy decides. A keyboard close returns focus
/// to the "Return here" button instead of the trigger. Both are inputs to the one open-focus
/// policy, not a second code path.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        div { style: "display: flex; gap: 0.5rem;",
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| open.set(true),
                "Open Focus Dialog"
            }
            Button { r#type: "button", id: "dialog-focus-return", variant: ButtonVariant::Outline, "Return here" }
        }
        Dialog {
            open: open(),
            on_open_change: move |v| open.set(v),
            show_close_button: false,
            initial_focus: Callback::new(|m: OpenModality| match m {
                OpenModality::Keyboard => FocusTarget::selector("#dialog-focus-second"),
                OpenModality::Mouse => FocusTarget::selector("#dialog-focus-cancel"),
                OpenModality::Touch => FocusTarget::Panel,
                OpenModality::Pen => FocusTarget::selector("#dialog-focus-missing"),
            }),
            final_focus: Callback::new(|m: OpenModality| match m {
                OpenModality::Keyboard => FocusTarget::selector("#dialog-focus-return"),
                _ => FocusTarget::Default,
            }),
            DialogHeader {
                DialogTitle { "Focus by modality" }
                DialogDescription { "Where focus lands depends on how this dialog was opened." }
            }
            Input { id: "dialog-focus-first", name: "first", initial_value: "First" }
            Input { id: "dialog-focus-second", name: "second", initial_value: "Second" }
            Button {
                r#type: "button",
                id: "dialog-focus-cancel",
                variant: ButtonVariant::Outline,
                onclick: move |_| open.set(false),
                "Cancel"
            }
        }
    }
}
