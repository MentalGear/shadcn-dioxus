use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::ContentSide;

/// `delay` and `close_delay` on `HoverCard` (shadcn's `HoverCardTrigger delay={10} closeDelay={100}`
/// demo and its `delay={100} closeDelay={100}` sides row, next to the default 600 / 300): nothing
/// opens until the pointer has rested for `delay` ms, and the card outlives the pointer leaving by
/// `close_delay` ms, so the pointer can cross onto it. Keyboard focus opens at once whatever the delay.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "padding: 50px; display: flex; flex-wrap: wrap; gap: 40px; justify-content: center; align-items: center;",
            DelayCard {
                label: "Default (600 / 300)",
                note: "Opens after the pointer rests for 600 ms and closes 300 ms after it leaves.",
            }
            DelayCard {
                label: "Quick (10 / 100)",
                delay: 10,
                close_delay: 100,
                note: "shadcn's demo: opens after 10 ms, closes 100 ms after the pointer leaves.",
            }
            DelayCard {
                label: "Instant (0 / 0)",
                delay: 0,
                close_delay: 0,
                note: "Opens on contact and closes the moment the pointer leaves.",
            }
        }
    }
}

#[component]
fn DelayCard(
    label: &'static str,
    note: &'static str,
    #[props(default = 600)] delay: u64,
    #[props(default = 300)] close_delay: u64,
) -> Element {
    rsx! {
        HoverCard { delay, close_delay,
            HoverCardTrigger { "{label}" }
            HoverCardContent { side: ContentSide::Bottom,
                div { style: "padding: 0.5rem; width: 14rem;", "{note}" }
            }
        }
    }
}
