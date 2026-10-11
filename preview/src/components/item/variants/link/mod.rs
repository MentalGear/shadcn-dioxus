use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronRight, ExternalLink};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "1rem",
            width: "100%",
            max_width: "28rem",

            Item {
                as: move |attrs: Vec<Attribute>| rsx! {
                    a { href: "#", ..attrs,
                        ItemContent {
                            ItemTitle { "Visit our documentation" }
                            ItemDescription { "Learn how to get started with our components." }
                        }
                        ItemActions {
                            ChevronRight { size: "16" }
                        }
                    }
                },
            }

            Item {
                variant: ItemVariant::Outline,
                as: move |attrs: Vec<Attribute>| rsx! {
                    a { href: "#", ..attrs,
                        ItemContent {
                            ItemTitle { "External resource" }
                            ItemDescription { "Opens in a new tab with extra information." }
                        }
                        ItemActions {
                            ExternalLink { size: "16" }
                        }
                    }
                },
            }
        }
    }
}
