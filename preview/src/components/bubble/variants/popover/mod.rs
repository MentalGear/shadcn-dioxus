use super::super::component::*;
use crate::components::popover::{
    PopoverContent, PopoverDescription, PopoverHeader, PopoverRoot, PopoverTitle, PopoverTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::Info;

/// Error details in a popover on a destructive bubble. `PopoverTrigger` has no `as`, so the
/// icon-only look (no border or fill, a 1.5rem square) is set on the trigger itself.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-8);",
            Bubble { align: BubbleAlign::End,
                BubbleContent { "Run the build script." }
            }
            Bubble { variant: BubbleVariant::Destructive,
                BubbleContent { "Failed to run the command." }
                BubbleReactions {
                    PopoverRoot {
                        PopoverTrigger {
                            aria_label: "Show error details",
                            style: "width: 1.5rem; height: 1.5rem; padding: 0; border-color: transparent; background: transparent; color: inherit;",
                            Info {}
                        }
                        PopoverContent {
                            PopoverHeader {
                                PopoverTitle { "Command failed with exit code 1" }
                                PopoverDescription { "ENOENT: no such file or directory, open pnpm-lock.yaml" }
                            }
                        }
                    }
                }
            }
        }
    }
}
