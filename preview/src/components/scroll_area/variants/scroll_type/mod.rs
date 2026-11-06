use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::{ScrollDirection, ScrollType};

const ITEMS: [&str; 8] = ["Inbox", "Drafts", "Sent", "Archive", "Spam", "Trash", "Notes", "Later"];

/// `scroll_type` sets when the scrollbar is painted: `Auto` (the browser's own rule, the default), `Always`
/// (the bar stays visible at rest) and `Hidden` (no bar, but the list still scrolls with the wheel, touch and keys).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 1rem; width: 100%;",
            for (label, scroll_type) in [("Auto", ScrollType::Auto), ("Always", ScrollType::Always), ("Hidden", ScrollType::Hidden)] {
                div { style: "display: flex; flex-direction: column; gap: 0.5rem; flex: 1 1 8rem;",
                    p { style: "margin: 0; font-size: var(--dx-text-sm); font-weight: 500;", "{label}" }
                    ScrollArea {
                        height: "10rem",
                        padding: "0.5rem 0.75rem",
                        border: "1px solid var(--dx-border)",
                        border_radius: "var(--dx-radius-lg)",
                        direction: ScrollDirection::Vertical,
                        scroll_type,
                        tabindex: "0",
                        "aria-label": "{label} folders",
                        div {
                            for item in ITEMS {
                                p { style: "margin: 0 0 0.5rem; font-size: var(--dx-text-sm);", "{item}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
