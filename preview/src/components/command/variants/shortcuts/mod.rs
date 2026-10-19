use super::super::component::{Command, CommandEmpty, CommandGroup, CommandGroupLabel, CommandItem, CommandShortcut};
use dioxus::prelude::*;

/// An inline `Command` whose items end in a `CommandShortcut` hint, as shadcn's command demo does.
#[component]
pub fn Demo() -> Element {
    let items = [("Profile", "⌘P"), ("Billing", "⌘B"), ("Settings", "⌘S")];
    rsx! {
        div { style: "width: 100%; max-width: 24rem; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); overflow: hidden;",
            Command::<String> {
                input_aria_label: "Search settings",
                placeholder: "Type a command or search...",
                list_aria_label: "Settings",
                CommandEmpty { "No results found." }
                CommandGroup {
                    CommandGroupLabel { "Settings" }
                    for (i, (label, keys)) in items.iter().enumerate() {
                        CommandItem::<String> {
                            index: i,
                            value: label.to_string(),
                            text_value: *label,
                            "{label}"
                            CommandShortcut { "{keys}" }
                        }
                    }
                }
            }
        }
    }
}
