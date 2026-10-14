use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    message::{Message, MessageAlign, MessageContent},
    select::{Select, SelectGroup, SelectOption},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ArrowUp, MessageCircleDashed, RotateCw};

/// `(user message, assistant reply)`; each press of send plays one pair.
const SCRIPT: [(&str, &str); 3] = [
    (
        "Can user messages pop in like iMessage without breaking anchoring?",
        "Yes. Animate the user row with opacity, scale and translate, and let the assistant response stream normally below it.\n\nThat keeps the row measurement predictable while still giving the newly sent bubble a more tactile entrance.",
    ),
    (
        "What makes the animation feel more like iMessage?",
        "Use a quick pop from the trailing edge: a little scale, a small upward move, and no layout animation.\n\nThe bubble feels tactile, but the measured row stays predictable, so anchoring and auto-scroll do not have to fight a changing layout.",
    ),
    (
        "Can I switch between presets while testing the same thread?",
        "Yes. Keep the conversation in place while you change the preset, then send the next message to compare the new entrance against the same context.\n\nThat makes it easier to judge the difference between a subtle fade, a snappy pop, and a small drop without rebuilding the scenario each time.",
    ),
];

/// The presets: `(id, name)`. Each plays one of the theme's shared entrance keyframes
/// (`dx-fade-in`, `dx-pop-in`, `dx-drop-in`), which animate only `opacity`, `scale` and
/// `translate`, so a row's measured height never changes while it plays.
const PRESETS: [(&str, &str); 3] = [("fade", "Fade"), ("pop", "Pop"), ("drop", "Drop")];

#[derive(Clone, PartialEq)]
struct Turn {
    id: u32,
    mine: bool,
    text: String,
    /// The preset that was selected when this row was sent. Stored with the row, so changing the
    /// preset later never replays the entrance of rows already on screen.
    preset: &'static str,
}

/// A real timer for the simulated stream (this demo has no async runtime of its own; the
/// browser's `setTimeout` is the clock).
async fn sleep_ms(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(Vec::<Turn>::new);
    let mut sent = use_signal(|| 0usize);
    let mut busy = use_signal(|| false);
    let mut preset = use_signal(|| "fade");
    let next = SCRIPT.get(sent());

    let mut send = move || {
        let Some(&(user, reply)) = SCRIPT.get(sent()) else {
            return;
        };
        if busy() {
            return;
        }
        busy.set(true);
        let id = sent() as u32 * 2;
        sent += 1;
        let preset_now = preset();
        spawn(async move {
            turns.write().push(Turn {
                id,
                mine: true,
                text: user.to_string(),
                preset: preset_now,
            });
            sleep_ms(400).await;
            turns.write().push(Turn {
                id: id + 1,
                mine: false,
                text: String::new(),
                preset: "",
            });
            for word in reply.split(' ') {
                sleep_ms(15).await;
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
        // Scoped to this demo. The animation lives behind `no-preference`, so reduced motion shows
        // the row straight away. `backwards` holds the first keyframe (transparent) until it starts.
        style { r#"
@media (prefers-reduced-motion: no-preference) {{
  .dx-demo-enter-fade {{
    animation: dx-fade-in 220ms ease-out backwards;
  }}

  .dx-demo-enter-pop {{
    animation: dx-pop-in 240ms var(--dx-motion-ease-out) backwards;
    transform-origin: bottom right;
  }}

  .dx-demo-enter-drop {{
    animation: dx-drop-in 260ms var(--dx-motion-ease-out) backwards;
  }}
}}
"# }
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-4);",
            Card { style: "height: 30rem; gap: 0;",
                CardHeader { style: "padding-bottom: var(--dx-card-spacing); border-bottom: 1px solid var(--dx-border);",
                    CardTitle { "Animation" }
                    CardDescription {
                        "Choose how user messages are animated when they are added to the conversation."
                    }
                    CardAction {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Icon,
                            r#type: "button",
                            aria_label: "Reset animated messages",
                            disabled: turns.read().is_empty() || busy(),
                            onclick: move |_| {
                                turns.write().clear();
                                sent.set(0);
                            },
                            RotateCw {}
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
                                EmptyTitle { "No Messages Yet" }
                                EmptyDescription { "Click the button below to send the first message." }
                            }
                        }
                    } else {
                        MessageScrollerProvider {
                            MessageScroller {
                                MessageScrollerViewport { "aria-label": "Animated conversation",
                                    MessageScrollerContent {
                                        "aria-busy": busy(),
                                        style: "padding: var(--dx-card-spacing);",
                                        for turn in turns() {
                                            MessageScrollerItem {
                                                key: "{turn.id}",
                                                message_id: "m{turn.id}",
                                                scroll_anchor: turn.mine,
                                                Message {
                                                    class: if turn.preset.is_empty() { String::new() } else { format!("dx-demo-enter-{}", turn.preset) },
                                                    align: if turn.mine { MessageAlign::End } else { MessageAlign::Start },
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
                CardFooter { style: "gap: var(--dx-space-2);",
                    Select::<String> {
                        default_value: "fade".to_string(),
                        trigger_aria_label: "Animation preset",
                        on_value_change: move |value: Option<String>| {
                            if let Some(value) = value {
                                if let Some(&(id, _)) = PRESETS.iter().find(|(id, _)| *id == value) {
                                    preset.set(id);
                                }
                            }
                        },
                        SelectGroup {
                            for (i , (id , name)) in PRESETS.iter().copied().enumerate() {
                                SelectOption::<String> {
                                    key: "{id}",
                                    index: i,
                                    value: id.to_string(),
                                    text_value: name,
                                    "{name}"
                                }
                            }
                        }
                    }
                    Button {
                        size: ButtonSize::Icon,
                        r#type: "button",
                        style: "margin-inline-start: auto;",
                        aria_label: "Send Message",
                        disabled: next.is_none() || busy(),
                        onclick: move |_| send(),
                        ArrowUp {}
                    }
                }
            }
            p { style: "margin: 0; text-align: center; text-wrap: balance; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);",
                "Select an animation then click send to see it in action."
            }
        }
    }
}
