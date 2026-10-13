use crate::components::{
    button::{Button, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
        DrawerHeader, DrawerSide, DrawerTitle,
    },
};
use dioxus::prelude::*;

/// The `side` prop's two edge placements the `main` variant does not show (it has the default
/// bottom drawer and an "Open from Top" one). One drawer, re-pointed at an edge by the button.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut side = use_signal(|| DrawerSide::Right);

    let (title, description) = match side() {
        DrawerSide::Left => ("Left drawer", "This drawer slides in from the left edge."),
        _ => ("Right drawer", "This drawer slides in from the right edge."),
    };

    rsx! {
        div { display: "flex", gap: "0.5rem",
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| {
                    side.set(DrawerSide::Left);
                    open.set(true);
                },
                "Open from Left"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| {
                    side.set(DrawerSide::Right);
                    open.set(true);
                },
                "Open from Right"
            }
        }
        Drawer {
            open: open(),
            on_open_change: move |v| open.set(v),
            side: side(),
            DrawerContent {
                DrawerHandle {}
                DrawerHeader {
                    DrawerTitle { "{title}" }
                    DrawerDescription { "{description}" }
                }
                DrawerFooter {
                    DrawerClose { "Close" }
                }
            }
        }
    }
}
