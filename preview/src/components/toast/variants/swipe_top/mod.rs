use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

/// A toaster at the top of the screen: a toast dismisses by swiping up, toward the edge it sits
/// at (a toaster at the bottom swipes down). Horizontal swipes work as everywhere else.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider {
            aria_label: "Top toast notifications",
            style: "inset-block: var(--dx-space-5) auto;",
            TopButton {}
        }
    }
}

#[component]
fn TopButton() -> Element {
    let toast_api = use_toast();

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                toast_api
                    .info(
                        "Swipe me up".to_string(),
                        ToastOptions::new().duration(Duration::from_secs(60)),
                    );
            },
            "Show top toast"
        }
    }
}
