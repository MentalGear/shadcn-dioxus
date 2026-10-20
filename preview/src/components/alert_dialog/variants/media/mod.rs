use crate::components::button::{Button, ButtonVariant};

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{BluetoothConnected, Trash2};

/// `AlertDialogMedia` puts an icon tile above the title, in the default size and in `Sm`.
#[component]
pub fn Demo() -> Element {
    let mut default_open = use_signal(|| false);
    let mut small_open = use_signal(|| false);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| default_open.set(true),
                "Share Project"
            }
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| small_open.set(true),
                "Pair Device"
            }
        }
        AlertDialog { open: default_open(), on_open_change: move |v| default_open.set(v),
            AlertDialogMedia {
                Trash2 {}
            }
            AlertDialogTitle { "Delete this chat?" }
            AlertDialogDescription {
                "This will permanently delete the conversation and every file attached to it."
            }
            AlertDialogActions {
                AlertDialogCancel { "Cancel" }
                AlertDialogAction { "Delete" }
            }
        }
        AlertDialog {
            size: AlertDialogSize::Sm,
            open: small_open(),
            on_open_change: move |v| small_open.set(v),
            AlertDialogMedia {
                BluetoothConnected {}
            }
            AlertDialogTitle { "Allow Bluetooth pairing?" }
            AlertDialogDescription { "Your headphones want to pair with this device." }
            AlertDialogActions {
                AlertDialogCancel { "Not now" }
                AlertDialogAction { variant: AlertDialogActionVariant::Primary, "Pair" }
            }
        }
    }
}
