use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Plus;

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");
const NINA_HART: Asset = asset!("/assets/avatars/nina-hart.svg");

/// shadcn's `avatar-badge` and `avatar-badge-icon`: a status dot (green here) and a default-coloured badge
/// holding an icon, then the three sizes to show how the badge scales (8px with no icon in the small avatar,
/// 10px, 12px).
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
                p { class: "dx-avatar-label", "Status dot" }
                Avatar { aria_label: "Avery Lin, online",
                    AvatarImage { src: AVERY_LIN.to_string(), alt: "" }
                    AvatarFallback { "AL" }
                    AvatarBadge { style: "background-color: var(--dx-success);" }
                }
            }
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Icon" }
                Avatar { style: "filter: grayscale(1);", aria_label: "Nina Hart, add",
                    AvatarImage { src: NINA_HART.to_string(), alt: "" }
                    AvatarFallback { "NH" }
                    AvatarBadge {
                        Plus {}
                    }
                }
            }
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Small" }
                Avatar { size: AvatarImageSize::ExtraSmall, aria_label: "Small avatar with badge",
                    AvatarImage { src: AVERY_LIN.to_string(), alt: "" }
                    AvatarFallback { "AL" }
                    AvatarBadge {
                        Plus {}
                    }
                }
            }
            div { class: "dx-avatar-item",
                p { class: "dx-avatar-label", "Large" }
                Avatar { size: AvatarImageSize::Medium, aria_label: "Large avatar with badge",
                    AvatarImage { src: AVERY_LIN.to_string(), alt: "" }
                    AvatarFallback { "AL" }
                    AvatarBadge {
                        Plus {}
                    }
                }
            }
        }
    }
}
