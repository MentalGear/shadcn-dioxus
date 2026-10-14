use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::separator::Separator;
use dioxus::prelude::*;
use dioxus_primitives::direction::Direction;
use dioxus_primitives::scroll_area::ScrollDirection;

const RELEASES: [&str; 14] = [
    "v1.2.0", "v1.1.2", "v1.1.1", "v1.1.0", "v1.0.4", "v1.0.3", "v1.0.2", "v1.0.1", "v1.0.0",
    "v0.9.0", "v0.8.2", "v0.8.1", "v0.8.0", "v0.7.0",
];

const TAGS: [&str; 14] = [
    "Accordion",
    "Alert",
    "Avatar",
    "Badge",
    "Breadcrumb",
    "Button",
    "Calendar",
    "Card",
    "Carousel",
    "Checkbox",
    "Collapsible",
    "Combobox",
    "Command",
    "Dialog",
];

/// The scroll-edge fade on a `ScrollArea` (the `dx-scroll-fade` utility from `dx-effects.css`).
/// The border sits on a wrapper and the fade on the inner scroller, so the mask dissolves the
/// content and not the card. Three cases: a vertical list that fades at the end and, once
/// scrolled, at the start (`ScrollFade::Y`); a horizontal strip that follows the reading
/// direction (`ScrollFade::X`, flip it with the button); and a list too short to scroll, which
/// has no fade at all.
#[component]
pub fn Demo() -> Element {
    let mut rtl = use_signal(|| false);
    let direction = if rtl() { Direction::Rtl } else { Direction::Ltr };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            div {
                "data-demo": "fade-y",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                ScrollArea {
                    height: "12rem",
                    padding: "0.75rem 1rem",
                    direction: ScrollDirection::Vertical,
                    fade: ScrollFade::Y,
                    tabindex: "0",
                    "aria-label": "Releases",
                    div {
                        p { style: "margin: 0 0 0.5rem; font-size: var(--dx-text-sm); font-weight: 500;", "Releases" }
                        for tag in RELEASES {
                            p { style: "margin: 0; font-size: var(--dx-text-sm);", "{tag}" }
                            Separator { decorative: true, style: "margin: 0.5rem 0;" }
                        }
                    }
                }
            }
            div {
                "data-demo": "fade-x",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                ScrollArea {
                    padding: "0.75rem 1rem",
                    direction: ScrollDirection::Horizontal,
                    dir: Some(direction),
                    fade: ScrollFade::X,
                    tabindex: "0",
                    "aria-label": "Components",
                    div { style: "display: flex; gap: 0.5rem; width: max-content;",
                        for name in TAGS {
                            span { style: "flex: none; padding: 0.25rem 0.75rem; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-md); font-size: var(--dx-text-sm);",
                                "{name}"
                            }
                        }
                    }
                }
            }
            div {
                "data-demo": "fade-short",
                style: "border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                ScrollArea {
                    height: "12rem",
                    padding: "0.75rem 1rem",
                    direction: ScrollDirection::Vertical,
                    fade: ScrollFade::Y,
                    tabindex: "0",
                    "aria-label": "Short list",
                    div {
                        for tag in &RELEASES[..3] {
                            p { style: "margin: 0; font-size: var(--dx-text-sm);", "{tag}" }
                            Separator { decorative: true, style: "margin: 0.5rem 0;" }
                        }
                        p { style: "margin: 0; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                            "Nothing to scroll to, so nothing fades."
                        }
                    }
                }
            }
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                Button {
                    variant: ButtonVariant::Outline,
                    "aria-pressed": rtl().to_string(),
                    onclick: move |_| rtl.toggle(),
                    "Right-to-left strip"
                }
            }
        }
    }
}
