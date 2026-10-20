use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: the multi-select's chip navigation maps the physical arrow keys through the ambient
/// direction (`use_direction`), so here ArrowRight is "previous" and steps from the input onto the last chip.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { class: "dx-combobox-demo", dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                ComboboxMulti::<String> {
                    item_to_label: |value: String| match value.as_str() {
                        "next" => "Next.js".to_string(),
                        "nuxt" => "Nuxt.js".to_string(),
                        "remix" => "Remix".to_string(),
                        _ => value,
                    },
                    default_values: vec!["next".to_string(), "nuxt".to_string()],
                    style: "--dx-combobox-width: 20rem;",
                    placeholder: "Choose frameworks...",
                    aria_label: "RTL frameworks",
                    list_aria_label: "RTL frameworks list",
                    ComboboxOption::<String> {
                        index: 0usize,
                        value: "next".to_string(),
                        text_value: "Next.js",
                        "Next.js"
                    }
                    ComboboxOption::<String> {
                        index: 1usize,
                        value: "nuxt".to_string(),
                        text_value: "Nuxt.js",
                        "Nuxt.js"
                    }
                    ComboboxOption::<String> {
                        index: 2usize,
                        value: "remix".to_string(),
                        text_value: "Remix",
                        "Remix"
                    }
                }
            }
        }
    }
}
