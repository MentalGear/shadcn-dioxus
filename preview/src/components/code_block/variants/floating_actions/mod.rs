use super::super::component::*;
use dioxus::prelude::*;

const CODE: &str = "npx shadcn@latest add code-block";

/// With no header the actions float over the top corner and appear on hover or focus (always
/// on touch screens).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; min-width: 0;",
            CodeBlock { size: CodeBlockSize::Sm,
                CodeBlockActions {
                    CodeBlockCopyButton { value: CODE }
                }
                CodeBlockContent { code: CODE }
            }
        }
    }
}
