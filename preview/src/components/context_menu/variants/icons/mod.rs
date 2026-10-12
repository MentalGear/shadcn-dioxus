use super::super::component::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Copy as CopyIcon, Download, Eye, FileText, FolderOpen, Link2, Trash2};

/// A file-row context menu: right-click the row and each action leads with a
/// lucide icon (shadcn's context-menu "icons" shape). The item is a flex row
/// with a `gap`, so the icon is just the first child.
///
/// The trigger text ("Q3-roadmap.pdf ... right-click for actions") and the item
/// names are chosen not to collide with what `context-menu.spec.ts` and the
/// shared `oracle/tier1-apg` specs query by name on this page ("right click
/// here", "Right click for ...", "Edit", "Share", ...).
///
/// Not shown: shortcut hints, a destructive-styled "Move to trash" and
/// separators between groups -- the component has no shortcut, destructive or
/// separator part yet (backlog row 155 group 2).
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                span { style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                    FileText { size: "16px" }
                    "Q3-roadmap.pdf"
                    span { style: "color: var(--dx-muted-foreground);", "(right-click for actions)" }
                }
            }
            ContextMenuContent {
                ContextMenuItem {
                    value: "Preview".to_string(),
                    index: 0usize,
                    text_value: "Preview",
                    on_select: move |value| chosen.set(Some(value)),
                    Eye { size: "16px" }
                    "Preview"
                }
                ContextMenuItem {
                    value: "Move to folder".to_string(),
                    index: 1usize,
                    text_value: "Move to folder",
                    on_select: move |value| chosen.set(Some(value)),
                    FolderOpen { size: "16px" }
                    "Move to folder"
                }
                ContextMenuItem {
                    value: "Make a copy".to_string(),
                    index: 2usize,
                    text_value: "Make a copy",
                    on_select: move |value| chosen.set(Some(value)),
                    CopyIcon { size: "16px" }
                    "Make a copy"
                }
                ContextMenuItem {
                    value: "Copy link".to_string(),
                    index: 3usize,
                    text_value: "Copy link",
                    on_select: move |value| chosen.set(Some(value)),
                    Link2 { size: "16px" }
                    "Copy link"
                }
                ContextMenuItem {
                    value: "Download".to_string(),
                    index: 4usize,
                    text_value: "Download",
                    on_select: move |value| chosen.set(Some(value)),
                    Download { size: "16px" }
                    "Download"
                }
                ContextMenuItem {
                    value: "Move to trash".to_string(),
                    index: 5usize,
                    text_value: "Move to trash",
                    disabled: true,
                    on_select: move |value| chosen.set(Some(value)),
                    Trash2 { size: "16px" }
                    "Move to trash"
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
