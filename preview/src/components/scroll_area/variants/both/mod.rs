use super::super::component::*;
use dioxus::prelude::*;

const ROWS: [u32; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

/// `ScrollArea` with no `direction`: the default, `ScrollDirection::Both`. The content is wider and taller
/// than the box, so both scrollbars appear and the area scrolls on both axes.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            ScrollArea {
                width: "100%",
                height: "12rem",
                border: "1px solid var(--dx-border)",
                border_radius: "0.5em",
                padding: "1em",
                tabindex: "0",
                "aria-label": "Seating chart",
                div { style: "width: 40rem;",
                    for row in ROWS {
                        div { style: "display: flex; gap: 0.5rem; margin-bottom: 0.5rem;",
                            for seat in 1..=10 {
                                span { style: "flex: none; width: 3.5rem; padding: 0.25rem 0; text-align: center; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-md); font-size: var(--dx-text-sm);",
                                    "{row}-{seat}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
