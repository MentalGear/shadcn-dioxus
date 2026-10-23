use super::super::component::*;
use crate::components::toggle::ToggleVariant;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bold, Italic, Strikethrough};

/// shadcn's `toggle-group-demo`: an outline, multiple-select group of three icon items.
/// (shadcn labels the third item "Toggle strikethrough" but draws an underline icon; this
/// one draws the strikethrough it names.)
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            variant: ToggleVariant::Outline,
            allow_multiple_pressed: true,
            aria_label: "Text formatting",
            ToggleItem { index: 0usize, value: "bold", aria_label: "Toggle bold", Bold {} }
            ToggleItem { index: 1usize, value: "italic", aria_label: "Toggle italic", Italic {} }
            ToggleItem { index: 2usize, value: "strikethrough", aria_label: "Toggle strikethrough", Strikethrough {} }
        }
    }
}
