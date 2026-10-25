use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    message::{Message, MessageAlign, MessageContent},
    tabs::{TabList, TabTrigger, Tabs},
};
use dioxus::prelude::*;

/// `(id, is_user, text)`. The last user turn (`open-3`) is where `LastAnchor` opens.
const CHAT: [(&str, bool, &str); 4] = [
    (
        "open-1",
        true,
        "This is the first message the user sent in the conversation.",
    ),
    (
        "open-2",
        false,
        "Workspace creation rose 8%, but first invite completion only rose 2%.",
    ),
    (
        "open-3",
        true,
        "This is the last message the user sent in the conversation.",
    ),
    (
        "open-4",
        false,
        "Start with the invite step. Teams are creating workspaces but waiting to add collaborators.\n\nRecommended follow-up:\n\n1. Compare invite drop-off by account size.\n2. Check whether users who skip invites still return within 24 hours.\n3. Review the empty-state copy on the first project screen.\n4. Segment activation by template, since template users may not need invites right away.\n\nIf that pattern holds, the next experiment should make collaboration useful earlier instead of prompting for invites harder.",
    ),
];

fn position_of(value: &str) -> DefaultScrollPosition {
    match value {
        "start" => DefaultScrollPosition::Start,
        "end" => DefaultScrollPosition::End,
        _ => DefaultScrollPosition::LastAnchor,
    }
}

/// `default_scroll_position` is live: choosing a tab changes the provider's prop, and the scroller
/// re-applies the new opening position to the thread that is already mounted (no remount).
#[component]
pub fn Demo() -> Element {
    let mut position = use_signal(|| "last-anchor".to_string());

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            Card { style: "height: 30rem; gap: 0;",
                CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                    CardTitle { "Opening Position" }
                    CardDescription { "Choose where a saved transcript opens." }
                }
                CardContent { style: "flex: 1; min-height: 0; padding: 0;",
                    MessageScrollerProvider {
                        default_scroll_position: position_of(&position()),
                        MessageScroller {
                            MessageScrollerViewport { "aria-label": "Opening position conversation",
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
                }
                CardFooter { style: "justify-content: center;",
                    Tabs {
                        value: Some(position()),
                        on_value_change: move |value: String| {
                            position.set(value);
                        },
                        horizontal: true,
                        style: "width: 100%;",
                        TabList { style: "width: 100%;",
                            TabTrigger { value: "start".to_string(), "start" }
                            TabTrigger { value: "end".to_string(), "end" }
                            TabTrigger { value: "last-anchor".to_string(), "last-anchor" }
                        }
                    }
                }
            }
            p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                "Toggle the default_scroll_position to see where the transcript starts when you open the thread."
            }
        }
    }
}
