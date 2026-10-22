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

fn rows(ids: Vec<u32>) -> Element {
    rsx! {
        for id in ids {
            MessageScrollerItem { key: "{id}", message_id: "m{id}",
                Message { align: if id.is_multiple_of(2) { MessageAlign::End } else { MessageAlign::Start },
                    MessageContent {
                        Bubble { variant: if id.is_multiple_of(2) { BubbleVariant::Default } else { BubbleVariant::Muted },
                            BubbleContent { {text(id)} }
                        }
                    }
                }
            }
        }
    }
}

const LABEL: &str = "margin: 0; font-size: var(--dx-text-xs); color: var(--dx-muted-foreground);";
const FRAME: &str = "height: 22rem; min-width: 0; overflow: hidden; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);";

/// The scroll-edge fade on a transcript. Both panels hold the same thread, opened at its newest
/// messages: the left one is `fade: ScrollFade::None` (the history above ends in a hard crop), the
/// right one is the interactive panel (`ScrollFade::Top` to start, so the history above fades out).
/// Loading earlier messages prepends rows and keeps your place; the fade follows the viewport's
/// scroll offset throughout. Pick another edge to compare (the Nova default is the bottom edge).
#[component]
pub fn Demo() -> Element {
    let mut turns = use_signal(|| (FIRST_LOADED..FIRST_LOADED + 40).collect::<Vec<_>>());
    let mut oldest = use_signal(|| FIRST_LOADED);
    let mut fade = use_signal(|| ScrollFade::Top);
    let current = match fade() {
        ScrollFade::Top => "ScrollFade::Top",
        ScrollFade::Bottom => "ScrollFade::Bottom",
        ScrollFade::Y => "ScrollFade::Y",
        _ => "ScrollFade::None",
    };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 40rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            div { style: "display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--dx-space-3);",
                div { style: "display: flex; min-width: 0; flex-direction: column; gap: var(--dx-space-2);",
                    p { style: LABEL, "ScrollFade::None (hard crop)" }
                    MessageScrollerProvider {
                        div { "data-demo": "fade-none", style: FRAME,
                            MessageScroller {
                                MessageScrollerViewport {
                                    "aria-label": "Conversation without an edge fade",
                                    fade: ScrollFade::None,
                                    MessageScrollerContent { style: "padding: var(--dx-space-4);",
                                        {rows((FIRST_LOADED..FIRST_LOADED + 40).collect())}
                                    }
                                }
                            }
                        }
                    }
                }
                div { style: "display: flex; min-width: 0; flex-direction: column; gap: var(--dx-space-2);",
                    p { style: LABEL, "{current}" }
                    MessageScrollerProvider {
                        div { "data-demo": "fade", style: FRAME,
                            MessageScroller {
                                MessageScrollerViewport {
                                    "aria-label": "Conversation with an edge fade",
                                    fade: fade(),
                                    MessageScrollerContent { style: "padding: var(--dx-space-4);",
                                        {rows(turns())}
                                    }
                                }
                                MessageScrollerButton {}
                            }
                        }
                    }
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
            p { style: LABEL,
                "Scroll both panels up from the newest message: the left one crops rows at the top edge, the right one fades them out."
            }
        }
    }
}
