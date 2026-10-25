use super::super::component::*;
use crate::components::form::DemoForm;
use dioxus::prelude::*;

/// A single-select combobox in a form: `name` submits the selected value (`item_to_value`, here the
/// value itself rather than its label), `show_clear` draws a button that clears it, and the form's
/// reset restores the default.
#[component]
pub fn Demo() -> Element {
    let frameworks: &'static [(&str, &str)] = &[
        ("next", "Next.js"),
        ("svelte", "SvelteKit"),
        ("nuxt", "Nuxt.js"),
        ("remix", "Remix"),
        ("astro", "Astro"),
    ];

    rsx! {
        div { class: "dx-combobox-demo",
            DemoForm { id: "combobox-single-form", style: "display: grid; gap: 0.75rem;",
                Combobox::<String> {
                    item_to_label: move |value: String| {
                        frameworks.iter().find(|(v, _)| *v == value).map_or(value.clone(), |(_, label)| label.to_string())
                    },
                    item_to_value: move |value: String| value,
                    name: Some("framework".to_string()),
                    default_value: Some("astro".to_string()),
                    show_clear: true,
                    placeholder: "Select framework...",
                    aria_label: "Form framework",
                    list_aria_label: "Form framework list",
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
