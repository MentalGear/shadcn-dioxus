use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Button { variant: ButtonVariant::Outline,
            "Search"
            KbdGroup {
                Kbd { "⌘" }
                Kbd { "K" }
            }
        }
    }
}
