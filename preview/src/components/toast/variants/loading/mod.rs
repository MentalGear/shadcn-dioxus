use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

/// A loading toast: a spinner, never auto-dismissed. Closed by hand here; the promise demo
/// moves one on by itself.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Loading toast notifications", LoadingButton {} }
    }
}

#[component]
fn LoadingButton() -> Element {
    let toast_api = use_toast();

    rsx! {
        Button {
            r#type: "button",
            "data-style": "outline",
            onclick: move |_| {
                toast_api
                    .loading(
                        "Uploading file...".to_string(),
                        ToastOptions::new().description("This toast stays until you close it."),
                    );
            },
            "Show loading"
        }
    }
}
