use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    message::{Message, MessageAlign, MessageContent},
    toggle_group::{ToggleGroup, ToggleItem},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowUp, MessageCircleDashed, RotateCw};
use std::collections::HashSet;

/// `(id, is_user, text)`: a scripted conversation sent one turn per press.
const SCRIPT: [(&str, bool, &str); 6] = [
    (
        "anchor-1-user",
        true,
        "Can you show me how anchoring behaves when a new prompt starts the turn?",
    ),
    (
        "anchor-1-assistant",
        false,
        "Append the user prompt first, then append the assistant response. With User selected, the prompt settles near the top and the assistant response fills in below it.",
    ),
    (
        "anchor-2-user",
        true,
        "What changes when assistant messages are the anchor?",
    ),
    (
        "anchor-2-assistant",
        false,
        "Now each assistant response is the item `MessageScroller` keeps in view. This is useful when the reply is the moment you want readers to land on after each turn.",
    ),
    (
        "anchor-3-user",
        true,
        "Can I switch roles and keep adding turns?",
    ),
    (
        "anchor-3-assistant",
        false,
        "Yes. The next appended message with the selected role becomes the anchor, so you can compare user and assistant anchoring without resetting the demo.",
    ),
];

/// `scroll_anchor` is a property of a row, not of a role. This demo derives it from the role the
/// toggle selects: appended rows of that role settle near the top, the others do not.
#[component]
pub fn Demo() -> Element {
    // 0 = User, 1 = Assistant: the index of the selected `ToggleItem`.
    let mut anchor_role = use_signal(|| 0usize);
    let mut sent = use_signal(|| 0usize);
    let next = SCRIPT.get(sent());
    let anchor_is_user = anchor_role() == 0;

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            Card { style: "height: 30rem; gap: 0;",
                CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                    CardTitle { "Anchoring Turns" }
                    CardDescription { "Choose which role settles near the top edge." }
                    CardAction {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Icon,
                            r#type: "button",
                            aria_label: "Reset anchored turns",
                            disabled: sent() == 0,
                            onclick: move |_| sent.set(0),
                            RotateCw {}
                        }
                    }
                }
                CardContent { style: "flex: 1; min-height: 0; padding: 0;",
                    if sent() == 0 {
                        Empty { style: "height: 100%;",
                            EmptyHeader {
                                EmptyMedia { variant: EmptyMediaVariant::Icon,
                                    MessageCircleDashed {}
                                }
                                EmptyTitle { "No anchored messages yet" }
                                EmptyDescription { "Send the first message to see the selected role anchor." }
                            }
                        }
                    } else {
                        MessageScrollerProvider {
                            MessageScroller {
                                MessageScrollerViewport { "aria-label": "Anchored conversation",
                                    MessageScrollerContent { style: "padding: var(--dx-card-spacing);",
                                        for (id , mine , text) in SCRIPT.iter().copied().take(sent()) {
                                            MessageScrollerItem {
                                                key: "{id}-{anchor_role}",
                                                message_id: id,
                                                scroll_anchor: mine == anchor_is_user,
                                                Message { align: if mine { MessageAlign::End } else { MessageAlign::Start },
                                                    MessageContent {
                                                        Bubble { variant: if mine { BubbleVariant::Muted } else { BubbleVariant::Ghost },
                                                            BubbleContent { "{text}" }
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
                }
                CardFooter { style: "gap: var(--dx-space-2);",
                    ToggleGroup {
                        horizontal: true,
                        role: "group",
                        aria_label: "Select scroll anchor role",
                        pressed: Some(HashSet::from([anchor_role()])),
                        on_pressed_change: move |pressed: HashSet<usize>| {
                            // The group is single-select; pressing the active item again would
                            // empty it, so keep the current role in that case.
                            if let Some(&role) = pressed.iter().next() {
                                if role != anchor_role() {
                                    anchor_role.set(role);
                                    sent.set(0);
                                }
                            }
                        },
                        ToggleItem { index: 0usize, aria_label: "Anchor user messages", "User" }
                        ToggleItem { index: 1usize, aria_label: "Anchor assistant messages", "Assistant" }
                    }
                    Button {
                        size: ButtonSize::Icon,
                        r#type: "button",
                        style: "margin-inline-start: auto;",
                        aria_label: "Send Message",
                        disabled: next.is_none(),
                        onclick: move |_| sent += 1,
                        ArrowUp {}
                    }
                }
            }
            p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                "Toggle the anchor role, then send messages to compare where turns settle."
            }
        }
    }
}
