use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Globe;

const TIME_ZONES: &[&str] = &[
    "Europe/Berlin",
    "Europe/London",
    "America/New_York",
    "America/Sao_Paulo",
    "Asia/Tokyo",
    "Australia/Sydney",
];

/// `addon`: an icon inside the input's edge at its inline start, the way shadcn's demo puts an
/// `InputGroupAddon` in its `ComboboxInput`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { class: "dx-combobox-demo",
            Combobox::<String> {
                item_to_label: |value: String| value,
                addon: rsx! {
                    Globe {}
                },
                style: "--dx-combobox-width: 16rem;",
                placeholder: "Select time zone...",
                aria_label: "Time zone",
                list_aria_label: "Time zone list",
                ComboboxEmpty { "No time zone found." }
                for zone in TIME_ZONES.iter() {
                    ComboboxOption::<String> { value: zone.to_string(), text_value: *zone, {*zone} }
                }
            }
        }
    }
}
