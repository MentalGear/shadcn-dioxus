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
            default_value: HashSet::from(["all".to_string()]),
            aria_label: "Message filter",
            ToggleItem { index: 0usize, value: "all", aria_label: "Toggle all", "All" }
            ToggleItem { index: 1usize, value: "missed", aria_label: "Toggle missed", "Missed" }
        }
    }
}
