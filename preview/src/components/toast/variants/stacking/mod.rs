use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

/// Stacking: several toasts collapse into a pile; hovering or focusing the stack (or pressing
/// F6) expands it. Under `prefers-reduced-motion` the pile expands without animation.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Stacked toast notifications", StackButton {} }
    }
}

#[component]
fn StackButton() -> Element {
    let toast_api = use_toast();
    let mut count = use_signal(|| 0u32);

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                count += 1;
                toast_api
                    .info(
                        format!("Notification #{count}"),
                        ToastOptions::new().duration(Duration::from_secs(60)),
                    );
            },
            "Add to stack"
        }
    }
}
