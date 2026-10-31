use super::super::component::*;
use dioxus::prelude::*;

// The readout under the bar shows the last selected value, so a spec can observe a selection
// (`playwright/oracle/tier1-apg/menu-family-parity.spec.ts`).
#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| None::<String>);

    rsx! {
        div {
            Menubar {
                MenubarMenu {
                    MenubarTrigger { "File" }
                    MenubarContent {
                        MenubarItem {
                            value: "new".to_string(),
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
                            },
                            "New"
                        }
                        MenubarItem {
                            value: "open".to_string(),
                            disabled: true,
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
                            },
                            "Open"
                        }
                        MenubarItem {
                            value: "save".to_string(),
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
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
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
                            },
                            "Cut"
                        }
                        MenubarItem {
                            value: "copy".to_string(),
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
                            },
                            "Copy"
                        }
                        MenubarItem {
                            value: "paste".to_string(),
                            on_select: move |value: String| {
                                tracing::info!("Selected value: {}", value);
                                selected.set(Some(value));
                            },
                            "Paste"
                        }
                    }
                }
            }
        }
        if let Some(value) = selected() {
            span { "Selected: {value}" }
        }
    }
}
