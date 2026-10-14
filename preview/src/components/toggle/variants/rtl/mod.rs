use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Italic;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: the same icon + label toggle as the `text` variant, with
/// Arabic copy. Labelled "مائل" (not "B") so `toggle.spec.ts`'s
/// `getByRole("button", { name: "B", exact: true })` still matches only `main`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                Toggle {
                    Italic {}
                    "مائل"
                }
            }
        }
    }
}
