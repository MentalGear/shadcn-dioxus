use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Plus;

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");
const CASEY_PARK: Asset = asset!("/assets/avatars/casey-park.svg");
const PRIYA_NAIR: Asset = asset!("/assets/avatars/priya-nair.svg");

/// shadcn's `avatar-group`, `avatar-group-count` and `avatar-group-count-icon`: the overlapping stack
/// (each avatar ringed in the page background), the stack ending in a "+3" tile, and the stack ending in an
/// icon tile. The tile follows the size of the avatars beside it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            align_items: "center",
            gap: "1.5rem",
            AvatarGroup { style: "filter: grayscale(1);", aria_label: "Team",
                ImageAvatar { src: AVERY_LIN.to_string(), alt: "Avery Lin", "AL" }
                ImageAvatar { src: CASEY_PARK.to_string(), alt: "Casey Park", "CP" }
                ImageAvatar { src: PRIYA_NAIR.to_string(), alt: "Priya Nair", "PN" }
            }
            AvatarGroup { style: "filter: grayscale(1);", aria_label: "Team with a count",
                ImageAvatar { src: AVERY_LIN.to_string(), alt: "Avery Lin", "AL" }
                ImageAvatar { src: CASEY_PARK.to_string(), alt: "Casey Park", "CP" }
                ImageAvatar { src: PRIYA_NAIR.to_string(), alt: "Priya Nair", "PN" }
                AvatarGroupCount { "+3" }
            }
            AvatarGroup { style: "filter: grayscale(1);", aria_label: "Team with an icon tile",
                ImageAvatar { src: AVERY_LIN.to_string(), alt: "Avery Lin", "AL" }
                ImageAvatar { src: CASEY_PARK.to_string(), alt: "Casey Park", "CP" }
                ImageAvatar { src: PRIYA_NAIR.to_string(), alt: "Priya Nair", "PN" }
                AvatarGroupCount {
                    Plus {}
                }
            }
        }
    }
}
