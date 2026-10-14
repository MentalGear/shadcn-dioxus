use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Inbox;

/// shadcn's `item-size`: the three sizes, each an outline `Item` with an icon media, title and description.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "1.5rem",
            width: "100%",
            max_width: "28rem",

            Item { variant: ItemVariant::Outline,
                ItemMedia { variant: ItemMediaVariant::Icon, Inbox {} }
                ItemContent {
                    ItemTitle { "Default Size" }
                    ItemDescription { "The standard size for most use cases." }
                }
            }
            Item { variant: ItemVariant::Outline, size: ItemSize::Sm,
                ItemMedia { variant: ItemMediaVariant::Icon, Inbox {} }
                ItemContent {
                    ItemTitle { "Small Size" }
                    ItemDescription { "A compact size for dense layouts." }
                }
            }
            Item { variant: ItemVariant::Outline, size: ItemSize::Xs,
                ItemMedia { variant: ItemMediaVariant::Icon, Inbox {} }
                ItemContent {
                    ItemTitle { "Extra Small Size" }
                    ItemDescription { "The most compact size available." }
                }
            }
        }
    }
}
