use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldLabel};
use crate::components::toggle::{ToggleSize, ToggleVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

const WEIGHTS: [(&str, &str, &str); 4] = [
    ("light", "Light", "300"),
    ("normal", "Normal", "400"),
    ("medium", "Medium", "500"),
    ("bold", "Bold", "700"),
];

/// shadcn's `toggle-group-font-weight-selector`: a controlled single-select group of 4rem "Aa"
/// tiles (`variant: outline`, `size: lg`, `spacing: 2`) inside a `Field`, with the chosen
/// `font-*` class named underneath.
#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| 1usize);
    let pressed = use_memo(move || Some(HashSet::from([selected()])));
    rsx! {
        Field {
            FieldLabel { html_for: "font-weight-group", id: "font-weight-label", "Font Weight" }
            ToggleGroup {
                id: "font-weight-group",
                aria_labelledby: "font-weight-label",
                variant: ToggleVariant::Outline,
                size: ToggleSize::Lg,
                spacing: 2u8,
                pressed,
                on_pressed_change: move |next: HashSet<usize>| {
                    // Single-select: pressing the pressed tile again leaves it chosen.
                    if let Some(index) = next.into_iter().next() {
                        selected.set(index);
                    }
                },
                for (index , (_ , label , weight)) in WEIGHTS.iter().enumerate() {
                    ToggleItem {
                        key: "{label}",
                        index,
                        aria_label: *label,
                        style: "width: 4rem; height: 4rem; flex-direction: column; border-radius: var(--dx-radius-xl);",
                        span { style: "font-size: var(--dx-text-2xl); font-weight: {weight}; line-height: 1;", "Aa" }
                        // shadcn tints this `text-muted-foreground`; on the pressed tile that is 3.2:1, so it keeps the ink.
                        span { style: "font-size: var(--dx-text-xs);", "{label}" }
                    }
                }
            }
            FieldDescription {
                "Use "
                code { style: "border-radius: var(--dx-radius-md); background: var(--dx-muted); padding: 0.125rem 0.25rem;",
                    "font-{WEIGHTS[selected()].0}"
                }
                " to set the font weight."
            }
        }
    }
}
