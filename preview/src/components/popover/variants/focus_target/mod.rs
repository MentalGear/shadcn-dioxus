use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use dioxus::prelude::*;
use dioxus_primitives::bridge::{self, Prop, Target};
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` as a function of the open modality on the non-modal popover: a keyboard open
/// lands on the second field, a mouse open on the first, a touch open on the panel. A keyboard
/// close returns focus to the "Return here" button (`final_focus`), the others to where focus was
/// before the open. The line inside the panel names the modality of the open (read from
/// `<html data-dx-modality>`) and where focus went.
#[component]
pub fn Demo() -> Element {
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
        div { style: "display: flex; gap: 0.5rem;",
            PopoverRoot {
                on_open_change: move |v| {
                    if v {
                        read_modality();
                    }
                },
                PopoverTrigger { "Open Focus Popover" }
                PopoverContent {
                    initial_focus: Callback::new(|m: OpenModality| match m {
                        OpenModality::Keyboard => FocusTarget::selector("#popover-focus-second"),
                        OpenModality::Mouse => FocusTarget::selector("#popover-focus-first"),
                        OpenModality::Touch | OpenModality::Pen => FocusTarget::Panel,
                    }),
                    final_focus: Callback::new(|m: OpenModality| match m {
                        OpenModality::Keyboard => FocusTarget::selector("#popover-focus-return"),
                        _ => FocusTarget::Default,
                    }),
                    PopoverHeader {
                        PopoverTitle { "Focus by modality" }
                        PopoverDescription { "Where focus lands depends on how this opened." }
                    }
                    p {
                        id: "popover-focus-line",
                        style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                        "Opened with: {modality}, focus on: {focus_label(&modality())}"
                    }
                    Input { id: "popover-focus-first", name: "first", initial_value: "First" }
                    Input { id: "popover-focus-second", name: "second", initial_value: "Second" }
                }
            }
            Button { id: "popover-focus-return", r#type: "button", variant: ButtonVariant::Outline, "Return here" }
        }
    }
}

/// Where `initial_focus` above sends focus for each modality, in words.
fn focus_label(modality: &str) -> &'static str {
    match modality {
        "keyboard" => "the second field",
        "mouse" => "the first field",
        "touch" | "pen" => "the panel",
        _ => "-",
    }
}
