use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronRight, File, Folder, FolderOpen};

#[derive(Clone, Copy, PartialEq)]
enum Node {
    File(&'static str),
    Folder(&'static str, &'static [Node]),
}

static TREE: &[Node] = &[
    Node::Folder(
        "src",
        &[
            Node::Folder(
                "components",
                &[Node::File("button.rs"), Node::File("dialog.rs")],
            ),
            Node::File("lib.rs"),
            Node::File("main.rs"),
        ],
    ),
    Node::Folder("assets", &[Node::File("logo.svg")]),
    Node::File("Cargo.toml"),
    Node::File("README.md"),
];

// A folder row is the collapsible's trigger, which carries `.dx-collapsible-trigger`
// (`justify-content: space-between`, for a title/chevron header); the row's own
// `justify-content: flex-start` below (inline style) packs chevron, icon and name together.
const ROW: &str = "display: flex; align-items: center; gap: 0.5rem; padding-block: 0.25rem; \
    font-size: var(--dx-text-sm); color: var(--dx-foreground);";

#[component]
fn TreeNode(node: Node, default_open: bool) -> Element {
    match node {
        Node::File(name) => rsx! {
            div { style: "{ROW} padding-inline-start: 1.5rem;",
                File { size: "1rem" }
                span { "{name}" }
            }
        },
        Node::Folder(name, children) => {
            let mut open = use_signal(|| default_open);
            rsx! {
                Collapsible {
                    open: Some(open()),
                    on_open_change: move |next: bool| open.set(next),
                    CollapsibleTrigger {
                        r#as: move |attrs: Vec<Attribute>| rsx! {
                            button { style: "{ROW} justify-content: flex-start; width: 100%; border: none; background: transparent; cursor: pointer;", ..attrs,
                                if open() {
                                    ChevronRight { size: "1rem", style: "transform: rotate(90deg);" }
                                } else {
                                    ChevronRight { size: "1rem" }
                                }
                                if open() {
                                    FolderOpen { size: "1rem" }
                                } else {
                                    Folder { size: "1rem" }
                                }
                                span { "{name}" }
                            }
                        },
                    }
                    CollapsibleContent {
                        div { style: "margin-inline-start: 0.5rem; padding-inline-start: 0.5rem; border-inline-start: 1px solid var(--dx-border);",
                            for child in children.iter() {
                                TreeNode { node: *child, default_open: false }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Nested file tree: every folder is its own `Collapsible`, so folders open and close independently.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 16rem;",
            for node in TREE.iter() {
                TreeNode { node: *node, default_open: matches!(node, Node::Folder("src", _)) }
            }
        }
    }
}
