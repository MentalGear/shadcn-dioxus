use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bold, Italic, Underline};
use std::collections::HashSet;

/// shadcn's `toggle-group-vertical`: a multiple-select column, `spacing: 1`, bold and italic pressed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            horizontal: false,
            allow_multiple_pressed: true,
            spacing: 1u8,
            default_value: HashSet::from(["bold".to_string(), "italic".to_string()]),
            aria_label: "Vertical text formatting",
            ToggleItem { index: 0usize, value: "bold", aria_label: "Toggle vertical bold", Bold {} }
            ToggleItem { index: 1usize, value: "italic", aria_label: "Toggle vertical italic", Italic {} }
            ToggleItem { index: 2usize, value: "underline", aria_label: "Toggle vertical underline", Underline {} }
        }
    }
}
