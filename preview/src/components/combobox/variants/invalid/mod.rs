use super::super::component::*;
use crate::components::field::{Field, FieldError, FieldLabel};
use dioxus::prelude::*;

/// `invalid` marks the input `aria-invalid="true"` and draws the invalid ring; `Field { invalid }`
/// turns the label red and shows the error.
#[component]
pub fn Demo() -> Element {
    let mut value = use_signal(|| None::<String>);
    let invalid = use_memo(move || value().is_none());

    rsx! {
        div { class: "dx-combobox-demo",
            Field { invalid: invalid(),
                FieldLabel { html_for: "combobox-invalid-country", "Country" }
                Combobox::<String> {
                    item_to_label: |value: String| value,
                    on_value_change: move |next| value.set(next),
                    invalid,
                    placeholder: "Select country...",
                    input_id: "combobox-invalid-country",
                    list_aria_label: "Required country list",
                    ComboboxEmpty { "No country found." }
                    for country in ["Argentina", "Canada", "Japan", "Kenya", "Norway"] {
                        ComboboxOption::<String> { value: country.to_string(), text_value: country, {country} }
                    }
                }
                if invalid() {
                    FieldError { "Pick a country to continue." }
                }
            }
        }
    }
}
