use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            Menubar {
                MenubarMenu {
                    MenubarTrigger { "File" }
                    MenubarContent {
                        MenubarItem {
                            value: "new".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "New"
                        }
                        MenubarItem {
                            value: "open".to_string(),
                            disabled: true,
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Open"
                        }
                        MenubarItem {
                            value: "save".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Save"
                        }
                    }
                }
                MenubarMenu {
                    MenubarTrigger { "Edit" }
                    MenubarContent {
                        MenubarItem {
                            value: "cut".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Cut"
                        }
                        MenubarItem {
                            value: "copy".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Copy"
                        }
                        MenubarItem {
                            value: "paste".to_string(),
                            on_select: move |value| {
                                tracing::info!("Selected value: {}", value);
                            },
                            "Paste"
                        }
                    }
                }
            }
        }
    }
}
