use super::super::component::*;
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;

const ROLES: &[&str] = &["Viewer", "Editor", "Admin"];

#[component]
pub fn Demo() -> Element {
    let mut role = use_signal(|| "Editor".to_string());

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            width: "100%",
            max_width: "28rem",

            Item { variant: ItemVariant::Outline,
                ItemContent {
                    ItemTitle { "Workspace access" }
                    ItemDescription { "Choose what this member can do. Current role: {role}." }
                }
                ItemActions {
                    DropdownMenu {
                        DropdownMenuTrigger { aria_label: "Change workspace role", "Change role" }
                        DropdownMenuContent {
                            for name in ROLES.iter() {
                                DropdownMenuItem::<String> {
                                    value: name.to_string(),
                                    text_value: *name,
                                    on_select: move |value: String| role.set(value),
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
