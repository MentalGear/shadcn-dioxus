use super::super::component::{
    Menubar, MenubarContent, MenubarGroup, MenubarItem, MenubarMenu, MenubarSeparator,
    MenubarShortcut, MenubarTrigger, MenuItemVariant,
};
use dioxus::prelude::*;

/// shadcn's `menubar-demo` parts: a `MenubarShortcut` hint at the inline end of
/// each item, `MenubarGroup`s split by `MenubarSeparator`s, `inset` items that
/// line up with a menu's checkable ones, and a `variant="destructive"` item.
/// The shortcut is text only; it binds no key.
///
/// Trigger names ("Document", "Selection") and item names are chosen not to
/// collide with what `menubar.spec.ts` and the shared `oracle/tier1-apg` specs
/// query by name on this page ("File", "Edit", "View", ...); trigger indices are
/// per-menubar.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        div {
            Menubar { aria_label: "Document menu",
                MenubarMenu {
                    MenubarTrigger { "Document" }
                    MenubarContent {
                        MenubarGroup {
                            MenubarItem {
                                value: "New Tab".to_string(),
                                text_value: "New Tab",
                                on_select: move |value| chosen.set(Some(value)),
                                "New Tab"
                                MenubarShortcut { "⌘T" }
                            }
                            MenubarItem {
                                value: "New Window".to_string(),
                                text_value: "New Window",
                                on_select: move |value| chosen.set(Some(value)),
                                "New Window"
                                MenubarShortcut { "⌘N" }
                            }
                            MenubarItem {
                                value: "New Private Window".to_string(),
                                text_value: "New Private Window",
                                disabled: true,
                                on_select: move |value| chosen.set(Some(value)),
                                "New Private Window"
                            }
                        }
                        MenubarSeparator {}
                        MenubarGroup {
                            MenubarItem {
                                value: "Print".to_string(),
                                text_value: "Print",
                                on_select: move |value| chosen.set(Some(value)),
                                "Print..."
                                MenubarShortcut { "⌘P" }
                            }
                        }
                    }
                }
                MenubarMenu {
                    MenubarTrigger { "Selection" }
                    MenubarContent {
                        MenubarGroup {
                            MenubarItem {
                                value: "Select All".to_string(),
                                text_value: "Select All",
                                inset: true,
                                on_select: move |value| chosen.set(Some(value)),
                                "Select All"
                                MenubarShortcut { "⌘A" }
                            }
                            MenubarItem {
                                value: "Invert Selection".to_string(),
                                text_value: "Invert Selection",
                                inset: true,
                                on_select: move |value| chosen.set(Some(value)),
                                "Invert Selection"
                            }
                        }
                        MenubarSeparator {}
                        MenubarGroup {
                            MenubarItem {
                                value: "Delete Selection".to_string(),
                                text_value: "Delete Selection",
                                variant: MenuItemVariant::Destructive,
                                on_select: move |value| chosen.set(Some(value)),
                                "Delete Selection"
                                MenubarShortcut { "⌫" }
                            }
                        }
                    }
                }
            }
            if let Some(item) = chosen() {
                p { style: "margin-top: 0.75rem;", "Chose: {item}" }
            }
        }
    }
}
