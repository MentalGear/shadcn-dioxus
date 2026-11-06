use super::super::component::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
};
use dioxus::prelude::*;

/// `modal: false`: an open menu does not lock page scrolling, so the page behind it stays
/// scrollable.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        ContextMenu { modal: false,
            ContextMenuTrigger { "Right click (non-modal)" }
            ContextMenuContent {
                ContextMenuItem {
                    value: "Copy".to_string(),
                    on_select: move |value| chosen.set(Some(value)),
                    "Copy"
                }
                ContextMenuItem {
                    value: "Paste".to_string(),
                    on_select: move |value| chosen.set(Some(value)),
                    "Paste"
                }
            }
        }
        if let Some(item) = chosen() {
            p { id: "context-menu-non-modal-chosen", "Selected: {item}" }
        }
    }
}
