use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                as: |el: AsChild| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes: el.attributes, "Copy link" }
                },
            }
            TooltipContent {
                div { style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { "Copy link" }
                    KbdGroup {
                        Kbd { "⌘" }
                        Kbd { "C" }
                    }
                }
            }
        }
    }
}
