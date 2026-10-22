use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bold, Italic, Strikethrough};

#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            horizontal: true,
            allow_multiple_pressed: true,
            disabled: true,
            aria_label: "Disabled text formatting",
            ToggleItem { index: 0usize, value: "bold", aria_label: "Toggle bold", Bold {} }
            ToggleItem { index: 1usize, value: "italic", aria_label: "Toggle italic", Italic {} }
            ToggleItem { index: 2usize, value: "strikethrough", aria_label: "Toggle strikethrough", Strikethrough {} }
        }
    }
}
