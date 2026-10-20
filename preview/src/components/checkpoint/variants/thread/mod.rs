use super::super::component::*;
use crate::components::bubble::{Bubble, BubbleContent, BubbleVariant};
use crate::components::message::{Message, MessageAlign, MessageContent};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct Turn {
    id: u32,
    mine: bool,
    text: &'static str,
    checkpoint: Option<&'static str>,
}

const THREAD: [Turn; 6] = [
    Turn { id: 1, mine: true, text: "Rename the config module.", checkpoint: None },
    Turn { id: 2, mine: false, text: "Done: config.rs is now settings.rs.", checkpoint: Some("After rename") },
    Turn { id: 3, mine: true, text: "Now split it by section.", checkpoint: None },
    Turn { id: 4, mine: false, text: "Split into three files; tests pass.", checkpoint: Some("After split") },
    Turn { id: 5, mine: true, text: "Also delete the old loader.", checkpoint: None },
    Turn { id: 6, mine: false, text: "Removed it; two callers needed updating.", checkpoint: Some("After cleanup") },
];

/// Restore keeps the turn and drops every message after it, exactly as the upstream recipe
/// describes; the checkpoint itself stays so you can move forward again by sending.
#[component]
pub fn Demo() -> Element {
    let mut kept = use_signal(|| THREAD.len());
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-4);",
            for turn in THREAD.iter().take(kept()) {
                Message {
                    key: "{turn.id}",
                    align: if turn.mine { MessageAlign::End } else { MessageAlign::Start },
                    MessageContent {
                        Bubble {
                            variant: if turn.mine { BubbleVariant::Default } else { BubbleVariant::Muted },
                            BubbleContent { "{turn.text}" }
                        }
                    }
                }
                if let Some(label) = turn.checkpoint {
                    {
                        let index = turn.id as usize;
                        rsx! {
                            Checkpoint {
                                key: "cp-{turn.id}",
                                label,
                                disabled: index == kept(),
                                on_restore: move |_| kept.set(index),
                            }
                        }
                    }
                }
            }
        }
    }
}
