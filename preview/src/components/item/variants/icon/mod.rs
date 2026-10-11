use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::ShieldAlert;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            width: "100%",
            max_width: "28rem",

            Item { variant: ItemVariant::Outline,
                ItemMedia { variant: ItemMediaVariant::Icon,
                    ShieldAlert { size: "16" }
                }
                ItemContent {
                    ItemTitle { "Security Alert" }
                    ItemDescription { "New login detected from an unknown device." }
                }
                ItemActions {
                    Button { variant: ButtonVariant::Outline, "Review" }
                }
            }
        }
    }
}
