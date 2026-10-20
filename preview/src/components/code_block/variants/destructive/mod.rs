use super::super::component::*;
use dioxus::prelude::*;

const CODE: &str = "error[E0382]: borrow of moved value: `name`\n --> src/main.rs:4:20\n  |\n3 |     let other = name;\n  |                 ---- value moved here\n4 |     println!(\"{name}\");\n  |                ^^^^ value borrowed here after move";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; min-width: 0;",
            CodeBlock { variant: CodeBlockVariant::Destructive,
                CodeBlockHeader {
                    CodeBlockTitle { "cargo check failed" }
                    CodeBlockActions {
                        CodeBlockCopyButton { value: CODE }
                    }
                }
                CodeBlockContent { code: CODE }
            }
        }
    }
}
