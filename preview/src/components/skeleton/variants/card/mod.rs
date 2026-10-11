use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's "card" loading layout: a 250px image block with two text lines
/// beneath it, the way a product card looks before its data arrives.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 15.625rem;",
            Skeleton { style: "width: 15.625rem; height: 7.8125rem; border-radius: 0.75rem;" }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                Skeleton { style: "width: 15.625rem; height: 1rem;" }
                Skeleton { style: "width: 12.5rem; height: 1rem;" }
            }
        }
    }
}
