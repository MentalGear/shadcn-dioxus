use super::super::component::{Command, CommandEmpty, CommandGroup, CommandGroupLabel, CommandItem};
use dioxus::prelude::*;

const TIMEZONES: [&str; 24] = [
    "Pacific/Honolulu", "America/Anchorage", "America/Los_Angeles", "America/Denver",
    "America/Chicago", "America/New_York", "America/Halifax", "America/Sao_Paulo",
    "Atlantic/Azores", "Europe/London", "Europe/Paris", "Europe/Athens", "Africa/Cairo",
    "Europe/Moscow", "Asia/Dubai", "Asia/Karachi", "Asia/Dhaka", "Asia/Bangkok",
    "Asia/Shanghai", "Asia/Tokyo", "Australia/Sydney", "Pacific/Auckland", "Pacific/Fiji",
    "Pacific/Tongatapu",
];

/// An inline `Command` with more items than fit: the list scrolls inside its frame and the
/// filter input stays above it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
            Command::<String> {
                input_aria_label: "Filter time zones",
                placeholder: "Search time zones...",
                list_aria_label: "Time zones",
                CommandEmpty { "No time zone found." }
                CommandGroup {
                    CommandGroupLabel { "Time zones" }
                    for (i, zone) in TIMEZONES.iter().enumerate() {
                        CommandItem::<String> {
                            index: i,
                            value: zone.to_string(),
                            text_value: *zone,
                            "{zone}"
                        }
                    }
                }
            }
        }
    }
}
