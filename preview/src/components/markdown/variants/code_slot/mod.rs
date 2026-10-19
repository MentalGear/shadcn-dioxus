use super::super::component::*;
use dioxus::prelude::*;

const SOURCE: &str = "The default block is a plain `<pre><code>`. This one is rendered by the consumer's `highlight` callback:\n\n```rust\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n```\n\n```\nno language, same slot\n```";

/// A stand-in for a real highlighter (syntect, tree-sitter, dioxus-code): numbers the lines and
/// emboldens a few keywords. It receives only the language and the verbatim code, and renders
/// ordinary elements, so it needs no raw HTML.
fn number_lines(block: CodeBlock) -> Element {
    let language = block.language.clone().unwrap_or_else(|| "text".to_string());
    rsx! {
        code { class: "language-{language}", "data-highlighter": "demo",
            for (n , line) in block.code.lines().enumerate() {
                span { style: "display: block;",
                    span { style: "display: inline-block; width: 2ch; margin-inline-end: 1ch; color: var(--dx-muted-foreground); text-align: end;",
                        "{n + 1}"
                    }
                    for (i , word) in line.split_inclusive(' ').enumerate() {
                        if matches!(word.trim(), "fn" | "let" | "pub") {
                            strong { key: "{i}", "{word}" }
                        } else {
                            span { key: "{i}", "{word}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; --dx-font-mono: ui-monospace, SFMono-Regular, Menlo, monospace;",
            Markdown { source: SOURCE, highlight: Callback::new(number_lines) }
        }
    }
}
