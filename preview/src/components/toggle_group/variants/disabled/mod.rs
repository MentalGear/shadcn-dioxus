use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bold, Italic, Underline};

#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            horizontal: true,
            allow_multiple_pressed: true,
            disabled: true,
            aria_label: "Disabled text formatting",
            ToggleItem { index: 0usize, aria_label: "Disabled bold", Bold {} }
            ToggleItem { index: 1usize, aria_label: "Disabled italic", Italic {} }
            ToggleItem { index: 2usize, aria_label: "Disabled underline", Underline {} }
        }
    }
}
