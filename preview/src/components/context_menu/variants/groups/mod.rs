use super::super::component::{
    ContextMenu, ContextMenuContent, ContextMenuGroup, ContextMenuItem, ContextMenuLabel,
    ContextMenuSeparator, ContextMenuShortcut, ContextMenuTrigger, MenuItemVariant,
};
use dioxus::prelude::*;

/// shadcn's `context-menu-groups` example: four `ContextMenuGroup`s split by
/// `ContextMenuSeparator`s, two of them headed by a `ContextMenuLabel`, a
/// `ContextMenuShortcut` hint on each item, and a last group holding one
/// `variant="destructive"` item. Items are indexed across the whole menu: groups
/// add structure, not a second collection.
///
/// The trigger name ("Right click for groups") and the item names are chosen not
/// to collide with what `context-menu.spec.ts` and the shared `oracle/tier1-apg`
/// specs query by name on this page.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        ContextMenu {
            ContextMenuTrigger { "Right click for groups" }
            ContextMenuContent {
                ContextMenuGroup { aria_label: "File",
                    ContextMenuLabel { "File" }
                    ContextMenuItem {
                        value: "New File".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "New File"
                        ContextMenuShortcut { "⌘N" }
                    }
                    ContextMenuItem {
                        value: "Open File".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Open File"
                        ContextMenuShortcut { "⌘O" }
                    }
                    ContextMenuItem {
                        value: "Save File".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Save File"
                        ContextMenuShortcut { "⌘S" }
                    }
                }
                ContextMenuSeparator {}
                ContextMenuGroup { aria_label: "History",
                    ContextMenuLabel { "History" }
                    ContextMenuItem {
                        value: "Undo".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Undo"
                        ContextMenuShortcut { "⌘Z" }
                    }
                    ContextMenuItem {
                        value: "Redo".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Redo"
                        ContextMenuShortcut { "⇧⌘Z" }
                    }
                }
                ContextMenuSeparator {}
                ContextMenuGroup {
                    ContextMenuItem {
                        value: "Cut".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Cut"
                        ContextMenuShortcut { "⌘X" }
                    }
                    ContextMenuItem {
                        value: "Copy".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Copy"
                        ContextMenuShortcut { "⌘C" }
                    }
                    ContextMenuItem {
                        value: "Paste".to_string(),
                        on_select: move |value| chosen.set(Some(value)),
                        "Paste"
                        ContextMenuShortcut { "⌘V" }
                    }
                }
                ContextMenuSeparator {}
                ContextMenuGroup {
                    ContextMenuItem {
                        value: "Delete selection".to_string(),
                        variant: MenuItemVariant::Destructive,
                        on_select: move |value| chosen.set(Some(value)),
                        "Delete selection"
                        ContextMenuShortcut { "⌫" }
                    }
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
