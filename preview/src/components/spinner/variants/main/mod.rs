use super::super::component::*;
use crate::components::item::{Item, ItemContent, ItemMedia, ItemTitle, ItemVariant};
use dioxus::prelude::*;

// shadcn's spinner-demo: a muted `Item` with the spinner as its media, a
// title, and the amount on the right.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Item { variant: ItemVariant::Muted,
                ItemMedia { Spinner {} }
                ItemContent {
                    ItemTitle { "Processing payment..." }
                }
                ItemContent { style: "flex: none; justify-content: flex-end;",
                    span { style: "font-size: 0.875rem; font-variant-numeric: tabular-nums;", "$100.00" }
                }
            }
        }
    }
}
