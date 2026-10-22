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

/// The user messages, one per press of send. Short on purpose: the demo is about the bubble's entrance,
/// so nothing else (no assistant reply, no streamed text) competes with it.
const SCRIPT: [&str; 8] = [
    "Can user messages pop in like iMessage without breaking anchoring?",
    "What makes the animation feel more like iMessage?",
    "Pick a preset, then send again.",
    "Each new bubble plays the preset you picked.",
    "Rows already on screen keep theirs.",
    "Try the spring bounce.",
    "Or the blur fade.",
    "Slide side is the last one.",
];

/// The presets: `(id, name)`. Each plays one entrance keyframes (the theme's `dx-fade-in`,
/// `dx-pop-in`, `dx-drop-in`, and `dx-effects.css`'s `dx-enter-*` for the last three), which
/// animate only `opacity`, `scale`, `translate` and `filter`, so a row's measured height never
/// changes while it plays.
const PRESETS: [(&str, &str); 6] = [
    ("fade", "Fade"),
    ("pop", "Pop"),
    ("drop", "Drop"),
    ("spring-bounce", "Spring bounce"),
    ("blur-fade", "Blur fade"),
    ("slide-side", "Slide side"),
];

/// The class that plays `preset` on a row: the three `dx-effects.css` presets are theme classes,
/// the first three are scoped to this demo's stylesheet below.
fn preset_class(preset: &str) -> String {
    match preset {
        "" => String::new(),
        "spring-bounce" | "blur-fade" | "slide-side" => format!("dx-enter-{preset}"),
        _ => format!("dx-demo-enter-{preset}"),
    }
}

#[derive(Clone, PartialEq)]
struct Turn {
    id: u32,
    text: String,
    /// The preset that was selected when this row was sent. Stored with the row, so changing the
    /// preset later never replays the entrance of rows already on screen.
    preset: &'static str,
}

#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(Vec::<Turn>::new);
    let mut sent = use_signal(|| 0usize);
    let mut preset = use_signal(|| "fade");
    let next = SCRIPT.get(sent());

    let mut send = move || {
        let Some(&user) = SCRIPT.get(sent()) else {
            return;
        };
        let id = sent() as u32;
        sent += 1;
        turns.write().push(Turn {
            id,
            text: user.to_string(),
            preset: preset(),
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
                            disabled: turns.read().is_empty(),
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
                                        style: "padding: var(--dx-card-spacing);",
                                        for turn in turns() {
                                            MessageScrollerItem {
                                                key: "{turn.id}",
                                                message_id: "m{turn.id}",
                                                Message {
                                                    class: preset_class(turn.preset),
                                                    align: MessageAlign::End,
                                                    MessageContent {
                                                        Bubble { variant: BubbleVariant::Muted,
                                                            BubbleContent { "{turn.text}" }
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
                        item_to_label: |id: String| {
                            PRESETS.iter().find(|(preset, _)| *preset == id).map_or(id.clone(), |(_, name)| name.to_string())
                        },
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
                        disabled: next.is_none(),
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
