use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::{ContentAlign, ContentSide};

/// `side` and `align` on `HoverCardContent`: the card opens on each side of its trigger, and
/// (below it) aligned to the start, centre or end of the trigger.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: grid; gap: 1.5rem; justify-items: center; padding: 3rem 1rem;",
            div { style: "display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center;",
                PlacementCard {
                    label: "Top",
                    side: ContentSide::Top,
                    align: ContentAlign::Center,
                    note: "Opens above the trigger.",
                }
                PlacementCard {
                    label: "Right",
                    side: ContentSide::Right,
                    align: ContentAlign::Center,
                    note: "Opens to the right of the trigger.",
                }
                PlacementCard {
                    label: "Bottom",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Center,
                    note: "Opens below the trigger.",
                }
                PlacementCard {
                    label: "Left",
                    side: ContentSide::Left,
                    align: ContentAlign::Center,
                    note: "Opens to the left of the trigger.",
                }
            }
            div { style: "display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center;",
                PlacementCard {
                    label: "Align start",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Start,
                    note: "Aligned to the start edge of the trigger.",
                }
                PlacementCard {
                    label: "Align center",
                    side: ContentSide::Bottom,
                    align: ContentAlign::Center,
                    note: "Centred on the trigger.",
                }
                PlacementCard {
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
fn PlacementCard(
    label: &'static str,
    side: ContentSide,
    align: ContentAlign,
    note: &'static str,
) -> Element {
    rsx! {
        HoverCard {
            HoverCardTrigger { "{label}" }
            HoverCardContent { side, align,
                div { style: "padding: 0.5rem; width: 12rem;", "{note}" }
            }
        }
    }
}
