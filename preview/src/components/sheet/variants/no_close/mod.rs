use crate::components::{
    button::{Button, ButtonVariant},
    sheet::{Sheet, SheetDescription, SheetHeader, SheetTitle},
};
use dioxus::prelude::*;

/// A sheet without the corner close button: `SheetContentClose` is an opt-in child of `Sheet`, so
/// leaving it out is the whole variant. Escape and a click on the scrim still dismiss it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Sheet without close button"
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            SheetHeader {
                SheetTitle { "No close button" }
                SheetDescription {
                    "This sheet has no close button in the top-right corner. Press Escape or click outside to dismiss it."
                }
            }
        }
    }
}
