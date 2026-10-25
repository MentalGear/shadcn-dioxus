use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    form::DemoForm,
    input_group::{InputGroup, InputGroupAddon, InputGroupAlign, InputGroupInput},
    message::{Message, MessageAlign, MessageContent},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{
    ArrowUp, Globe, Image, MessageCircleDashed, Paperclip, Plus, RotateCw, Telescope,
};

/// The queued turns: the composer shows the next user message; sending it streams the reply.
const QUEUE: [(&str, &str); 4] = [
    (
        "I'm building a chat for our app and the scroll behavior is driving me nuts. Every time the AI streams a reply, the whole thread jumps around.",
        "That's the classic streaming scroll problem. Wrap your message list in `MessageScroller` and turn on `auto_scroll` -- the viewport pins to the bottom as tokens arrive, so users always see the latest text land in place.\n\nThe important part: it only auto-scrolls while the reader is already at the bottom. The moment they scroll up to read something earlier, auto-scroll backs off and their position is preserved. You get smooth streaming without fighting the user's intent.",
    ),
    (
        "Okay, but when someone sends a new message the view still feels jarring -- like the whole conversation reloads from the top.",
        "MessageScrollerItem fixes that with turn anchoring. Set `scroll_anchor` on the turn that should settle near the top instead of blindly snapping to the bottom.\n\nIt also leaves a small peek of the previous exchange visible above the anchor, so context isn't lost. The reply starts in view without that disorienting jump you get from a plain overflow container.",
    ),
    (
        "And if they've scrolled up to re-read an older answer? I don't want to yank them back down.",
        "You won't. Auto-scroll only runs when the viewport is already pinned to the bottom, so scrolling up is a deliberate opt-out -- their place in the thread stays put even as new tokens keep arriving below.\n\nWhen there is content they haven't seen yet, `MessageScrollerButton` appears at the bottom of the viewport. One tap jumps them back to the newest message and re-engages auto-scroll. Same pattern as Slack or iMessage: quiet when you're caught up, helpful when you're not.",
    ),
    (
        "Last one -- does this work with assistive tech?",
        "`MessageScrollerContent` sets `role=\"log\"` and `aria-relevant=\"additions\"` by default, so screen readers announce new messages as they stream in.\n\nThe scroll button is a real `<button>` with an sr-only label, and it's removed from the tab order when you're already at the bottom -- no ghost focus stops.",
    ),
];

const ATTACH_MENU: [&str; 4] = [
    "Add Photos & Files",
    "Create Image",
    "Deep Research",
    "Web Search",
];

#[derive(Clone, PartialEq)]
struct Turn {
    id: u32,
    mine: bool,
    text: String,
}

