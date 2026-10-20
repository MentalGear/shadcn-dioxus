use super::super::component::*;
use dioxus::prelude::*;

/// A disabled `Select`: the trigger is inert and dimmed, and the listbox can
/// never open, so the options below are never rendered.
///
/// The placeholder and options deliberately avoid the words the `main`
/// variant's locators key on ("Select an option", "Apple", "Banana"):
/// `select.spec.ts`, `main-thread.spec.ts` and `top-layer-ink.spec.ts` find
/// the single-select trigger by `getByRole("combobox").filter({ hasText:
/// /Select an option|Apple|Banana/ })`, and every variant renders on the same
/// page, so a trigger here reading "Select an option" would make that
/// locator ambiguous.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Select::<String> {
            item_to_label: |value: String| {
                match value.as_str() {
                    "blueberry" => "Blueberry",
                    "grapes" => "Grapes",
                    "mango" => "Mango",
                    other => other,
                }
                .to_string()
            },
            width: "12rem",
            disabled: true,
            placeholder: "Select a fruit",
            trigger_aria_label: "Disabled fruit select",
            list_aria_label: "Disabled fruit options",
            SelectGroup {
                SelectGroupLabel { "Fruits" }
                SelectOption::<String> { index: 0usize, value: "blueberry", text_value: "Blueberry", "Blueberry" }
                SelectOption::<String> { index: 1usize, value: "grapes", text_value: "Grapes", "Grapes" }
                SelectOption::<String> { index: 2usize, value: "mango", text_value: "Mango", "Mango" }
            }
        }
    }
}
