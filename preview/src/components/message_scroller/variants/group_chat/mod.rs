use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    marker::{Marker, MarkerContent, MarkerVariant},
    message::{Message, MessageAlign, MessageContent, MessageHeader},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::RotateCw;

const CURRENT_USER: &str = "Grace";

#[derive(Clone, Copy, PartialEq)]
enum Item {
    /// A system event (a join marker). `scroll_anchor` makes it start a turn.
    Event {
        id: &'static str,
        text: &'static str,
        scroll_anchor: bool,
    },
    Message {
        id: &'static str,
        sender: &'static str,
        assistant: bool,
        text: &'static str,
        scroll_anchor: bool,
    },
}

const INITIAL: [Item; 3] = [
    Item::Message {
        id: "group-1",
        sender: "Grace",
        assistant: false,
        text: "@mary, the astrophage line keeps matching Venus energy output. Can you check my math?",
        scroll_anchor: false,
    },
    Item::Message {
        id: "group-2",
        sender: "Mary (Agent)",
        assistant: true,
        text: "Yes. Confirmed. The curve points to a microorganism harvesting stellar energy and breeding near carbon dioxide. If @rocky agrees, this is the clue we need.",
        scroll_anchor: false,
    },
    Item::Message {
        id: "group-3",
        sender: "Grace",
        assistant: false,
        text: "ping @rocky",
        scroll_anchor: true,
    },
];

const ROCKY_MARKER: Item = Item::Event {
    id: "group-4",
    text: "Rocky has joined the chat",
    scroll_anchor: true,
};

const ROCKY_MESSAGE: Item = Item::Message {
    id: "group-5",
    sender: "Rocky",
    assistant: false,
    text: "Amaze. Astrophage eats light, makes heat, goes to carbon dioxide. Rocky has fuel model. Grace is smart.",
    scroll_anchor: false,
};

/// A join `Marker` is a row like any other, so it can carry `scroll_anchor`: when Rocky joins, the
/// marker settles near the top and the reply that follows fills in below it.
#[component]
pub fn Demo() -> Element {
    // 0 = idle, 1 = Rocky's join marker added, 2 = Rocky's reply added.
    let mut rocky_turn = use_signal(|| 0u8);
    let mut demo_key = use_signal(|| 0u32);
    let mut items = INITIAL.to_vec();
    if rocky_turn() >= 1 {
        items.push(ROCKY_MARKER);
    }
    if rocky_turn() >= 2 {
        items.push(ROCKY_MESSAGE);
    }

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            Card { style: "height: 30rem; gap: 0;",
                CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                    CardTitle { "Group Chat" }
                    CardDescription {
                        "A group chat with several participants and an assistant. The Marker is marked as a turn."
                    }
                    CardAction {
                        Tooltip {
                            TooltipTrigger {
                                as: move |attributes: Vec<Attribute>| rsx! {
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Icon,
                                        r#type: "button",
                                        aria_label: "Reset conversation",
                                        disabled: rocky_turn() == 0,
                                        onclick: move |_| {
                                            rocky_turn.set(0);
                                            demo_key += 1;
                                        },
                                        attributes,
                                        RotateCw {}
                                    }
                                },
                            }
                            TooltipContent { "Reset" }
                        }
                    }
                }
                CardContent { style: "flex: 1; min-height: 0; padding: 0;",
                    MessageScrollerProvider { key: "{demo_key}",
                        MessageScroller {
                            MessageScrollerViewport { "aria-label": "Group chat",
                                MessageScrollerContent { style: "padding: var(--dx-card-spacing);",
                                    for item in items {
                                        match item {
                                            Item::Event { id , text , scroll_anchor } => rsx! {
                                                MessageScrollerItem { key: "{id}", message_id: id, scroll_anchor,
                                                    Marker { variant: MarkerVariant::Separator,
                                                        MarkerContent { "{text}" }
                                                    }
                                                }
                                            },
                                            Item::Message { id , sender , assistant , text , scroll_anchor } => rsx! {
                                                MessageScrollerItem { key: "{id}", message_id: id, scroll_anchor,
                                                    Message { align: if sender == CURRENT_USER { MessageAlign::End } else { MessageAlign::Start },
                                                        MessageContent {
                                                            if sender != CURRENT_USER {
                                                                MessageHeader { "{sender}" }
                                                            }
                                                            Bubble {
                                                                variant: if sender == CURRENT_USER {
                                                                    BubbleVariant::Muted
                                                                } else if assistant {
                                                                    BubbleVariant::Ghost
                                                                } else {
                                                                    BubbleVariant::Tinted
                                                                },
                                                                BubbleContent { "{text}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                            MessageScrollerButton {}
                        }
                    }
                }
                CardFooter { style: "flex-direction: column; align-items: stretch; gap: var(--dx-space-2);",
                    Button {
                        variant: ButtonVariant::Secondary,
                        r#type: "button",
                        disabled: rocky_turn() == 2,
                        onclick: move |_| rocky_turn += 1,
                        if rocky_turn() == 0 {
                            "Add Rocky"
                        } else {
                            "Send Message as Rocky"
                        }
                    }
                    p { style: "margin: 0; text-align: center; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                        if rocky_turn() == 0 {
                            "This will create a marker and make it the anchor"
                        } else {
                            "Now send Rocky's reply into the conversation"
                        }
                    }
                }
            }
            p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                "When a user joins, a marker is created. scroll_anchor on the marker marks it as the next turn."
            }
        }
    }
}
