use crate::components::{
    button::{Button, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
        DrawerHeader, DrawerTitle,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// A drawer inside a drawer (Vaul's `NestedRoot`). The child is a plain `Drawer` rendered inside the
/// parent's `DrawerContent`: opening it steps the parent back (a small shrink and offset), closing it
/// restores the parent. Escape, a drag and a click on the scrim act on the top-most drawer only.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut nested_open = use_signal(|| false);

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Open nested drawers"
        }
        Drawer { open: open(), on_open_change: move |v| open.set(v),
            DrawerContent {
                DrawerHandle {}
                DrawerHeader {
                    DrawerTitle { "Parent drawer" }
                    DrawerDescription { "Open another drawer from inside this one." }
                }
                DrawerFooter {
                    Button { onclick: move |_| nested_open.set(true), "Open nested drawer" }
                    DrawerClose {
                        as: |el: AsChild| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                        },
                        "Close"
                    }
                }

                Drawer {
                    open: nested_open(),
                    on_open_change: move |v| nested_open.set(v),
                    DrawerContent {
                        DrawerHandle {}
                        DrawerHeader {
                            DrawerTitle { "Nested drawer" }
                            DrawerDescription {
                                "Escape, a drag or a tap on the scrim closes only this one."
                            }
                        }
                        DrawerFooter {
                            DrawerClose {
                                as: |el: AsChild| rsx! {
                                    Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                                },
                                "Close nested"
                            }
                        }
                    }
                }
            }
        }
    }
}
