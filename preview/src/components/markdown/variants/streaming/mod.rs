use super::super::component::*;
use crate::components::{
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::{Button, ButtonVariant},
    message::{Message, MessageAlign, MessageContent},
    message_scroller::{
        MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem,
        MessageScrollerProvider, MessageScrollerViewport,
    },
    switch::Switch,
};
use dioxus::prelude::*;

const REPLY: &str = "Here is a **quick plan** for the release:\n\n\
1. Run the *full* gate suite with `scripts/run-gates.sh`.\n\
2. Read the [release notes](https://example.com/release-notes) and fix anything flagged.\n\
3. Tag the build.\n\n\
```rust\nfn main() {\n    println!(\"ship it\");\n}\n```\n\n\
| Step | Owner |\n|------|-------|\n| Gates | CI |\n| Tag | You |\n\n\
Questions? See https://example.com/faq for the rest.";

async fn sleep_ms(ms: u32) {
    dioxus_primitives::sleep::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}

#[component]
pub fn Demo() -> Element {
    let mut text = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut heal = use_signal(|| true);

    let stream = move |_| {
        if busy() {
            return;
        }
        busy.set(true);
        text.set(String::new());
        spawn(async move {
            let chars: Vec<char> = REPLY.chars().collect();
            for chunk in chars.chunks(4) {
                sleep_ms(30).await;
                text.write().extend(chunk);
            }
            busy.set(false);
        });
    };

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 36rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            div { style: "height: 24rem; overflow: hidden; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg);",
                MessageScrollerProvider { auto_scroll: true,
                    MessageScroller {
                        MessageScrollerViewport { "aria-label": "Conversation",
                            MessageScrollerContent { style: "padding: var(--dx-space-4);",
                                MessageScrollerItem { message_id: "q", scroll_anchor: true,
                                    Message { align: MessageAlign::End,
                                        MessageContent {
                                            Bubble { variant: BubbleVariant::Default,
                                                BubbleContent { "Give me a release plan." }
                                            }
                                        }
                                    }
                                }
                                if !text().is_empty() {
                                    MessageScrollerItem { message_id: "a",
                                        Message { align: MessageAlign::Start,
                                            MessageContent {
                                                Bubble { variant: BubbleVariant::Ghost,
                                                    BubbleContent {
                                                        Markdown {
                                                            source: text(),
                                                            streaming: heal() && busy(),
                                                            "data-testid": "streamed-markdown",
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
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                Button {
                    disabled: busy(),
                    onclick: stream,
                    "data-testid": "stream-reply",
                    "Stream a reply"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: busy(),
                    onclick: move |_| text.set(String::new()),
                    "Clear"
                }
                label { style: "display: inline-flex; align-items: center; gap: var(--dx-space-2); font-size: var(--dx-text-sm);",
                    Switch {
                        checked: heal(),
                        aria_label: "Heal unfinished text",
                        on_checked_change: move |value| heal.set(value),
                    }
                    "Heal unfinished text"
                }
            }
        }
    }
}
