use super::super::component::*;
use crate::components::avatar::{Avatar, AvatarFallback};
use crate::components::bubble::{
    Bubble, BubbleAlign, BubbleContent, BubbleGroup, BubbleReactions, BubbleVariant,
};
use crate::components::marker::{Marker, MarkerContent};
use dioxus::prelude::*;

/// The whole kit in one conversation: avatars on both sides, a delivery footer, a bubble group
/// with a reaction, and a typing indicator (`Marker` with `role: "status"` and `dx-shimmer`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; min-width: 0; flex-direction: column; gap: var(--dx-space-6);",
            Message { align: MessageAlign::End,
                MessageAvatar {
                    Person { name: "Me", initials: "ME" }
                }
                MessageContent {
                    Bubble {
                        BubbleContent { "Deploying to prod real quick." }
                    }
                }
            }
            Message {
                MessageAvatar {
                    Person { name: "Rabbit", initials: "R" }
                }
                MessageContent {
                    Bubble { variant: BubbleVariant::Muted,
                        BubbleContent { "It's 4:55 PM. On a Friday." }
                    }
                }
            }
            Message { align: MessageAlign::End,
                MessageAvatar {
                    Person { name: "Me", initials: "ME" }
                }
                MessageContent {
                    Bubble {
                        BubbleContent { "It's a one-line change." }
                    }
                    MessageFooter { "Delivered" }
                }
            }
            Message {
                MessageAvatar {
                    Person { name: "Rabbit", initials: "R" }
                }
                MessageContent {
                    BubbleGroup {
                        Bubble { variant: BubbleVariant::Muted, align: BubbleAlign::Start,
                            BubbleContent { "It's always a one-line change 😭." }
                        }
                        Bubble { variant: BubbleVariant::Muted, align: BubbleAlign::Start,
                            BubbleContent { "Alright, let me take a look." }
                            BubbleReactions { role: "img", aria_label: "Reactions: thumbs up",
                                span { "👍" }
                            }
                        }
                    }
                }
            }
            Marker { role: "status",
                MarkerContent { class: "dx-shimmer",
                    span { style: "font-weight: 500;", "Oliver" }
                    " is typing..."
                }
            }
        }
    }
}

/// A message avatar: the root is `role="img"`, so it carries a name; the fallback is restyled to
/// sit on the avatar slot's muted circle.
#[component]
fn Person(name: &'static str, initials: &'static str) -> Element {
    rsx! {
        Avatar { aria_label: name,
            AvatarFallback { style: "background: var(--dx-muted); font-size: var(--dx-text-xs);", "{initials}" }
        }
    }
}
