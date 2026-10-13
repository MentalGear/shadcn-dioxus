use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::{ContentAlign, ContentSide};

/// `side` and `align` on `PopoverContent`. The popover is non-modal so several triggers can sit
/// side by side; it is light-dismissed, and every panel is anchored to its own trigger.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: grid; gap: 1.5rem; justify-items: center; padding: 1rem;",
            div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem; justify-content: center;",
                PlacementPopover {
                    label: "Place top",
                    side: ContentSide::Top,
                    align: ContentAlign::Center,
                    note: "Opens above the trigger.",
                }
                PlacementPopover {
                    label: "Place right",
                    side: ContentSide::Right,
                    align: ContentAlign::Center,
                    note: "Opens to the right of the trigger.",
                }
                PlacementPopover {
                    label: "Place bottom",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Center,
                    note: "Opens below the trigger.",
                }
                PlacementPopover {
                    label: "Place left",
                    side: ContentSide::Left,
                    align: ContentAlign::Center,
                    note: "Opens to the left of the trigger.",
                }
            }
            div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem; justify-content: center;",
                PlacementPopover {
                    label: "Align start",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    note: "Aligned to the start edge of the trigger.",
                }
                PlacementPopover {
                    label: "Align center",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Center,
                    note: "Centred on the trigger.",
                }
                PlacementPopover {
                    label: "Align end",
                    side: ContentSide::Bottom,
                    align: ContentAlign::End,
                    note: "Aligned to the end edge of the trigger.",
                }
            }
        }
    }
}

#[component]
fn PlacementPopover(
    label: &'static str,
    side: ContentSide,
    align: ContentAlign,
    note: &'static str,
) -> Element {
    rsx! {
        PopoverRoot { is_modal: false,
            PopoverTrigger { "{label}" }
            PopoverContent { side, align, width: "14rem",
                PopoverHeader {
                    PopoverTitle { "{label}" }
                    PopoverDescription { "{note}" }
                }
            }
        }
    }
}
