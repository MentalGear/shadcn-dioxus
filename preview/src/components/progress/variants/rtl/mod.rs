use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: the label-and-readout row and the bar under `dir="rtl"`. The
/// determinate fill grows from the inline start edge, so it mirrors with the
/// rest of the layout.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 24rem;",
                    div { style: "display: flex; justify-content: space-between; font-size: 14px; font-weight: 500;",
                        span { id: "progress-rtl-label", "جارٍ الرفع..." }
                        span { style: "color: var(--dx-muted-foreground); font-variant-numeric: tabular-nums;", "66%" }
                    }
                    Progress { aria_labelledby: "progress-rtl-label", value: 66.0 }
                }
            }
        }
    }
}
