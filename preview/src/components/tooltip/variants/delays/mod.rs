use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// `delay` and `close_delay` on `Tooltip` (Radix `delayDuration`, Base UI `delay` / `closeDelay`):
/// the styled tooltip opens on contact by default (shadcn's `TooltipProvider delay={0}`),
/// `delay: 600` asks for the primitive's Base UI rest-to-open delay, and a `close_delay` keeps the
/// tooltip open long enough to move the pointer onto it. Keyboard focus opens each one at once.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "padding: 2rem 1rem; display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center;",
            // No `delay` prop: the styled default (0).
            Tooltip {
                TooltipTrigger {
                    as: |el: AsChild| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes: el.attributes, "Default (0)" }
                    },
                }
                TooltipContent { "Opens on contact." }
            }
            DelayTooltip { label: "Rests (600)", delay: 600, note: "Opens after 600 ms of rest." }
            DelayTooltip {
                label: "Lingers (0 / 600)",
                delay: 0,
                close_delay: 600,
                note: "Stays 600 ms after the pointer leaves, and while it is over this tooltip.",
            }
        }
    }
}

#[component]
fn DelayTooltip(
    label: &'static str,
    note: &'static str,
    delay: u64,
    #[props(default = 0)] close_delay: u64,
) -> Element {
    rsx! {
        Tooltip { delay, close_delay,
            TooltipTrigger {
                as: move |el: AsChild| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes: el.attributes, "{label}" }
                },
            }
            TooltipContent { "{note}" }
        }
    }
}
