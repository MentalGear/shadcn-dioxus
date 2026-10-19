use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

/// Swipe to dismiss: drag a toast toward the edge of the screen it sits at (right in LTR, left
/// in RTL), or flick it. A short slow drag springs back.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Swipe toast notifications", SwipeButton {} }
    }
}

#[component]
fn SwipeButton() -> Element {
    let toast_api = use_toast();

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                toast_api
                    .info(
                        "Swipe me away".to_string(),
                        ToastOptions::new().duration(Duration::from_secs(60)),
                    );
            },
            "Show swipeable toast"
        }
    }
}
