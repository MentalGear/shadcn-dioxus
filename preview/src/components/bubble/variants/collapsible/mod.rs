use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;

const TEXT: &str = "The accessibility review found two focus states that were visually too subtle in dark mode.\n\nI checked the dialog, menu, and drawer paths because each one renders focusable controls inside a layered surface.\n\nThe dialog and drawer are fine. The menu needs the hover and focus tokens split so keyboard focus stays visible when the pointer is not involved.\n\nI also recommend keeping the change in the style file instead of the primitive so the other themes can choose their own focus treatment later.";

const PREVIEW_LENGTH: usize = 180;

/// A long reply folded to a preview. The `Collapsible` sits inside `BubbleContent`; its trigger
/// is rendered as a link-variant `Button` through `as`, with a chevron that turns when open.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let is_long = TEXT.chars().count() > PREVIEW_LENGTH;
    let preview = if is_long {
        format!(
            "{}...",
            TEXT.chars().take(PREVIEW_LENGTH).collect::<String>()
        )
    } else {
        TEXT.to_string()
    };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-8);",
            Bubble { variant: BubbleVariant::Muted,
                BubbleContent { "How can I help you today?" }
            }
            Bubble { variant: BubbleVariant::Muted, align: BubbleAlign::End,
                BubbleContent { style: "white-space: pre-line;",
                    Collapsible {
                        open: Some(open()),
                        on_open_change: move |next: bool| open.set(next),
                        if !open() {
                            div { "{preview}" }
                        }
                        CollapsibleContent {
                            div { "{TEXT}" }
                        }
                        if is_long {
                            CollapsibleTrigger {
                                as: move |attributes: Vec<Attribute>| rsx! {
                                    Button {
                                        variant: ButtonVariant::Link,
                                        r#type: "button",
                                        style: "gap: 0.25rem; padding: 0; color: var(--dx-muted-foreground);",
                                        attributes,
                                        if open() {
                                            "Show less"
                                        } else {
                                            "Show more"
                                        }
                                        if open() {
                                            ChevronDown { style: "rotate: 180deg;" }
                                        } else {
                                            ChevronDown {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
