use super::super::component::*;
use dioxus::prelude::*;

const GROUPS: &[(&str, &[&str])] = &[
    ("Fruits", &["Apple", "Banana", "Blueberry", "Grapes"]),
    ("Vegetables", &["Broccoli", "Carrot", "Courgette", "Leek"]),
    ("Grains", &["Barley", "Oats", "Rice"]),
];

/// Options in labelled groups: each `ComboboxGroup` is a `role="group"` named by its
/// `ComboboxGroupLabel`, the arrow keys run through every group in one order, and a group whose
/// options the query filters out hides with its heading.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { class: "dx-combobox-demo",
            Combobox::<String> {
                item_to_label: |value: String| value,
                placeholder: "Select food...",
                aria_label: "Grouped food",
                list_aria_label: "Grouped food list",
                ComboboxEmpty { "No food found." }
                for (heading , items) in GROUPS.iter() {
                    ComboboxGroup { key: "{heading}",
                        ComboboxGroupLabel { "{heading}" }
                        for item in items.iter() {
                            ComboboxOption::<String> { value: item.to_string(), text_value: *item, {*item} }
                        }
                    }
                }
            }
        }
    }
}
