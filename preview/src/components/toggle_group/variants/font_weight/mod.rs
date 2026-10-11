use super::super::component::*;
use dioxus::prelude::*;
use std::collections::HashSet;

/// A single-select group (the default `allow_multiple_pressed: false`) where
/// each item previews the weight it names.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToggleGroup {
            horizontal: true,
            default_pressed: HashSet::from([1usize]),
            aria_label: "Font weight",
            ToggleItem { index: 0usize, style: "font-weight: 300;", "Light" }
            ToggleItem { index: 1usize, style: "font-weight: 400;", "Normal" }
            ToggleItem { index: 2usize, style: "font-weight: 500;", "Medium" }
            ToggleItem { index: 3usize, style: "font-weight: 700;", "Bold" }
        }
    }
}
