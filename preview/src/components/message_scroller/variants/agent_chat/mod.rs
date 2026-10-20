use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonSize, ButtonVariant},
    checkpoint::Checkpoint,
    code_block::{
        CodeBlock, CodeBlockActions, CodeBlockContent, CodeBlockCopyButton, CodeBlockHeader,
        CodeBlockTitle,
    },
    message::{Message, MessageAlign, MessageContent},
    message_scroller::{
        MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem,
        MessageScrollerProvider, MessageScrollerViewport,
    },
    trace::{
        Trace, TraceCollapsible, TraceCollapsibleContent, TraceCollapsibleTrigger,
        TraceDescription, TraceIcon, TraceLine, TraceStatus, TraceTitle,
    },
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Brain, FileText, RotateCw};

const CODE: &str = "export async function POST(request: Request) {\n  const { messages } = await request.json()\n  const result = streamText({ model, messages })\n  return result.toUIMessageStreamResponse()\n}";

/// A real timer for the simulated run (this demo has no async runtime of its own; the
/// browser's `setTimeout` is the clock).
async fn sleep_ms(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

/// The agent chat of upstream's `chat-demo`, with the parts this library has: one agent turn
/// that thinks (a collapsible reasoning `Trace` that is open and shimmering while it runs), reads
/// files (a tool-call `Trace`), answers with a `CodeBlock`, and ends in a `Checkpoint`.
///
/// `stage` is how far the run has got: 0 nothing, 1 thinking, 2 reading, 3 answering, 4 done.
/// The checkpoint sits after the question; restoring it drops the answer that came after, and
/// a rerun is one click.
#[component]
pub fn Demo() -> Element {
    let mut stage = use_signal(|| 4u8);
    let mut running = use_signal(|| false);
    let mut thinking_open = use_signal(|| false);

    let mut run = move || {
        if running() {
            return;
        }
        running.set(true);
        stage.set(1);
        thinking_open.set(true);
        spawn(async move {
            sleep_ms(1200).await;
            stage.set(2);
            thinking_open.set(false);
            sleep_ms(1000).await;
            stage.set(3);
            sleep_ms(700).await;
            stage.set(4);
            running.set(false);
        });
    };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            div { style: "height: 26rem; min-height: 0; overflow: hidden; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                MessageScrollerProvider {
                    MessageScroller {
                        MessageScrollerViewport { "aria-label": "Agent conversation",
                            MessageScrollerContent {
                                "aria-busy": running(),
                                style: "padding: var(--dx-space-4);",
                                MessageScrollerItem { message_id: "agent-user", scroll_anchor: true,
                                    Message { align: MessageAlign::End,
                                        MessageContent {
                                            Bubble { variant: BubbleVariant::Muted,
                                                BubbleContent { "How do I stream a response from a route handler?" }
                                            }
                                        }
                                    }
                                }
                                MessageScrollerItem { message_id: "agent-checkpoint",
                                    Checkpoint {
                                        label: "Before the answer",
                                        disabled: stage() == 0 || running(),
                                        on_restore: move |_| stage.set(0),
                                    }
                                }
                                if stage() >= 1 {
                                    MessageScrollerItem { message_id: "agent-reply",
                                        Message {
                                            MessageContent { style: "gap: var(--dx-space-3);",
                                                Trace {
                                                    TraceCollapsible {
                                                        open: Some(thinking_open()),
                                                        on_open_change: move |next| thinking_open.set(next),
                                                        TraceCollapsibleTrigger {
                                                            TraceLine { status: if stage() == 1 { Some(TraceStatus::Running) } else { None },
                                                                TraceIcon {
                                                                    Brain {}
                                                                }
                                                                TraceTitle {
                                                                    if stage() == 1 {
                                                                        "Thinking"
                                                                    } else {
                                                                        "Thought"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        TraceCollapsibleContent {
                                                            p { style: "margin: 0;",
                                                                "A route handler can return the stream the AI SDK builds; the client reads it with a hook."
                                                            }
                                                        }
                                                    }
                                                    if stage() >= 2 {
                                                        TraceLine { status: if stage() == 2 { Some(TraceStatus::Running) } else { None },
                                                            TraceIcon {
                                                                FileText {}
                                                            }
                                                            TraceTitle {
                                                                if stage() == 2 {
                                                                    "Reading 3 files"
                                                                } else {
                                                                    "Read 3 files"
                                                                }
                                                            }
                                                            if stage() > 2 {
                                                                TraceDescription { "0.9s" }
                                                            }
                                                        }
                                                    }
                                                }
                                                if stage() >= 3 {
                                                    p { style: "margin: 0;", "Create a route handler that returns the stream:" }
                                                    CodeBlock { size: crate::components::code_block::CodeBlockSize::Sm,
                                                        CodeBlockHeader {
                                                            CodeBlockTitle { "app/api/chat/route.ts" }
                                                            CodeBlockActions {
                                                                CodeBlockCopyButton { value: CODE }
                                                            }
                                                        }
                                                        CodeBlockContent { code: CODE }
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
            div { style: "align-self: flex-start;",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: running(),
                    onclick: move |_| run(),
                    RotateCw {}
                    "Run again"
                }
            }
        }
    }
}
