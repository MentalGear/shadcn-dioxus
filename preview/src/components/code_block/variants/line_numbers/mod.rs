use super::super::component::*;
use dioxus::prelude::*;

const CODE: &str = "fn main() {\n    let name = \"world\";\n\n    println!(\"hello, {name}\");\n}";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 36rem; min-width: 0; flex-direction: column; gap: var(--dx-space-4);",
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "src/main.rs" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { code: CODE, line_numbers: true }
            }
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "src/main.rs (from line 41)" }
                }
                CodeBlockContent { code: CODE, line_numbers: true, start_line: 41 }
            }
        }
    }
}
