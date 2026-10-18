use super::super::component::*;
use dioxus::prelude::*;

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");
const CASEY_PARK: Asset = asset!("/assets/avatars/casey-park.svg");
const LENA_PARK: Asset = asset!("/assets/avatars/lena-park.svg");
const PRIYA_NAIR: Asset = asset!("/assets/avatars/priya-nair.svg");

/// shadcn's `avatar-demo`: a grayscale avatar, an avatar with a green status badge, and a grayscale
/// group ending in a "+3" count tile. The pictures are bundled illustrations (`preview/assets/avatars`),
/// so the demo loads the same offline and no real person's photo is hot-linked. The loading and error
/// states are in the `states` example.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            flex_wrap: "wrap",
            align_items: "center",
            justify_content: "center",
            gap: "clamp(1.5rem, 6vw, 3rem)",
            Avatar { aria_label: "Grayscale avatar",
                AvatarImage {
                    src: AVERY_LIN.to_string(),
                    alt: "Avery Lin",
                    style: "filter: grayscale(1);",
                }
                AvatarFallback { "AL" }
            }
            Avatar { aria_label: "Online avatar",
                AvatarImage { src: LENA_PARK.to_string(), alt: "Lena Park" }
                AvatarFallback { "LP" }
                AvatarBadge { style: "background-color: var(--dx-success);" }
            }
            AvatarGroup { style: "filter: grayscale(1);", aria_label: "Team",
                ImageAvatar { src: AVERY_LIN.to_string(), alt: "Avery Lin", "AL" }
                ImageAvatar { src: CASEY_PARK.to_string(), alt: "Casey Park", "CP" }
                ImageAvatar { src: PRIYA_NAIR.to_string(), alt: "Priya Nair", "PN" }
                AvatarGroupCount { "+3" }
            }
        }
    }
}
