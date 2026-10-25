use super::super::component::*;
use dioxus::prelude::*;

/// `horizontal: false`: the bar stacks its buttons in a column, `ArrowUp`/`ArrowDown` move between
/// them, and the separator between the two groups is drawn as a horizontal rule.
#[component]
pub fn Demo() -> Element {
    let mut last = use_signal(|| None::<&'static str>);

    rsx! {
        div { class: "dx-toolbar-demo",
            Toolbar { aria_label: "Clipboard", horizontal: false,
                ToolbarGroup {
                    ToolbarButton { index: 0usize, on_click: move |_| last.set(Some("Cut")), "Cut" }
                    ToolbarButton { index: 1usize, on_click: move |_| last.set(Some("Copy")), "Copy" }
                    ToolbarButton { index: 2usize, on_click: move |_| last.set(Some("Paste")), "Paste" }
                }
                ToolbarSeparator {}
                ToolbarGroup {
                    ToolbarButton { index: 3usize, on_click: move |_| last.set(Some("Undo")), "Undo" }
                    ToolbarButton { index: 4usize, on_click: move |_| last.set(Some("Redo")), "Redo" }
                }
            }
            if let Some(action) = last() {
                p { "Last action: {action}" }
            }
        }
    }
}
