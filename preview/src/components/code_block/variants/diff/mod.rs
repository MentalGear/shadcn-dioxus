use super::super::component::*;
use dioxus::prelude::*;

const AFTER: &str = "const result = streamText({\n  model: \"anthropic/claude-sonnet-4.5\",\n  messages,\n})";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; min-width: 0;",
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "app/api/chat/route.ts" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: AFTER }
                    }
                }
                CodeBlockContent { line_numbers: true,
                    CodeBlockLine { "const result = streamText({{" }
                    CodeBlockLine { variant: CodeBlockLineVariant::Removed, "  model: \"openai/gpt-4o\"," }
                    CodeBlockLine { variant: CodeBlockLineVariant::Added, "  model: \"anthropic/claude-sonnet-4.5\"," }
                    CodeBlockLine { variant: CodeBlockLineVariant::Highlighted, "  messages," }
                    CodeBlockLine { "}})" }
                }
            }
        }
    }
}
