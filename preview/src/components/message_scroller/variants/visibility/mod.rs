use super::super::component::ScrollBehavior;
use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonVariant},
    card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    message::{Message, MessageAlign, MessageContent},
};
use dioxus::prelude::*;

/// `(id, is_user, text)`. Each user message is a turn anchor.
const CHAT: [(&str, bool, &str); 8] = [
    ("visibility-q1", true, "How do I read where the reader is in a long thread?"),
    (
        "visibility-a1",
        false,
        "Subscribe to the scroller's visibility. It reports the current turn and the rows in view, and it only reports when they change.\n\nNothing is observed until the first subscriber mounts, so a transcript that never asks pays nothing.",
    ),
    ("visibility-q2", true, "What counts as the current turn?"),
    (
        "visibility-a2",
        false,
        "The last user message whose top has reached the reading line: the top of the viewport plus the scroll margin plus the previous-item peek.\n\nIt stays current after it scrolls off the top, until the next turn arrives, so a table of contents does not flicker between turns.",
    ),
    ("visibility-q3", true, "And the visible rows?"),
    (
        "visibility-a3",
        false,
        "Every row with a message id that is at least partly inside the viewport and below the reading line, in document order.\n\nA row hidden under the peek of the previous turn is not listed, which matches what the reader can actually read.",
    ),
    ("visibility-q4", true, "Can I build a jump menu from it?"),
    (
        "visibility-a4",
        false,
        "Yes: highlight the current turn in a list of user messages and call scroll_to_message from each entry.\n\nBecause the state is a signal, any component under the provider can read it.",
    ),
];

/// The readout: it lives inside the provider, so it reads the same controller as the transcript.
#[component]
fn Readout() -> Element {
    let visibility = use_message_scroller_visibility();
    let state = visibility();
    let current = state
        .current_anchor_id
        .as_deref()
        .and_then(|id| CHAT.iter().position(|(row, _, _)| *row == id))
        .map(|i| format!("Turn {}", i / 2 + 1))
        .unwrap_or_else(|| "No turn yet".to_string());
    let rows = state.visible_message_ids.len();
    rsx! {
        span { role: "status", "data-testid": "visibility-readout",
            "{current}, {rows} rows in view"
        }
    }
}

/// Two jumps to the same turn, one with the provider's margin and one with a 24px margin for this
/// call only. Both are `Instant`, so they land at once whatever the viewport's CSS
/// `scroll-behavior` says.
#[component]
fn JumpButtons() -> Element {
    let scroller = use_message_scroller();
    let jump = move |scroll_margin: Option<f64>| {
        scroller.scroll_to_message(
            "visibility-q3",
            ScrollToMessageOptions {
                align: ScrollAlign::Start,
                behavior: ScrollBehavior::Instant,
                scroll_margin,
            },
        );
    };
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            r#type: "button",
            onclick: move |_| jump(None),
            "Jump to turn 3"
        }
        Button {
            variant: ButtonVariant::Outline,
            r#type: "button",
            onclick: move |_| jump(Some(24.0)),
            "Jump with margin"
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        MessageScrollerProvider { default_scroll_position: DefaultScrollPosition::Start,
            div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
                Card { style: "height: 30rem; gap: 0;",
                    CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                        CardTitle { "Visibility" }
                        CardDescription {
                            Readout {}
                        }
                    }
                    CardContent { style: "flex: 1; min-height: 0; padding: 0;",
                        MessageScroller {
                            MessageScrollerViewport { "aria-label": "Visibility conversation",
                                MessageScrollerContent { style: "padding: var(--dx-card-spacing);",
                                    for (id , mine , text) in CHAT {
                                        MessageScrollerItem {
                                            key: "{id}",
                                            message_id: id,
                                            scroll_anchor: mine,
                                            Message { align: if mine { MessageAlign::End } else { MessageAlign::Start },
                                                MessageContent {
                                                    Bubble { variant: if mine { BubbleVariant::Muted } else { BubbleVariant::Ghost },
                                                        BubbleContent { style: "display: flex; flex-direction: column; gap: var(--dx-space-2);",
                                                            for (i , paragraph) in text.split("\n\n").enumerate() {
                                                                p { key: "{i}", style: "margin: 0; white-space: pre-wrap;",
                                                                    "{paragraph}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            MessageScrollerButton {}
                        }
                    }
                    CardFooter { style: "gap: var(--dx-space-2);",
                        JumpButtons {}
                    }
                }
                p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                    "Scroll the conversation; the header follows the turn you are reading."
                }
            }
        }
    }
}
