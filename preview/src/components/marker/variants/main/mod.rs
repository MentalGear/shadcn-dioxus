use super::super::component::*;
use crate::components::spinner::Spinner;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::{ChevronRight, CircleUserRound, Clock, FileText, GitBranch};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; flex-direction: column; gap: var(--dx-space-6);",
            Marker {
                MarkerContent { "A default marker" }
            }
            Marker {
                MarkerIcon {
                    FileText {}
                }
                MarkerContent { "Marker with icon" }
            }
            Marker { role: "status",
                MarkerIcon {
                    Spinner {}
                }
                MarkerContent { "Marker with a spinner" }
            }
            Marker { role: "status",
                MarkerContent { class: "dx-shimmer", "Thinking..." }
            }
            Marker {
                r#as: move |el: AsChild| rsx! {
                    a { href: "#", ..el.attributes,
                        {el.children}
                    }
                },
                MarkerIcon {
                    GitBranch {}
                }
                MarkerContent { "Marker as a link" }
            }
            Marker {
                r#as: move |el: AsChild| rsx! {
                    button { r#type: "button", ..el.attributes,
                        {el.children}
                    }
                },
                MarkerIcon {
                    Clock {}
                }
                MarkerContent { style: "flex: 1", "Marker as a button" }
                MarkerIcon {
                    ChevronRight {}
                }
            }
            Marker {
                MarkerIcon {
                    CircleUserRound {}
                }
                MarkerContent { "Rhea joined the chat" }
            }
            Marker { style: "justify-content: center",
                MarkerContent { "Olivia Rose left the chat" }
            }
        }
    }
}
