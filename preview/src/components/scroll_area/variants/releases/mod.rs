use super::super::component::*;
use crate::components::separator::Separator;
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;

const RELEASES: [&str; 10] = [
    "v1.2.0", "v1.1.2", "v1.1.1", "v1.1.0", "v1.0.4", "v1.0.3", "v1.0.2", "v1.0.1", "v1.0.0",
    "v0.9.0",
];

#[component]
pub fn Demo() -> Element {
    rsx! {
        ScrollArea {
            width: "12rem",
            height: "12rem",
            border: "1px solid var(--dx-border)",
            border_radius: "0.5em",
            padding: "0.75rem 1rem",
            direction: ScrollDirection::Vertical,
            tabindex: "0",
            div {
                p { style: "margin: 0 0 0.5rem; font-size: var(--dx-text-sm); font-weight: 500;", "Releases" }
                for tag in RELEASES {
                    p { style: "margin: 0; font-size: var(--dx-text-sm);", "{tag}" }
                    Separator { decorative: true, style: "margin: 0.5rem 0;" }
                }
            }
        }
    }
}
