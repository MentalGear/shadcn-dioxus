use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

/// The whole card is a `FieldLabel` wrapping a `Field`, so clicking anywhere on
/// it toggles the switch; `FieldLabel` draws the card (border, padding, checked
/// tint) itself. The `aria_label` names the switch directly: `Field` is a
/// `role="group"`, and axe does not read a label's name through a group, so the
/// wrapping `<label for>` alone leaves it unnamed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldLabel { html_for: "switch-choice-card-sync",
                Field { orientation: FieldOrientation::Horizontal,
                    FieldContent {
                        "Share across devices"
                        FieldDescription {
                            "Focus is shared across devices, and turns off when you leave the app."
                        }
                    }
                    Switch {
                        id: "switch-choice-card-sync",
                        aria_label: "Share across devices",
                    }
                }
            }
        }
    }
}
