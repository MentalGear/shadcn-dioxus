use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::ContentSide;

/// Radix's `forceMount` on `HoverCardContent`: the card stays in the DOM while it is closed
/// (hidden, `data-state="closed"`), instead of being mounted when it opens and removed after its
/// exit animation. It still opens, and fades out, exactly like the default card.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "padding: 50px; display: flex; justify-content: center;",
            HoverCard { delay: 10u64, close_delay: 100u64,
                HoverCardTrigger { "Always mounted" }
                HoverCardContent { side: ContentSide::Bottom, force_mount: true,
                    div { style: "padding: 0.5rem; width: 14rem;",
                        "This card is in the markup before it is ever opened."
                    }
                }
            }
        }
    }
}
