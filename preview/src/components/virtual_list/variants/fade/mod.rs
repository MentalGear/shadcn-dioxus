use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

const MANY: usize = 20_000;
const COUNTS: [usize; 2] = [MANY, 3];

fn row(idx: usize) -> Element {
    rsx! {
        article { style: "padding: 0.75rem 0.9rem; border: 1px solid var(--dx-border); border-radius: 0.625rem; background: var(--dx-card);",
            p { style: "margin: 0; color: var(--dx-foreground); font-size: 0.95rem; font-weight: 660;",
                "Row {idx + 1}"
            }
            p { style: "margin: 0.2rem 0 0; color: var(--dx-muted-foreground); font-size: 0.875rem;",
                "Index {idx}"
            }
        }
    }
}

/// A demo caption: a title line and a one-sentence description.
#[component]
fn Caption(title: &'static str, children: Element) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.125rem;",
            p { style: "margin: 0; color: var(--dx-foreground); font-size: var(--dx-text-sm); font-weight: 500;",
                "{title}"
            }
            p { style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);",
                {children}
            }
        }
    }
}

/// `VirtualList`'s `fade` prop on both modes. The fade is a mask on the list's own scroll
/// container, driven by its scroll offset, so it follows the scrollbar whichever rows are
/// mounted: the windowed list below holds 20,000 rows but mounts a few dozen. Switch to 3 rows
/// and the fade disappears (nothing to scroll). The border sits on a wrapper so the mask
/// dissolves the rows and not the card.
#[component]
pub fn Demo() -> Element {
    let mut rows = use_signal(|| MANY);

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; flex-direction: column; gap: var(--dx-space-4);",
            Caption {
                title: "Fade top and bottom, windowed",
                "Only the rows in view are mounted, and the fade still tracks the scroll position."
            }
            div {
                "data-demo": "fade-windowed",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                VirtualList {
                    count: rows,
                    buffer: 12usize,
                    virtualize: VirtualizeMode::Windowed,
                    fade: ScrollFade::Y,
                    estimate_size: |_| 68,
                    style: "position: relative; max-height: 18rem; overflow-y: auto;",
                    "aria-label": "Rows, windowed",
                    render_item: move |idx: usize| row(idx),
                }
            }
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                for count in COUNTS {
                    Button {
                        key: "{count}",
                        variant: if rows() == count { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                        "aria-pressed": (rows() == count).to_string(),
                        onclick: move |_| rows.set(count),
                        "{count} rows"
                    }
                }
            }
            Caption {
                title: "Fade top and bottom, content-visibility",
                "Every row is in the DOM and the browser skips the off-screen ones; the fade is the same."
            }
            div {
                "data-demo": "fade-content-visibility",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                VirtualList {
                    count: 300usize,
                    virtualize: VirtualizeMode::ContentVisibility,
                    fade: ScrollFade::Y,
                    estimate_size: |_| 68,
                    style: "position: relative; max-height: 18rem; overflow-y: auto;",
                    "aria-label": "Rows, content-visibility",
                    render_item: move |idx: usize| row(idx),
                }
            }
        }
    }
}
