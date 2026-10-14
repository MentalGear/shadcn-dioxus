use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonVariant},
    message::{Message, MessageAlign, MessageContent},
};
use dioxus::prelude::*;

const FIRST_LOADED: u32 = 60;
const BATCH: u32 = 6;

fn text(id: u32) -> String {
    let filler = match id % 3 {
        0 => " Short.",
        1 => " A somewhat longer line that wraps onto a second row in a narrow column.",
        _ => " A much longer one, which keeps going so that the heights of the rows differ from each other by quite a lot.",
    };
    format!("Message {id}.{filler}")
}

/// The scroll-edge fade on a transcript, `fade: ScrollFade::Top`: the thread opens at its
/// newest messages and the history above fades out instead of ending in a hard crop. Loading
/// earlier messages prepends rows and keeps your place; the fade follows the viewport's scroll
/// offset throughout. Pick another edge to compare (the Nova default is the bottom edge).
#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(|| (FIRST_LOADED..FIRST_LOADED + 40).collect::<Vec<_>>());
    let mut oldest = use_signal(|| FIRST_LOADED);
    let mut fade = use_signal(|| ScrollFade::Top);

    rsx! {
        MessageScrollerProvider {
            div { style: "display: flex; width: 100%; max-width: 32rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
                div {
                    "data-demo": "fade",
                    style: "height: 22rem; overflow: hidden; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                    MessageScroller {
                        MessageScrollerViewport {
                            "aria-label": "Conversation with a faded top edge",
                            fade: fade(),
                            MessageScrollerContent { style: "padding: var(--dx-space-4);",
                                for id in turns() {
                                    MessageScrollerItem { key: "{id}", message_id: "m{id}",
                                        Message {
                                            align: if id.is_multiple_of(2) { MessageAlign::End } else { MessageAlign::Start },
                                            MessageContent {
                                                Bubble {
                                                    variant: if id.is_multiple_of(2) { BubbleVariant::Default } else { BubbleVariant::Muted },
                                                    BubbleContent { {text(id)} }
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
                div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: oldest() == 0,
                        onclick: move |_| {
                            let from = oldest().saturating_sub(BATCH);
                            turns.write().splice(0..0, from..oldest());
                            oldest.set(from);
                        },
                        "Load earlier messages"
                    }
                    for (label, value) in [
                        ("Top", ScrollFade::Top),
                        ("Bottom", ScrollFade::Bottom),
                        ("Both", ScrollFade::Y),
                        ("None", ScrollFade::None),
                    ] {
                        Button {
                            key: "{label}",
                            variant: if fade() == value { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                            "aria-pressed": (fade() == value).to_string(),
                            onclick: move |_| fade.set(value),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
