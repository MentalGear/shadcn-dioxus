use crate::components::button::{Button, ButtonVariant};

use super::super::component::*;
use dioxus::prelude::*;

/// `size: AlertDialogSize::Sm` (shadcn's `size="sm"`): a compact, centred panel whose two actions
/// share one row.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Show Small Dialog"
        }
        AlertDialog {
            size: AlertDialogSize::Sm,
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogTitle { "Allow accessory to connect?" }
            AlertDialogDescription {
                "Do you want to allow the USB accessory to connect to this device?"
            }
            AlertDialogActions {
                AlertDialogCancel { "Don't allow" }
                AlertDialogAction { variant: AlertDialogActionVariant::Primary, "Allow" }
            }
        }
    }
}
