use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::FileCode;

const CODE: &str = "import { streamText } from \"ai\"\n\nexport async function POST(request: Request) {\n  const { messages } = await request.json()\n\n  const result = streamText({\n    model: \"anthropic/claude-sonnet-4.5\",\n    messages,\n  })\n\n  return result.toUIMessageStreamResponse()\n}";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; min-width: 0;",
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle {
                        FileCode {}
                        "app/api/chat/route.ts"
                    }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { code: CODE }
            }
        }
    }
}
