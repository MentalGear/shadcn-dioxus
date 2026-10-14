use super::super::component::*;
use crate::components::toggle::ToggleVariant;
use dioxus::prelude::*;
use std::collections::HashSet;

/// shadcn's `toggle-group-outline`: a single-select outline group, "All" pressed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            variant: ToggleVariant::Outline,
            default_pressed: HashSet::from([0usize]),
            aria_label: "Message filter",
            ToggleItem { index: 0usize, aria_label: "Toggle all", "All" }
            ToggleItem { index: 1usize, aria_label: "Toggle missed", "Missed" }
        }
    }
}
