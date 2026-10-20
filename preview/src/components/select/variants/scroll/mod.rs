use super::super::component::*;
use dioxus::prelude::*;

const ZONES: [&str; 24] = [
    "UTC-12", "UTC-11", "UTC-10", "UTC-9", "UTC-8", "UTC-7", "UTC-6", "UTC-5", "UTC-4", "UTC-3", "UTC-2", "UTC-1",
    "UTC+0", "UTC+1", "UTC+2", "UTC+3", "UTC+4", "UTC+5", "UTC+6", "UTC+7", "UTC+8", "UTC+9", "UTC+10", "UTC+11",
];

/// A list taller than the popup: it scrolls inside it, and a scroll button appears at the top or bottom edge
/// while there is more in that direction (hover it to auto-scroll). The placeholder and options avoid the words
/// the `main` variant's locators key on (see the `disabled` variant's note).
#[component]
pub fn Demo() -> Element {
    rsx! {
        Select::<String> {
            item_to_label: |zone: String| zone,
            width: "12rem",
            placeholder: "Pick a time zone",
            trigger_aria_label: "Time zone select",
            list_aria_label: "Time zone options",
            SelectGroup {
                for (i , zone) in ZONES.iter().enumerate() {
                    SelectOption::<String> { index: i, value: zone.to_string(), text_value: zone.to_string(), "{zone}" }
                }
            }
        }
    }
}
