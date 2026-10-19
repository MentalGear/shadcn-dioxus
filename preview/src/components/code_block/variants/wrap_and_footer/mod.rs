use super::super::component::*;
use dioxus::prelude::*;

const CODE: &str = "curl -X POST https://api.example.com/v1/chat/completions -H \"Authorization: Bearer $API_KEY\" -H \"Content-Type: application/json\" -d '{\"model\": \"claude-sonnet-4.5\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello\"}]}'";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-4);",
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "Scrolls sideways" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { code: CODE }
            }
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "Wraps" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { code: CODE, wrap: true }
                CodeBlockFooter { "Exited with code 0 in 214 ms" }
            }
        }
    }
}
