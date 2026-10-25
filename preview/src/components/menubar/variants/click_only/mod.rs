use super::super::component::*;
use dioxus::prelude::*;

/// Click-activation fixture (`open_on_hover: false`).
///
/// Once a menu is open, moving the pointer across the other triggers does
/// **not** switch menus -- another menu opens only on click, Enter/Space or
/// the arrow keys. (The default `main` variant hover-switches.)
///
/// Every label below ("Insert"/"Tools"/"Table"/"Image"/"Spelling"/
/// "Statistics") deliberately differs from the `main` variant's
/// ("File"/"Edit"/"New"/"Open"/"Save"/"Cut"/"Copy"/"Paste"):
/// `menubar.spec.ts` and the oracle `keyboard-matrix`/`menu-roles` specs
/// query those words by name (substring, no `exact`), and this variant
/// renders alongside `main` on the same page, so reusing them would make
/// those locators match both.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            Menubar { open_on_hover: false,
                MenubarMenu {
                    MenubarTrigger { "Insert" }
                    MenubarContent {
                        MenubarItem {
                            value: "table".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Table"
                        }
                        MenubarItem {
                            value: "image".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Image"
                        }
                    }
                }
                MenubarMenu {
                    MenubarTrigger { "Tools" }
                    MenubarContent {
                        MenubarItem {
                            value: "spelling".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Spelling"
                        }
                        MenubarItem {
                            value: "statistics".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Statistics"
                        }
                    }
                }
            }
        }
    }
}
