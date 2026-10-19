use crate::components::{
    button::{Button, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
        DrawerHeader, DrawerSide, DrawerTitle,
    },
};
use dioxus::prelude::*;

/// A non-modal drawer (Base UI `modal: false`): no scrim, no focus trap, no scroll lock, and the page
/// behind it stays interactive. Escape and the close button still close it, and so does a press outside
/// the panel (Base UI's default light dismissal), so the page control below is pressed once, then the
/// drawer goes.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut count = use_signal(|| 0);

    rsx! {
        div { display: "flex", gap: "0.5rem",
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| open.set(true),
                "Open non-modal drawer"
            }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: move |_| count += 1,
                "Page button: {count}"
            }
        }
        Drawer {
            open: open(),
            on_open_change: move |v| open.set(v),
            side: DrawerSide::Bottom,
            is_modal: false,
            DrawerContent {
                DrawerHandle {}
                DrawerHeader {
                    DrawerTitle { "Non-modal drawer" }
                    DrawerDescription {
                        "The page behind stays interactive and scrollable. Escape or Close dismisses it."
                    }
                }
                DrawerFooter {
                    DrawerClose {
                        as: |attributes| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes, "Close" }
                        },
                    }
                }
            }
        }
    }
}
