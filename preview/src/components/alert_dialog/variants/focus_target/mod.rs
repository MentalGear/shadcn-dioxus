use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::bridge::{self, Prop, Target};
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` on an alert dialog: the destructive action is never the default focus. A
/// keyboard open lands on Cancel (the safe choice) and a mouse open on the panel; touch and pen
/// open on the panel as always. The line inside the panel names the modality of the open (read from
/// `<html data-dx-modality>`) and where focus went: rings are keyboard-only, so a mouse open
/// focuses the panel without painting anything.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut modality = use_signal(String::new);
    let read_modality = move || {
        spawn(async move {
            let prop = Prop::Attr("data-dx-modality".into());
            if let Ok(props) = bridge::get(Target::selector("html"), [prop.clone()]).await {
                if let Some(m) = props.string(&prop) {
                    modality.set(m.to_string());
                }
            }
        });
    };

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                read_modality();
                open.set(true);
            },
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
            p {
                id: "alert-focus-line",
                style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "Opened with: {modality}, focus on: {focus_label(&modality())}"
            }
            AlertDialogActions {
                AlertDialogAction { "Discard" }
                AlertDialogCancel { id: "alert-focus-cancel", "Cancel" }
            }
        }
    }
}

/// Where `initial_focus` above sends focus for each modality, in words.
fn focus_label(modality: &str) -> &'static str {
    match modality {
        "keyboard" => "Cancel (ring shown: rings follow the keyboard)",
        "mouse" | "touch" | "pen" => "the panel (no ring)",
        _ => "-",
    }
}
