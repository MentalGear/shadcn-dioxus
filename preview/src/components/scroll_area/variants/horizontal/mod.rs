use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;

const AVATAR_CARDS: [(Asset, &str); 5] = [
    (asset!("/assets/avatars/nina-hart.svg"), "Nina Hart"),
    (asset!("/assets/avatars/sarah-chen.svg"), "Sarah Chen"),
    (asset!("/assets/avatars/priya-nair.svg"), "Priya Nair"),
    (asset!("/assets/avatars/marcus-wright.svg"), "Marcus Wright"),
    (asset!("/assets/avatars/avery-lin.svg"), "Avery Lin"),
];

#[component]
pub fn Demo() -> Element {
    rsx! {
        ScrollArea {
            width: "24rem",
            max_width: "100%",
            border: "1px solid var(--dx-border)",
            border_radius: "0.5em",
            padding: "1em",
            direction: ScrollDirection::Horizontal,
            always_show_scrollbars: true,
            tabindex: "0",
            div { style: "display: flex; gap: 1rem; width: max-content;",
                for (src, name) in AVATAR_CARDS {
                    figure { style: "margin: 0; width: 9rem; flex: none;",
                        img {
                            style: "display: block; width: 9rem; height: 12rem; object-fit: cover; border-radius: var(--dx-radius-md);",
                            src,
                            alt: "Avatar of {name}",
                        }
                        figcaption { style: "margin-top: 0.5rem; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}
