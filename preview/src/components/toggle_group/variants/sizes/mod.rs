use super::super::component::*;
use crate::components::toggle::{ToggleSize, ToggleVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

/// shadcn's `toggle-group-sizes`: the same four-item outline group at `sm` (28px) and
/// `default` (32px).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem;",
            ToggleGroup {
                size: ToggleSize::Sm,
                variant: ToggleVariant::Outline,
                default_pressed: HashSet::from([0usize]),
                aria_label: "Small placement",
                ToggleItem { index: 0usize, aria_label: "Toggle small top", "Top" }
                ToggleItem { index: 1usize, aria_label: "Toggle small bottom", "Bottom" }
                ToggleItem { index: 2usize, aria_label: "Toggle small left", "Left" }
                ToggleItem { index: 3usize, aria_label: "Toggle small right", "Right" }
            }
            ToggleGroup {
                variant: ToggleVariant::Outline,
                default_pressed: HashSet::from([0usize]),
                aria_label: "Default placement",
                ToggleItem { index: 0usize, aria_label: "Toggle default top", "Top" }
                ToggleItem { index: 1usize, aria_label: "Toggle default bottom", "Bottom" }
                ToggleItem { index: 2usize, aria_label: "Toggle default left", "Left" }
                ToggleItem { index: 3usize, aria_label: "Toggle default right", "Right" }
            }
        }
    }
}
