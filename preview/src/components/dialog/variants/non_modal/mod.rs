use crate::components::button::{Button, ButtonVariant};

use super::super::component::{
    Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// `is_modal: false`: no focus trap, no inert page and no scrim, so the page behind the dialog stays usable.
/// The page button counts its clicks while the dialog is open.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut page_clicks = use_signal(|| 0_u32);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| open.set(true),
                "Open non-modal dialog"
            }
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| page_clicks.set(page_clicks() + 1),
                "Page button"
            }
            span { style: "font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "Page clicks: {page_clicks()}"
            }
        }
        Dialog {
            is_modal: false,
            open: open(),
            on_open_change: move |v| open.set(v),
            DialogHeader {
                DialogTitle { "Non-modal dialog" }
                DialogDescription { "The page behind is still usable: press the page button while this is open." }
            }
            DialogFooter {
                DialogClose {
                    r#as: |el: AsChild| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                    },
                    "Close"
                }
            }
        }
    }
}
