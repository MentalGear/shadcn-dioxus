use super::super::component::*;
use dioxus::prelude::*;

const ZONES: [&str; 24] = [
    "UTC-12", "UTC-11", "UTC-10", "UTC-9", "UTC-8", "UTC-7", "UTC-6", "UTC-5", "UTC-4", "UTC-3", "UTC-2", "UTC-1",
    "UTC+0", "UTC+1", "UTC+2", "UTC+3", "UTC+4", "UTC+5", "UTC+6", "UTC+7", "UTC+8", "UTC+9", "UTC+10", "UTC+11",
];

/// `position: SelectPosition::Popper` (shadcn's `position="popper"`): the list opens below the trigger and flips
/// above it when there is no room, whatever the selected option is.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Select::<String> {
            item_to_label: |zone: String| zone,
            default_value: "UTC+5".to_string(),
            position: SelectPosition::Popper,
            width: "12rem",
            trigger_aria_label: "Popper zone select",
            list_aria_label: "Popper zone options",
            SelectGroup {
                for zone in ZONES.iter() {
                    SelectOption::<String> { value: zone.to_string(), text_value: zone.to_string(), "{zone}" }
                }
            }
        }
    }
}
