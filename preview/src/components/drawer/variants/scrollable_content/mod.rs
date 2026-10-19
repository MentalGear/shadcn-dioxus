use crate::components::{
    button::{Button, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader,
        DrawerSide, DrawerTitle,
    },
};
use dioxus::prelude::*;

/// shadcn's "Scrollable Content" drawer: a side drawer whose body is longer than the screen and
/// scrolls inside the panel while the header and footer stay put.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Open scrollable drawer"
        }
        Drawer {
            open: open(),
            on_open_change: move |v| open.set(v),
            side: DrawerSide::Right,
            DrawerContent {
                DrawerHeader {
                    DrawerTitle { "Release notes" }
                    DrawerDescription { "Scroll the list; the footer stays in view." }
                }
                div {
                    "data-testid": "drawer-scroll-body",
                    tabindex: "0",
                    role: "region",
                    aria_label: "Release notes list",
                    style: "flex: 1 1 0%; min-height: 0; overflow-y: auto; padding: 0 1rem; display: grid; gap: 0.75rem;",
                    for i in 1..=30 {
                        p { style: "margin: 0; font-size: var(--dx-text-sm);",
                            "Release 1.{i}: fixes, polish and small improvements."
                        }
                    }
                }
                DrawerFooter {
                    DrawerClose { "Done reading" }
                }
            }
        }
    }
}
