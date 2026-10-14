use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::Check;

/// A tooltip on a bubble's reaction: `TooltipTrigger`'s `as` renders the trigger as a ghost
/// icon `Button`, so the read receipt is one focusable control with a hover and focus hint.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-8);",
            Bubble { variant: BubbleVariant::Secondary,
                BubbleContent { "Did you remove the stale route?" }
            }
            Bubble { align: BubbleAlign::End,
                BubbleContent { "Yes, removed it from the registry." }
                BubbleReactions {
                    Tooltip {
                        TooltipTrigger {
                            as: move |attributes: Vec<Attribute>| rsx! {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconXs,
                                    r#type: "button",
                                    aria_label: "Read on Jan 5, 2026 at 4:32 PM",
                                    attributes,
                                    Check {}
                                }
                            },
                        }
                        TooltipContent { "Read on Jan 5, 2026 at 4:32 PM" }
                    }
                }
            }
        }
    }
}
