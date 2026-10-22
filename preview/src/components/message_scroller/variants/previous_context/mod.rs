use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    form::DemoForm,
    input_group::{InputGroup, InputGroupAddon, InputGroupAlign, InputGroupInput},
    message::{Message, MessageAlign, MessageContent},
    slider::Slider,
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowUp, RotateCw};

const DEFAULT_PEEK: f64 = 64.0;

/// The earlier conversation, already in the thread when the demo opens: enough history that the
/// thread scrolls, so the "previous context" a new turn leaves visible above it is real text.
const INITIAL: [(bool, &str); 6] = [
    (true, "Which scroll library should I use for a chat UI?"),
    (
        false,
        "You don't need one. Wrap the message list in `MessageScroller`: it handles pinning to the newest message, loading older history and jumping back to the end.\n\nThe rest of this thread shows what happens when you send a new message.",
    ),
    (true, "Does it keep my place when older messages load above?"),
    (
        false,
        "Yes. Prepended rows are measured and the viewport is moved by their height, so whatever you were reading stays exactly where it was on screen.",
    ),
    (
        true,
        "I'm building a chat for our app and the scroll behavior is driving me nuts. Every time the AI streams a reply, the whole thread jumps around.",
    ),
    (
        false,
        "That's the classic streaming scroll problem. Turn on `autoScroll` -- the viewport pins to the bottom as tokens arrive, so users always see the latest text land in place.\n\nThe important part: it only auto-scrolls while the reader is already at the bottom. The moment they scroll up to read something earlier, auto-scroll backs off and their position is preserved.",
    ),
];

/// The queued turns: the composer shows the next user message, sending it streams the reply.
const QUEUE: [(&str, &str); 3] = [
    (
        "Okay, but when someone sends a new message the view still feels jarring -- like the whole conversation reloads from the top.",
        "MessageScrollerItem fixes that with turn anchoring. Set `scrollAnchor` on the turn that should settle near the top instead of blindly snapping to the document bottom.\n\nIt also leaves a small peek of the previous exchange visible above the anchor, so context isn't lost. The reply starts in view without that disorienting jump you get from a plain overflow container.",
    ),
    (
        "And if they've scrolled up to re-read an older answer? I don't want to yank them back down.",
        "You won't. Auto-scroll only runs when the viewport is already pinned to the bottom, so scrolling up is a deliberate opt-out -- their place in the thread stays put even as new tokens keep arriving below.\n\nWhen there is content they haven't seen yet, `MessageScrollerButton` appears at the bottom of the viewport. One tap jumps them back to the newest message and re-engages auto-scroll.",
    ),
    (
        "Last one -- does this work with assistive tech?",
        "`MessageScrollerContent` sets `role=\"log\"` and `aria-relevant=\"additions\"` by default, so screen readers announce new messages as they stream in.\n\nThe scroll button is a real `<button>` with an sr-only label, and it's removed from the tab order when you're already at the bottom -- no ghost focus stops.",
    ),
];

#[derive(Clone, PartialEq)]
struct Turn {
    id: u32,
    mine: bool,
    text: String,
}

fn initial_turns() -> Vec<Turn> {
    INITIAL
        .iter()
        .enumerate()
        .map(|(i, (mine, text))| Turn {
            id: i as u32,
            mine: *mine,
            text: text.to_string(),
        })
        .collect()
}

/// A real timer for the simulated stream (this demo has no async runtime of its own; the
/// browser's `setTimeout` is the clock).
async fn sleep_ms(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

/// `scroll_previous_item_peek` is how many pixels of the previous row stay visible above a newly
/// anchored turn. The slider writes it; send the next message to see the new peek.
#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(initial_turns);
    let mut next_id = use_signal(|| INITIAL.len() as u32);
    let mut queued = use_signal(|| 0usize);
    let mut busy = use_signal(|| false);
    let mut peek = use_signal(|| DEFAULT_PEEK);
    let mut demo_key = use_signal(|| 0u32);
    let next = QUEUE.get(queued()).map(|(user, _)| *user);

    let mut send = move || {
        let Some(&(user, reply)) = QUEUE.get(queued()) else {
            return;
        };
        if busy() {
            return;
        }
        busy.set(true);
        queued += 1;
        spawn(async move {
            let user_id = next_id();
            let reply_id = user_id + 1;
            next_id.set(user_id + 2);
            turns.write().push(Turn {
                id: user_id,
                mine: true,
                text: user.to_string(),
            });
            sleep_ms(400).await;
            turns.write().push(Turn {
                id: reply_id,
                mine: false,
                text: String::new(),
            });
            for word in reply.split(' ') {
                sleep_ms(25).await;
                if let Some(turn) = turns.write().iter_mut().find(|t| t.id == reply_id) {
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
        MessageScrollerProvider {
            key: "{demo_key}",
            scroll_margin: 24.0,
            scroll_previous_item_peek: peek(),
            div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
                Card { style: "height: 32rem; gap: 0;",
                    CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                        CardTitle { "Keeping Context Visible" }
                        CardDescription { "A new turn settles near the top and keeps the end of the previous reply in view." }
                        CardAction {
                            Tooltip {
                                TooltipTrigger {
                                    as: move |attributes: Vec<Attribute>| rsx! {
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            size: ButtonSize::Icon,
                                            r#type: "button",
                                            aria_label: "Reset context example",
                                            disabled: busy(),
                                            onclick: move |_| {
                                                turns.set(initial_turns());
                                                next_id.set(INITIAL.len() as u32);
                                                queued.set(0);
                                                peek.set(DEFAULT_PEEK);
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
                        MessageScroller {
                            MessageScrollerViewport { "aria-label": "Previous context conversation",
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
                    CardFooter { style: "flex-direction: column; align-items: stretch; gap: var(--dx-space-3);",
                        div { style: "display: flex; align-items: center; gap: var(--dx-space-3);",
                            span { style: "flex: none; min-width: 2.5rem; font-size: var(--dx-text-xs); font-variant-numeric: tabular-nums; color: var(--dx-muted-foreground);",
                                "data-testid": "peek-value",
                                "{peek():.0}px"
                            }
                            div { style: "flex: 1; min-width: 0;",
                                Slider {
                                    label: "Previous context peek",
                                    horizontal: true,
                                    min: 64.0,
                                    max: 128.0,
                                    step: 1.0,
                                    default_value: DEFAULT_PEEK,
                                    value: Some(peek()),
                                    disabled: busy(),
                                    on_value_change: move |value: f64| peek.set(value),
                                }
                            }
                        }
                        DemoForm {
                            id: "previous-context-composer",
                            show_status: false,
                            onsubmit: move |_: FormEvent| send(),
                            InputGroup {
                                InputGroupInput {
                                    "aria-label": "Next message",
                                    readonly: true,
                                    value: next.unwrap_or("No messages queued. Reset the context."),
                                }
                                InputGroupAddon { align: InputGroupAlign::End,
                                    Button {
                                        r#type: "submit",
                                        size: ButtonSize::IconXs,
                                        aria_label: "Send",
                                        disabled: next.is_none() || busy(),
                                        ArrowUp {}
                                    }
                                }
                            }
                        }
                    }
                }
                p { style: "margin: 0; text-align: center; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                    "Send a message: the new turn settles near the top, and the last {peek():.0}px of the previous reply stays visible above it. Move the slider, reset and send again to compare."
                }
            }
        }
    }
}
