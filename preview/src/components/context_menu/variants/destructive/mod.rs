use super::super::component::{
    ContextMenu, ContextMenuContent, ContextMenuGroup, ContextMenuItem, ContextMenuSeparator,
    ContextMenuTrigger, MenuItemVariant,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Pencil, Share2, Trash2};

/// shadcn's `context-menu-destructive` example: `variant="destructive"` draws
/// "Delete" in the destructive ink, with a destructive-tinted highlight, in a
/// group of its own after a `ContextMenuSeparator`. The role is still a plain
/// `menuitem`.
///
/// The trigger text ("Right click for destructive") is chosen not to collide
/// with what `context-menu.spec.ts` and the shared `oracle/tier1-apg` specs query
/// by name on this page.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        ContextMenu {
            ContextMenuTrigger { "Right click for destructive" }
            ContextMenuContent {
                ContextMenuGroup {
                    ContextMenuItem {
                        value: "Rename note".to_string(),
                        index: 0usize,
                        on_select: move |value| chosen.set(Some(value)),
                        Pencil { size: "16px" }
                        "Rename note"
                    }
                    ContextMenuItem {
                        value: "Share note".to_string(),
                        index: 1usize,
                        on_select: move |value| chosen.set(Some(value)),
                        Share2 { size: "16px" }
                        "Share note"
                    }
                }
                ContextMenuSeparator {}
                ContextMenuGroup {
                    ContextMenuItem {
                        value: "Delete note".to_string(),
                        index: 2usize,
                        variant: MenuItemVariant::Destructive,
                        on_select: move |value| chosen.set(Some(value)),
                        Trash2 { size: "16px" }
                        "Delete note"
                    }
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
