use super::super::component::*;
use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::Plus;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            width: "100%",
            max_width: "28rem",

            Item { variant: ItemVariant::Outline,
                ItemMedia {
                    ImageAvatar {
                        size: AvatarImageSize::Medium,
                        src: asset!("/assets/avatars/avery-lin.svg").to_string(),
                        alt: "Avery Lin",
                        "AL"
                    }
                }
                ItemContent {
                    ItemTitle { "Avery Lin" }
                    ItemDescription { "Last seen 5 months ago" }
                }
                ItemActions {
                    Button {
                        variant: ButtonVariant::Outline,
                        aria_label: "Invite Avery Lin",
                        Plus { size: "16" }
                    }
                }
            }
        }
    }
}
