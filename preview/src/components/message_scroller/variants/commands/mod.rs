use super::super::component::*;
use super::super::component::ScrollBehavior;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle},
    dropdown_menu::{
        DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel,
        DropdownMenuTrigger,
    },
    message::{Message, MessageAlign, MessageContent},
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// `(id, is_user, text)`. Only the user turns appear in the "Jump to..." menu.
const CHAT: [(&str, bool, &str); 8] = [
    (
        "command-activation",
        true,
        "We're seeing activation dip after workspace creation. Can you help me find the likely step?",
    ),
    (
        "command-activation-reply",
        false,
        "The sharpest drop is between creating the workspace and inviting the first teammate.\n\nWorkspace creation is still healthy, but the invite step is where users pause. That suggests the product is asking for collaboration before the user has enough confidence in the workspace.",
    ),
    (
        "command-compare",
        true,
        "What should I compare before we change the onboarding flow?",
    ),
    (
        "command-compare-reply",
        false,
        "Compare three cohorts:\n\n1. Users who choose a template before inviting teammates.\n2. Users who start from a blank workspace.\n3. Users who skip invites and return within 24 hours.\n\nIf template users invite faster, the fix is probably better first-run guidance rather than a louder invite prompt.",
    ),
    (
        "command-experiment",
        true,
        "Can you turn that into an experiment?",
    ),
    (
        "command-experiment-reply",
        false,
        "Yes. Create a variant that shows a short checklist after workspace creation:\n\n- Pick a template.\n- Add one project detail.\n- Invite a teammate when the workspace has context.\n\nMeasure first invite completion, 24-hour return rate, and whether teams create a second project.",
    ),
    (
        "command-risk",
        true,
        "What's the risk if we delay the invite prompt?",
    ),
    (
        "command-risk-reply",
        false,
        "The main risk is reducing team creation for accounts that already know who they want to invite.\n\nTo protect that path, keep the invite action visible in the header and only change the primary empty-state guidance. That gives confident teams a direct route without forcing uncertain users through the invite step too early.",
    ),
];

/// The "Jump to..." menu: it lives inside the provider, so `use_message_scroller()` reaches the
/// same controller as the transcript. Each item calls `scroll_to_message` for one user turn.
#[component]
fn CommandMenu() -> Element {
    let scroller = use_message_scroller();
    rsx! {
        DropdownMenu {
            DropdownMenuTrigger {
                as: move |el: AsChild| rsx! {
                    Button { variant: ButtonVariant::Secondary, r#type: "button", attributes: el.attributes, "Jump to..." }
                },
            }
            DropdownMenuContent {
                DropdownMenuLabel { "Conversations" }
                for (i , (id , _ , text)) in CHAT.iter().filter(|(_, mine, _)| *mine).enumerate() {
                    DropdownMenuItem::<String> {
                        key: "{id}",
                        value: id.to_string(),
                        index: i,
                        text_value: *text,
                        on_select: move |id: String| {
                            scroller
                                .scroll_to_message(
                                    id,
                                    ScrollToMessageOptions {
                                        align: ScrollAlign::Start,
                                        behavior: ScrollBehavior::Smooth,
                                        scroll_margin: None,
                                    },
                                );
                        },
                        span { style: "display: block; max-width: 16rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{trimmed(text)}"
                        }
                    }
                }
            }
        }
    }
}

fn trimmed(text: &str) -> String {
    if text.chars().count() > 42 {
        format!("{}...", text.chars().take(39).collect::<String>())
    } else {
        text.to_string()
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        MessageScrollerProvider { default_scroll_position: DefaultScrollPosition::End,
            div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
                Card { style: "height: 30rem; gap: 0;",
                    CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                        CardTitle { "Commands" }
                        CardDescription { "Drive the transcript from outside." }
                        CardAction {
                            CommandMenu {}
                        }
                    }
                    CardContent { style: "flex: 1; min-height: 0; padding: 0;",
                        MessageScroller {
                            MessageScrollerViewport { "aria-label": "Commands conversation",
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
                p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                    "Use the controls to jump to any message in the conversation."
                }
            }
        }
    }
}
