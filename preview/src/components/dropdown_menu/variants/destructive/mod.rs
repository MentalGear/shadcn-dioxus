use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuSeparator,
    DropdownMenuTrigger, MenuItemVariant,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Pencil, Share2, Trash2};

/// shadcn's `dropdown-menu-destructive` example: `variant="destructive"` draws
/// "Delete" in the destructive ink, with a destructive-tinted highlight, after a
/// `DropdownMenuSeparator` and in a `DropdownMenuGroup` of its own. The role is
/// still a plain `menuitem`.
///
/// The trigger name ("Danger zone") is chosen not to collide with what
/// `dropdown-menu.spec.ts` and the shared `oracle/tier1-apg` specs query by name
/// on this page.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { "Danger zone" }
            DropdownMenuContent {
                DropdownMenuGroup {
                    DropdownMenuItem::<String> {
                        value: "Rename".to_string(),
                        index: 0usize,
                        text_value: "Rename",
                        on_select: move |value| chosen.set(Some(value)),
                        Pencil { size: "16px" }
                        "Rename"
                    }
                    DropdownMenuItem::<String> {
                        value: "Share link".to_string(),
                        index: 1usize,
                        text_value: "Share link",
                        on_select: move |value| chosen.set(Some(value)),
                        Share2 { size: "16px" }
                        "Share link"
                    }
                }
                DropdownMenuSeparator {}
                DropdownMenuGroup {
                    DropdownMenuItem::<String> {
                        value: "Delete project".to_string(),
                        index: 2usize,
                        text_value: "Delete project",
                        variant: MenuItemVariant::Destructive,
                        on_select: move |value| chosen.set(Some(value)),
                        Trash2 { size: "16px" }
                        "Delete project"
                    }
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
