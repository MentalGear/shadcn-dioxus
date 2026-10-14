use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: the label, the value and the bar under `dir="rtl"`. The label sits at the inline start, the
/// value at the inline end, and the determinate fill grows from the inline start edge, so all three mirror.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl", style: "width: 100%; max-width: 24rem;",
            DirectionProvider { direction: Direction::Rtl,
                Progress { value: 66.0,
                    ProgressLabel { "جارٍ الرفع..." }
                    ProgressValue {}
                }
            }
        }
    }
}
