use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

/// A toast is as tall as its content: one line, a title and a wrapped description, or more. Its
/// padding is the same for all of them, and a collapsed pile shows every toast at the height of the
/// front one whatever each holds.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Multiline toast notifications", MultilineButtons {} }
    }
}

#[component]
fn MultilineButtons() -> Element {
    let toast_api = use_toast();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .info("Saved".to_string(), ToastOptions::new().duration(Duration::from_secs(60)));
                },
                "Add short toast"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .info(
                            "Uploading file...".to_string(),
                            ToastOptions::new()
                                .description(
                                    "The upload keeps running in the background, so you can close this tab and come back to it later on.",
                                )
                                .duration(Duration::from_secs(60)),
                        );
                },
                "Add long toast"
            }
        }
    }
}
