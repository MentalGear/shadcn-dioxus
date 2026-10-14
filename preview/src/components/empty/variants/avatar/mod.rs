use super::super::component::*;
use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use crate::components::button::Button;
use dioxus::prelude::*;

const OFFLINE_USER: Asset = asset!("/assets/avatars/robin-hayes.svg");

#[component]
pub fn Demo() -> Element {
    rsx! {
        Empty { style: "width: 100%; max-width: 28rem;",
            EmptyHeader {
                EmptyMedia {
                    ImageAvatar {
                        size: AvatarImageSize::Medium,
                        src: OFFLINE_USER.to_string(),
                        alt: "Robin Hayes",
                    }
                }
                EmptyTitle { "User offline" }
                EmptyDescription {
                    "This user is currently offline. You can leave a message to notify them or try again later."
                }
            }
            EmptyContent {
                Button { "Leave a message" }
            }
        }
    }
}
