use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut query = use_signal(String::new);
    let frameworks: &[(&str, &str)] = &[
        ("next", "Next.js"),
        ("svelte", "SvelteKit"),
        ("nuxt", "Nuxt.js"),
        ("remix", "Remix"),
        ("astro", "Astro"),
        ("solid", "SolidStart"),
        ("dioxus", "Dioxus"),
    ];

    rsx! {
        Combobox::<String> {
            item_to_label: move |value: String| {
                frameworks
                    .iter()
                    .find(|(id, _)| *id == value)
                    .map_or(value.clone(), |(_, label)| label.to_string())
            },
            query: Some(query()),
            on_query_change: move |next| query.set(next),
            placeholder: "Select framework...",
            aria_label: "Select framework",
            list_aria_label: "Frameworks",
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
