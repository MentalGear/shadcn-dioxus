use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                FieldContent {
                    FieldLabel { html_for: "switch-description-sync", "Share across devices" }
                    FieldDescription {
                        "Focus is shared across devices, and turns off when you leave the app."
                    }
                }
                Switch { id: "switch-description-sync" }
            }
        }
    }
}
