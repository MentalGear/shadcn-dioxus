use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;

use super::super::component::{Dialog, DialogDescription, DialogHeader, DialogTitle};
use dioxus::prelude::*;
use dioxus_primitives::bridge::{self, Prop, Target};
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` and `final_focus` as functions of the open modality (Base UI's
/// `initialFocus(openType)`): a keyboard open lands on the second field, a mouse open on the
/// Cancel button, a touch open on the panel (no software keyboard) and a pen open names a
/// selector that matches nothing, so the built-in policy decides. A keyboard close returns focus
/// to the "Return here" button instead of the trigger. Both are inputs to the one open-focus
/// policy, not a second code path. The line inside the panel names the modality of the open (read
/// from `<html data-dx-modality>`, the page's one modality state) and where focus was sent: the
/// ring is keyboard-only by design, so a mouse open moves focus to Cancel without painting it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut modality = use_signal(String::new);
    let mut set_open = move |v: bool| {
        open.set(v);
        if v {
            spawn(async move {
                let prop = Prop::Attr("data-dx-modality".into());
                if let Ok(props) = bridge::get(Target::selector("html"), [prop.clone()]).await {
                    if let Some(m) = props.string(&prop) {
                        modality.set(m.to_string());
                    }
                }
            });
        }
    };

    rsx! {
        div { style: "display: flex; gap: 0.5rem;",
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| set_open(true),
                "Open Focus Dialog"
            }
            Button { r#type: "button", id: "dialog-focus-return", variant: ButtonVariant::Outline, "Return here" }
        }
        Dialog {
            open: open(),
            on_open_change: move |v| set_open(v),
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
            p {
                id: "dialog-focus-line",
                style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "Opened with: {modality}, focus on: {focus_label(&modality())}"
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

/// Where `initial_focus` above sends focus for each modality, in words.
fn focus_label(modality: &str) -> &'static str {
    match modality {
        "keyboard" => "the second field",
        "mouse" => "Cancel (no ring: rings are keyboard-only)",
        "touch" => "the panel",
        "pen" => "the panel (the selector matches nothing, so the built-in policy decides)",
        _ => "-",
    }
}
