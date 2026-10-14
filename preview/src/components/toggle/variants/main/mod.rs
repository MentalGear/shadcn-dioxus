use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Bookmark;

/// shadcn's `toggle-demo`: an outline, small Bookmark toggle named "Toggle bookmark".
/// The icon fills with the ink while the toggle is pressed (`group-aria-pressed/toggle:fill-foreground`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        Toggle {
            aria_label: "Toggle bookmark",
            size: ToggleSize::Sm,
            variant: ToggleVariant::Outline,
            Bookmark { "data-pressed-fill": "true" }
            "Bookmark"
        }
    }
}
