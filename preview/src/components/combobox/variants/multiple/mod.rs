use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| vec!["next".to_string(), "astro".to_string()]);
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
        div { class: "dx-combobox-demo", style: "display: grid; gap: 0.75rem;",
            div { "data-testid": "combobox-multiple-value", "{selected().join(\",\")}" }
            ComboboxMulti::<String> {
                values: Some(selected()),
                on_values_change: move |next| selected.set(next),
                style: "--dx-combobox-width: 20rem;",
                placeholder: "Choose frameworks...",
                aria_label: "Choose frameworks",
                list_aria_label: "Frameworks",
                ComboboxEmpty { "No framework found." }
                for (i , (value , label)) in frameworks.iter().enumerate() {
                    ComboboxOption::<String> {
                        index: i,
                        value: value.to_string(),
                        text_value: label.to_string(),
                        {*label}
                    }
                }
            }
        }
    }
}
