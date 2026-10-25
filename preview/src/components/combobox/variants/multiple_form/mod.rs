use super::super::component::*;
use crate::components::form::DemoForm;
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
            DemoForm { id: "combobox-multiple-form", style: "display: grid; gap: 0.75rem;",
                ComboboxMulti::<String> {
                    item_to_label: move |value: String| {
                        frameworks.iter().find(|(v, _)| *v == value).map_or(value.clone(), |(_, label)| label.to_string())
                    },
                    name: Some("frameworks".to_string()),
                    default_values: vec!["next".to_string()],
                    style: "--dx-combobox-width: 20rem;",
                    placeholder: "Choose frameworks...",
                    aria_label: "Form frameworks",
                    list_aria_label: "Form frameworks list",
                    ComboboxEmpty { "No framework found." }
                    for (value , label) in frameworks.iter() {
                        ComboboxOption::<String> {
                            value: value.to_string(),
                            text_value: label.to_string(),
                            {*label}
                        }
                    }
                }
                button { r#type: "submit", "Submit" }
                button { r#type: "reset", "Reset" }
            }
        }
    }
}
