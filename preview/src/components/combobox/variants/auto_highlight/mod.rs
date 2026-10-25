use super::super::component::*;
use dioxus::prelude::*;

/// `auto_highlight`: typing highlights the first option that matches, so Enter picks it straight
/// away.
#[component]
pub fn Demo() -> Element {
    let languages = ["Rust", "Ruby", "Python", "TypeScript", "Go", "Kotlin", "Swift"];

    rsx! {
        div { class: "dx-combobox-demo",
            Combobox::<String> {
                item_to_label: |value: String| value,
                auto_highlight: true,
                placeholder: "Select language...",
                aria_label: "Auto-highlight language",
                list_aria_label: "Auto-highlight language list",
                ComboboxEmpty { "No language found." }
                for language in languages {
                    ComboboxOption::<String> { value: language.to_string(), text_value: language, {language} }
                }
            }
        }
    }
}
