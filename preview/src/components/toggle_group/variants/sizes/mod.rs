use super::super::component::*;
use crate::components::toggle::{ToggleSize, ToggleVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

/// A demo caption: a title line and a one-sentence description.
#[component]
fn Caption(title: &'static str, children: Element) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.125rem;",
            p { style: "margin: 0; color: var(--dx-foreground); font-size: var(--dx-text-sm); font-weight: 500;",
                "{title}"
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                {children}
            }
        }
    }
}

/// shadcn's `toggle-group-sizes`: the same four-item outline group at `sm` (28px) and
/// `default` (32px).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem;",
            Caption { title: "Small", "ToggleSize::Sm, 28px tall." }
            ToggleGroup {
                size: ToggleSize::Sm,
                variant: ToggleVariant::Outline,
                default_value: HashSet::from(["top".to_string()]),
                aria_label: "Small placement",
                ToggleItem { index: 0usize, value: "top", aria_label: "Toggle small top", "Top" }
                ToggleItem { index: 1usize, value: "bottom", aria_label: "Toggle small bottom", "Bottom" }
                ToggleItem { index: 2usize, value: "left", aria_label: "Toggle small left", "Left" }
                ToggleItem { index: 3usize, value: "right", aria_label: "Toggle small right", "Right" }
            }
            Caption { title: "Default", "The default size, 32px tall." }
            ToggleGroup {
                variant: ToggleVariant::Outline,
                default_value: HashSet::from(["top".to_string()]),
                aria_label: "Default placement",
                ToggleItem { index: 0usize, value: "top", aria_label: "Toggle default top", "Top" }
                ToggleItem { index: 1usize, value: "bottom", aria_label: "Toggle default bottom", "Bottom" }
                ToggleItem { index: 2usize, value: "left", aria_label: "Toggle default left", "Left" }
                ToggleItem { index: 3usize, value: "right", aria_label: "Toggle default right", "Right" }
            }
        }
    }
}
