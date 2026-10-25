use super::super::component::*;
use dioxus::prelude::*;

fn fruit_label(value: String) -> String {
    match value.as_str() {
        "blueberry" => "Blueberry",
        "grapes" => "Grapes",
        other => other,
    }
    .to_string()
}

/// The trigger `size` prop: `Sm` (28px) beside `Default` (32px). Placeholders avoid the words the
/// `main` variant's locators key on (see the `disabled` variant's note).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 12rem;",
            Select::<String> {
                item_to_label: fruit_label,
                size: SelectSize::Sm,
                placeholder: "Small size",
                trigger_aria_label: "Small size select",
                list_aria_label: "Small size options",
                SelectGroup {
                    SelectOption::<String> { value: "blueberry", text_value: "Blueberry", "Blueberry" }
                    SelectOption::<String> { value: "grapes", text_value: "Grapes", "Grapes" }
                }
            }
            Select::<String> {
                item_to_label: fruit_label,
                size: SelectSize::Default,
                placeholder: "Default size",
                trigger_aria_label: "Default size select",
                list_aria_label: "Default size options",
                SelectGroup {
                    SelectOption::<String> { value: "blueberry", text_value: "Blueberry", "Blueberry" }
                    SelectOption::<String> { value: "grapes", text_value: "Grapes", "Grapes" }
                }
            }
        }
    }
}
