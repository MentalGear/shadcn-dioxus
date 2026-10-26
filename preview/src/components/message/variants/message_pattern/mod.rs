use super::super::component::*;
use crate::components::bubble::{Bubble, BubbleContent, BubbleVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::markdown::Markdown;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Copy, Pencil, RefreshCw, ThumbsDown, ThumbsUp};

const REPLY: &str = "Install Dialog, then open it from the gear button in `site-header.tsx`.\n\n```bash\nnpx shadcn@latest add dialog\n```\n\nWrap the button in a `DialogTrigger` and put the settings form in `DialogContent`. Focus moves into the dialog when it opens, and **Esc** closes it.";

#[derive(Copy, Clone, PartialEq)]
enum Feedback {
    Like,
    Dislike,
}

/// Upstream's `message-pattern`: the footer actions of a user message (copy, edit) and of an
/// assistant reply (copy with a "Copied" state, a pressed-state thumbs pair, retry). Toggle
/// buttons carry `aria-pressed`; icon-only buttons carry `aria-label`.
#[component]
pub fn Demo() -> Element {
    let mut copied = use_signal(|| false);
    let mut feedback = use_signal(|| None::<Feedback>);
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-8); font-size: var(--dx-text-sm);",
            Message { align: MessageAlign::End,
                MessageContent {
                    Bubble { variant: BubbleVariant::Muted,
                        BubbleContent { "How do I add a settings dialog? It should open from the gear icon in the header." }
                    }
                    MessageFooter {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: "Copy message",
                            Copy {}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: "Edit message",
                            Pencil {}
                        }
                    }
                }
            }
            Message {
                MessageContent { style: "gap: var(--dx-space-2);",
                    Markdown { source: REPLY }
                    MessageFooter { style: "padding-inline: 0;",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: if copied() { "Copied" } else { "Copy response" },
                            onclick: move |_| {
                                copied.set(true);
                                spawn(async move {
                                    dioxus_primitives::sleep::sleep(
                                        std::time::Duration::from_millis(2000),
                                    )
                                    .await;
                                    copied.set(false);
                                });
                            },
                            if copied() {
                                Check {}
                            } else {
                                Copy {}
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: "Good response",
                            "aria-pressed": feedback() == Some(Feedback::Like),
                            onclick: move |_| {
                                feedback
                                    .set(
                                        if feedback() == Some(Feedback::Like) {
                                            None
                                        } else {
                                            Some(Feedback::Like)
                                        },
                                    )
                            },
                            ThumbsUp {}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: "Bad response",
                            "aria-pressed": feedback() == Some(Feedback::Dislike),
                            onclick: move |_| {
                                feedback
                                    .set(
                                        if feedback() == Some(Feedback::Dislike) {
                                            None
                                        } else {
                                            Some(Feedback::Dislike)
                                        },
                                    )
                            },
                            ThumbsDown {}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            aria_label: "Retry",
                            RefreshCw {}
                        }
                    }
                }
            }
        }
    }
}
