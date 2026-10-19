use super::super::component::*;
use dioxus::prelude::*;

const CODE: &str = "fn add(a: i32, b: i32) -> i32 {\n    let sum = a + b;\n    sum\n}";

/// A stand-in for a real highlighter (syntect, tree-sitter, dioxus-code): it only needs to
/// return one `CodeBlockLine` per line, with whatever spans it likes inside. This one
/// emboldens a few keywords; the copy button still copies the verbatim `CODE`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; min-width: 0;",
            CodeBlock {
                CodeBlockHeader {
                    CodeBlockTitle { "lib.rs" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { line_numbers: true,
                    for (n , line) in CODE.lines().enumerate() {
                        CodeBlockLine { key: "{n}",
                            for (i , word) in line.split_inclusive(' ').enumerate() {
                                if matches!(word.trim(), "fn" | "let") {
                                    strong { key: "{i}", style: "color: var(--dx-primary);", "{word}" }
                                } else {
                                    span { key: "{i}", "{word}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
