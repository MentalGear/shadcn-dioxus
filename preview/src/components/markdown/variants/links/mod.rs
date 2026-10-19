use super::super::component::*;
use dioxus::prelude::*;

const SOURCE: &str = "\
- [Web link](https://example.com/page) opens in a new tab with `rel=\"noopener noreferrer\"`.\n\
- [Relative link](/component/button/) stays in this tab.\n\
- [Mail link](mailto:hello@example.com) and [phone link](tel:+15551234567) are handed to the OS.\n\
- [A script link](javascript:alert(1)) keeps its label and loses its target.\n\
- [A data link](data:text/html,hi) is refused too.\n\
- Autolinked: https://example.com/a_path, www.example.org.";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem; --dx-font-mono: ui-monospace, SFMono-Regular, Menlo, monospace;",
            Markdown { source: SOURCE }
        }
    }
}
