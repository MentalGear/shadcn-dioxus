use super::super::component::*;
use dioxus::prelude::*;

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");
const CASEY_PARK: Asset = asset!("/assets/avatars/casey-park.svg");
const PRIYA_NAIR: Asset = asset!("/assets/avatars/priya-nair.svg");

/// `shape: AvatarShape::Rounded`: a rounded-square avatar, beside the default circle at the same size, then
/// the rounded shape at the large size.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            align_items: "center",
            justify_content: "center",
            flex_wrap: "wrap",
            gap: "1.5rem",
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Circle" }
                ImageAvatar {
                    size: AvatarImageSize::Small,
                    src: AVERY_LIN.to_string(),
                    alt: "Avery Lin",
                    aria_label: "Circle avatar",
                    "AL"
                }
            }
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Rounded" }
                ImageAvatar {
                    size: AvatarImageSize::Small,
                    shape: AvatarShape::Rounded,
                    src: CASEY_PARK.to_string(),
                    alt: "Casey Park",
                    aria_label: "Rounded avatar",
                    "CP"
                }
            }
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Rounded, large" }
                ImageAvatar {
                    size: AvatarImageSize::Medium,
                    shape: AvatarShape::Rounded,
                    src: PRIYA_NAIR.to_string(),
                    alt: "Priya Nair",
                    aria_label: "Large rounded avatar",
                    "PN"
                }
            }
        }
    }
}
