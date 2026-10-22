use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

/// Pause on hover and focus: while the pointer is over the toaster or focus is inside it, the
/// toasts stay; they resume with the time that was left when you move away.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Pausing toast notifications", PauseButton {} }
    }
}

#[component]
fn PauseButton() -> Element {
    let toast_api = use_toast();
    // Numbered so several toasts from repeated clicks can be told apart.
    let mut count = use_signal(|| 0u32);

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                count += 1;
                toast_api
                    .info(
                        format!("Hover or focus me #{count}"),
                        ToastOptions::new().duration(Duration::from_secs(3)),
                    );
            },
            "Show timed toast"
        }
    }
}
