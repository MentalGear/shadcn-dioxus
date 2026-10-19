use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use dioxus::prelude::*;
use dioxus_primitives::modality::{FocusTarget, OpenModality};

/// `initial_focus` as a function of the open modality on the non-modal popover: a keyboard open
/// lands on the second field, a mouse open on the first, a touch open on the panel. A keyboard
/// close returns focus to the "Return here" button (`final_focus`), the others to where focus was
/// before the open.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; gap: 0.5rem;",
            PopoverRoot {
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
                    Input { id: "popover-focus-first", name: "first", initial_value: "First" }
                    Input { id: "popover-focus-second", name: "second", initial_value: "Second" }
                }
            }
            Button { id: "popover-focus-return", r#type: "button", variant: ButtonVariant::Outline, "Return here" }
        }
    }
}
