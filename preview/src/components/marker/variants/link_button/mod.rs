use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::{ChevronRight, Clock, FileText};

/// `as` turns the whole marker into a real link or button.
#[component]
pub fn Demo() -> Element {
    let mut clicks = use_signal(|| 0);
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; flex-direction: column; gap: var(--dx-space-6);",
            Marker {
                r#as: move |el: AsChild| rsx! {
                    a { href: "#", ..el.attributes,
                        {el.children}
                    }
                },
                MarkerIcon {
                    FileText {}
                }
                MarkerContent { "View the pull request" }
            }
            Marker {
                r#as: move |el: AsChild| rsx! {
                    button {
                        r#type: "button",
                        onclick: move |_| clicks += 1,
                        ..el.attributes,
                        {el.children}
                    }
                },
                MarkerIcon {
                    Clock {}
                }
                MarkerContent { style: "flex: 1", "Show earlier activity ({clicks})" }
                MarkerIcon {
                    ChevronRight {}
                }
            }
        }
    }
}
