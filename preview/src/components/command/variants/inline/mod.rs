use super::super::component::{
    Command, CommandEmpty, CommandGroup, CommandGroupLabel, CommandItem,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Calculator, Calendar, CreditCard, Settings, Smile, User};

/// A `Command` rendered inline (no `CommandDialog`): a bordered filterable list with icons,
/// two groups and an empty state. Names deliberately avoid the main demo's "commands" wording so
/// its page-wide locators stay unambiguous.
#[component]
pub fn Demo() -> Element {
    let mut picked = use_signal(|| None::<String>);
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 24rem;",
            div { style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
                Command::<String> {
                    on_value_change: move |value: Option<String>| picked.set(value),
                    input_aria_label: "Filter quick actions",
                    placeholder: "Type an action or search...",
                    list_aria_label: "Quick actions",
                    CommandEmpty { "Nothing matches that search." }
                    CommandGroup {
                        CommandGroupLabel { "Suggestions" }
                        CommandItem::<String> {
                            index: 0usize,
                            value: "calendar".to_string(),
                            text_value: "Calendar",
                            Calendar { size: "16px" }
                            "Calendar"
                        }
                        CommandItem::<String> {
                            index: 1usize,
                            value: "emoji".to_string(),
                            text_value: "Search Emoji",
                            Smile { size: "16px" }
                            "Search Emoji"
                        }
                        CommandItem::<String> {
                            index: 2usize,
                            value: "calculator".to_string(),
                            text_value: "Calculator",
                            disabled: true,
                            Calculator { size: "16px" }
                            "Calculator"
                        }
                    }
                    CommandGroup {
                        CommandGroupLabel { "Account" }
                        CommandItem::<String> {
                            index: 3usize,
                            value: "profile".to_string(),
                            text_value: "Profile",
                            User { size: "16px" }
                            "Profile"
                        }
                        CommandItem::<String> {
                            index: 4usize,
                            value: "billing".to_string(),
                            text_value: "Billing",
                            CreditCard { size: "16px" }
                            "Billing"
                        }
                        CommandItem::<String> {
                            index: 5usize,
                            value: "settings".to_string(),
                            text_value: "Settings",
                            Settings { size: "16px" }
                            "Settings"
                        }
                    }
                }
            }
            p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                if let Some(value) = picked() {
                    "Selected: {value}"
                } else {
                    "Pick an action to see it here."
                }
            }
        }
    }
}
