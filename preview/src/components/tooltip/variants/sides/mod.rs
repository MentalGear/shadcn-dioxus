use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_primitives::ContentSide;

/// shadcn's "Side" tooltip demo: one outline button per `side` of `TooltipContent`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem; justify-content: center; padding: 2rem 1rem;",
            SideTip { label: "Tip above", side: ContentSide::Top, text: "Opens above the trigger" }
            SideTip { label: "Tip right", side: ContentSide::Right, text: "Opens to the right of the trigger" }
            SideTip { label: "Tip below", side: ContentSide::Bottom, text: "Opens below the trigger" }
            SideTip { label: "Tip left", side: ContentSide::Left, text: "Opens to the left of the trigger" }
        }
    }
}

#[component]
fn SideTip(label: String, side: ContentSide, text: String) -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                as: move |el: AsChild| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes: el.attributes, "{label}" }
                },
            }
            TooltipContent { side, "{text}" }
        }
    }
}
