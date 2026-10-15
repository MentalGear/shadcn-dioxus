use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Download, Pencil, Pin, Share2, Star};

/// shadcn's "Icons" example: a lucide icon leads each item's label. The item
/// is already a `display: flex` row with a `gap`, so the icon is just the
/// first child -- no part of its own is needed.
///
/// Labels and the trigger name ("Message menu") are chosen not to collide with
/// what `dropdown-menu.spec.ts` and the shared `oracle/tier1-apg` specs query
/// by name on this page ("Open Menu", "Actions", "Edit", ...).
///
/// Shortcut hints, the destructive item and separators between groups are the
/// `shortcuts` and `destructive` variants.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { "Message menu" }
            DropdownMenuContent {
                DropdownMenuItem::<String> {
                    value: "Compose".to_string(),
                    index: 0usize,
                    text_value: "Compose",
                    on_select: move |value| chosen.set(Some(value)),
                    Pencil { size: "16px" }
                    "Compose"
                }
                DropdownMenuItem::<String> {
                    value: "Star".to_string(),
                    index: 1usize,
                    text_value: "Star",
                    on_select: move |value| chosen.set(Some(value)),
                    Star { size: "16px" }
                    "Star"
                }
                DropdownMenuItem::<String> {
                    value: "Pin to top".to_string(),
                    index: 2usize,
                    text_value: "Pin to top",
                    on_select: move |value| chosen.set(Some(value)),
                    Pin { size: "16px" }
                    "Pin to top"
                }
                DropdownMenuItem::<String> {
                    value: "Share".to_string(),
                    index: 3usize,
                    text_value: "Share",
                    on_select: move |value| chosen.set(Some(value)),
                    Share2 { size: "16px" }
                    "Share"
                }
                DropdownMenuItem::<String> {
                    value: "Export".to_string(),
                    index: 4usize,
                    text_value: "Export",
                    disabled: true,
                    on_select: move |value| chosen.set(Some(value)),
                    Download { size: "16px" }
                    "Export"
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
