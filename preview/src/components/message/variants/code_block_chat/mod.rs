use super::super::component::*;
use crate::components::bubble::{Bubble, BubbleContent, BubbleVariant};
use crate::components::code_block::{
    CodeBlock, CodeBlockActions, CodeBlockContent, CodeBlockCopyButton, CodeBlockHeader,
    CodeBlockSize, CodeBlockTitle,
};
use dioxus::prelude::*;

const CODE: &str = "import { streamText } from \"ai\"\n\nexport async function POST(request: Request) {\n  const { messages } = await request.json()\n\n  const result = streamText({\n    model: \"anthropic/claude-sonnet-4.5\",\n    messages,\n  })\n\n  return result.toUIMessageStreamResponse()\n}";

/// Upstream's `code-block-chat`: an assistant reply with prose around a framed code block that has
/// a filename and a copy button.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-4); font-size: var(--dx-text-sm);",
            Message { align: MessageAlign::End,
                MessageContent {
                    Bubble { variant: BubbleVariant::Muted,
                        BubbleContent { "How do I stream a response?" }
                    }
                }
            }
            Message {
                MessageContent { style: "gap: var(--dx-space-3);",
                    p { style: "margin: 0;", "Create a route handler that returns the stream:" }
                    CodeBlock { size: CodeBlockSize::Sm,
                        CodeBlockHeader {
                            CodeBlockTitle { "app/api/chat/route.ts" }
                            CodeBlockActions {
                                CodeBlockCopyButton { value: CODE }
                            }
                        }
                        CodeBlockContent { code: CODE }
                    }
                    p { style: "margin: 0;", "The client reads it with useChat." }
                }
            }
        }
    }
}
