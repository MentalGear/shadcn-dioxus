use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Italic;

/// shadcn's `toggle-text`: the default variant with an icon and a label.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Toggle { aria_label: "Toggle italic",
            Italic {}
            "Italic"
        }
    }
}
