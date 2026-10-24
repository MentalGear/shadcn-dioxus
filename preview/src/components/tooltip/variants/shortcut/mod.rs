use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::kbd::{Kbd, KbdGroup};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// shadcn's keyboard-shortcut tooltip: a "Save" button whose tooltip names the
/// action and the shortcut that performs it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                as: |el: AsChild| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes: el.attributes, "Save" }
                },
            }
            TooltipContent {
                div { style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { "Save changes" }
                    KbdGroup {
                        Kbd { "⌘" }
                        Kbd { "S" }
                    }
                }
            }
        }
    }
}