/// A real timer for the simulated stream (this demo has no async runtime of its own; the
/// browser's `setTimeout` is the clock).
async fn sleep_ms(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

#[component]
fn MenuIcon(name: &'static str) -> Element {
    match name {
        "Create Image" => rsx! {
            Image { size: "16px" }
        },
        "Deep Research" => rsx! {
            Telescope { size: "16px" }
        },
        "Web Search" => rsx! {
            Globe { size: "16px" }
        },
        _ => rsx! {
            Paperclip { size: "16px" }
        },
    }
}

/// The docs hero, approximated with the parts this library has: a `Card` shell, an `Empty` state
/// before the first message, a `Reset` button with a `Tooltip`, and a composer made of an
/// `InputGroup` with an inline `+` menu on the start side and the send button on the end side. The
/// composer is read only, like shadcn's: it shows the next queued message and send plays it.
#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(Vec::<Turn>::new);
    let mut queued = use_signal(|| 0usize);
    let mut busy = use_signal(|| false);
    let mut chosen = use_signal(|| None::<String>);
    let next = QUEUE.get(queued()).map(|(user, _)| *user);

    let mut send = move || {
        let Some(&(user, reply)) = QUEUE.get(queued()) else {
            return;
        };
        if busy() {
            return;
        }
        busy.set(true);
        let id = queued() as u32 * 2;
        queued += 1;
        spawn(async move {
            turns.write().push(Turn {
                id,
                mine: true,
                text: user.to_string(),
            });
            sleep_ms(400).await;
            turns.write().push(Turn {
                id: id + 1,
                mine: false,
                text: String::new(),
            });
            for word in reply.split(' ') {
                sleep_ms(20).await;
                if let Some(turn) = turns.write().iter_mut().find(|t| t.id == id + 1) {
                    if !turn.text.is_empty() {
                        turn.text.push(' ');
                    }
                    turn.text.push_str(word);
                }
            }
            busy.set(false);
        });
    };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            Card { style: "height: 32rem; gap: 0;",
                CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                    CardTitle { "New Chat" }
                    CardDescription { "How can I help you today?" }
                    CardAction {
                        Tooltip {
                            TooltipTrigger {
                                as: move |attributes: Vec<Attribute>| rsx! {
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Icon,
                                        r#type: "button",
                                        aria_label: "Reset conversation",
                                        disabled: busy(),
                                        onclick: move |_| {
                                            turns.write().clear();
                                            queued.set(0);
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
                    if turns.read().is_empty() {
                        Empty { style: "height: 100%;",
                            EmptyHeader {
                                EmptyMedia { variant: EmptyMediaVariant::Icon,
                                    MessageCircleDashed {}
                                }
                                EmptyTitle { "Morning, shadcn!" }
                                EmptyDescription {
                                    "What are we working on today? Press send to start a new conversation"
                                }
                            }
                        }
                    } else {
                        MessageScrollerProvider {
                            MessageScroller {
                                MessageScrollerViewport { "aria-label": "Chat conversation",
                                    MessageScrollerContent {
                                        "aria-busy": busy(),
                                        style: "padding: var(--dx-card-spacing);",
                                        for turn in turns() {
                                            MessageScrollerItem {
                                                key: "{turn.id}",
                                                message_id: "m{turn.id}",
                                                scroll_anchor: turn.mine,
                                                Message { align: if turn.mine { MessageAlign::End } else { MessageAlign::Start },
                                                    MessageContent {
                                                        Bubble { variant: if turn.mine { BubbleVariant::Muted } else { BubbleVariant::Ghost },
                                                            BubbleContent { style: "display: flex; flex-direction: column; gap: var(--dx-space-2);",
                                                                for (i , paragraph) in turn.text.split("\n\n").enumerate() {
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
                }
                CardFooter { style: "flex-direction: column; align-items: stretch; gap: var(--dx-space-2);",
                    DemoForm {
                        id: "full-chat-composer",
                        show_status: false,
                        onsubmit: move |_: FormEvent| send(),
                        InputGroup {
                            InputGroupAddon { align: InputGroupAlign::Start,
                                DropdownMenu {
                                    DropdownMenuTrigger {
                                        as: move |attributes: Vec<Attribute>| rsx! {
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::IconSm,
                                                r#type: "button",
                                                aria_label: "Add files",
                                                attributes,
                                                Plus {}
                                            }
                                        },
                                    }
                                    DropdownMenuContent {
                                        for name in ATTACH_MENU.into_iter() {
                                            DropdownMenuItem::<String> {
                                                key: "{name}",
                                                value: name.to_string(),
                                                text_value: name,
                                                on_select: move |value: String| chosen.set(Some(value)),
                                                MenuIcon { name }
                                                "{name}"
                                            }
                                        }
                                    }
                                }
                            }
                            InputGroupInput {
                                "aria-label": "Next message",
                                readonly: true,
                                value: next.unwrap_or("No messages queued. Reset the conversation."),
                            }
                            InputGroupAddon { align: InputGroupAlign::End,
                                Button {
                                    r#type: "submit",
                                    size: ButtonSize::IconSm,
                                    aria_label: "Send",
                                    disabled: next.is_none() || busy(),
                                    ArrowUp {}
                                }
                            }
                        }
                    }
                }
            }
            p { role: "status", style: "margin: 0; text-align: center; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                if let Some(choice) = chosen() {
                    "Picked \"{choice}\" from the menu. Demo is read only."
                } else {
                    "Demo is read only. Press send to send messages."
                }
            }
        }
    }
}
