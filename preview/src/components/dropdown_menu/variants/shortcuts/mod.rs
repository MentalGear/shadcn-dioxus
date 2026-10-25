use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
    DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuTrigger,
};
use dioxus::prelude::*;

/// shadcn's `dropdown-menu-shortcuts` example: a `DropdownMenuGroup` headed by a
/// `DropdownMenuLabel`, a `DropdownMenuShortcut` hint at the inline end of each
/// item, a `DropdownMenuSeparator`, then a lone item. The shortcut is text only;
/// it binds no key.
///
/// The trigger name ("Shortcuts menu") and the items are chosen not to collide
/// with what `dropdown-menu.spec.ts` and the shared `oracle/tier1-apg` specs
/// query by name on this page ("Open Menu", "Actions", "Edit", ...).
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { "Shortcuts menu" }
            DropdownMenuContent {
                DropdownMenuGroup { aria_label: "My Account",
                    DropdownMenuLabel { "My Account" }
                    DropdownMenuItem::<String> {
                        value: "Profile".to_string(),
                        text_value: "Profile",
                        on_select: move |value| chosen.set(Some(value)),
                        "Profile"
                        DropdownMenuShortcut { "⇧⌘P" }
                    }
                    DropdownMenuItem::<String> {
                        value: "Billing".to_string(),
                        text_value: "Billing",
                        on_select: move |value| chosen.set(Some(value)),
                        "Billing"
                        DropdownMenuShortcut { "⌘B" }
                    }
                    DropdownMenuItem::<String> {
                        value: "Settings".to_string(),
                        text_value: "Settings",
                        on_select: move |value| chosen.set(Some(value)),
                        "Settings"
                        DropdownMenuShortcut { "⌘S" }
                    }
                }
                DropdownMenuSeparator {}
                DropdownMenuItem::<String> {
                    value: "Log out".to_string(),
                    text_value: "Log out",
                    on_select: move |value| chosen.set(Some(value)),
                    "Log out"
                    DropdownMenuShortcut { "⇧⌘Q" }
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
