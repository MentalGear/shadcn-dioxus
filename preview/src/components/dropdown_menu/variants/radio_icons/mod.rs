use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuRadioGroup,
    DropdownMenuRadioItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Monitor, Moon, Sun};

/// shadcn's "Radio Group" with icons: an appearance picker whose items each
/// lead with a lucide icon. The radio item keeps its own check indicator at
/// the inline end, so the icon is simply the first child. Controlled, and
/// the items keep the menu open (the default) so the checked item can be seen
/// to move without reopening the menu.
///
/// The trigger ("Appearance") and group heading differ from the sibling
/// `radio_group` variant's ("Radio Group"/"Panel Position"), whose names
/// `oracle/tier1-apg/menu-roles.spec.ts` queries on this same page.
#[component]
pub fn Demo() -> Element {
    let mut appearance = use_signal(|| "system".to_string());

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { "Appearance" }
            DropdownMenuContent {
                DropdownMenuLabel { id: "dropdown-menu-radio-icons-label", "Color scheme" }
                DropdownMenuRadioGroup {
                    aria_labelledby: "dropdown-menu-radio-icons-label",
                    value: Some(appearance()),
                    on_value_change: move |value| appearance.set(value),
                    DropdownMenuRadioItem {
                        value: "light".to_string(),
                        Sun { size: "16px" }
                        "Light"
                    }
                    DropdownMenuRadioItem {
                        value: "dark".to_string(),
                        Moon { size: "16px" }
                        "Dark"
                    }
                    DropdownMenuRadioItem {
                        value: "system".to_string(),
                        Monitor { size: "16px" }
                        "System"
                    }
                }
            }
        }
        p { style: "margin-top: 0.75rem;", "Color scheme: {appearance()}" }
    }
}
