use super::super::component::*;
use dioxus::prelude::*;

const ZONES: [&str; 24] = [
    "UTC-12", "UTC-11", "UTC-10", "UTC-9", "UTC-8", "UTC-7", "UTC-6", "UTC-5", "UTC-4", "UTC-3", "UTC-2", "UTC-1",
    "UTC+0", "UTC+1", "UTC+2", "UTC+3", "UTC+4", "UTC+5", "UTC+6", "UTC+7", "UTC+8", "UTC+9", "UTC+10", "UTC+11",
];

/// `position: SelectPosition::ItemAligned` (shadcn's `position="item-aligned"`): the list opens over the trigger
/// with the selected option on the trigger's line, shifted and scrolled to stay inside the viewport. The second
/// select has no value, so its whole list opens below the trigger and flips above it when there is no room.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 12rem;",
            Select::<String> {
                item_to_label: |zone: String| zone,
                default_value: "UTC+5".to_string(),
                position: SelectPosition::ItemAligned,
                width: "12rem",
                trigger_aria_label: "Aligned zone select",
                list_aria_label: "Aligned zone options",
                SelectGroup {
                    for zone in ZONES.iter() {
                        SelectOption::<String> { value: zone.to_string(), text_value: zone.to_string(), "{zone}" }
                    }
                }
            }
            Select::<String> {
                item_to_label: |zone: String| zone,
                placeholder: "Pick a zone",
                position: SelectPosition::ItemAligned,
                width: "12rem",
                trigger_aria_label: "Placeholder zone select",
                list_aria_label: "Placeholder zone options",
                SelectGroup {
                    for zone in ZONES.iter() {
                        SelectOption::<String> { value: zone.to_string(), text_value: zone.to_string(), "{zone}" }
                    }
                }
            }
        }
    }
}
