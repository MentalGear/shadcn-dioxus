use super::super::component::*;
use crate::components::toggle::{ToggleSize, ToggleVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

/// shadcn's `toggle-group-spacing`: `spacing: 2` keeps the items as separate toggles 8px apart
/// (the default); the second group is the joined bar (`spacing: 0`) for comparison.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem;",
            ToggleGroup {
                size: ToggleSize::Sm,
                variant: ToggleVariant::Outline,
                spacing: 2u8,
                default_value: HashSet::from(["top".to_string()]),
                aria_label: "Spaced placement",
                ToggleItem { index: 0usize, value: "top", aria_label: "Toggle spaced top", "Top" }
                ToggleItem { index: 1usize, value: "bottom", aria_label: "Toggle spaced bottom", "Bottom" }
                ToggleItem { index: 2usize, value: "left", aria_label: "Toggle spaced left", "Left" }
                ToggleItem { index: 3usize, value: "right", aria_label: "Toggle spaced right", "Right" }
            }
            ToggleGroup {
                size: ToggleSize::Sm,
                variant: ToggleVariant::Outline,
                spacing: 0u8,
                default_value: HashSet::from(["top".to_string()]),
                aria_label: "Joined placement",
                ToggleItem { index: 0usize, value: "top", aria_label: "Toggle joined top", "Top" }
                ToggleItem { index: 1usize, value: "bottom", aria_label: "Toggle joined bottom", "Bottom" }
                ToggleItem { index: 2usize, value: "left", aria_label: "Toggle joined left", "Left" }
                ToggleItem { index: 3usize, value: "right", aria_label: "Toggle joined right", "Right" }
            }
        }
    }
}
