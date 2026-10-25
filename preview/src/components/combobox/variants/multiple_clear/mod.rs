use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let frameworks: &'static [(&str, &str)] = &[
        ("next", "Next.js"),
        ("svelte", "SvelteKit"),
        ("nuxt", "Nuxt.js"),
        ("remix", "Remix"),
        ("astro", "Astro"),
        ("solid", "SolidStart"),
        ("dioxus", "Dioxus"),
    ];

    rsx! {
        div { class: "dx-combobox-demo",
            ComboboxMulti::<String> {
                item_to_label: move |value: String| {
                    frameworks.iter().find(|(v, _)| *v == value).map_or(value.clone(), |(_, label)| label.to_string())
                },
                default_values: vec!["svelte".to_string(), "remix".to_string(), "dioxus".to_string()],
                show_clear: true,
                style: "--dx-combobox-width: 20rem;",
                placeholder: "Choose frameworks...",
                aria_label: "Frameworks with clear",
                list_aria_label: "Frameworks to clear",
                ComboboxEmpty { "No framework found." }
                for (value , label) in frameworks.iter() {
                    ComboboxOption::<String> {
                        value: value.to_string(),
                        text_value: label.to_string(),
                        {*label}
                    }
                }
            }
        }
    }
}
