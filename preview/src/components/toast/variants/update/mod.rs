use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

/// `show` returns a `ToastId`; `update(id, ..)` changes that toast in place and `close(id)`
/// dismisses it. Updating or closing an id whose toast is already gone does nothing.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Update toast notifications", UpdateButtons {} }
    }
}

#[component]
fn UpdateButtons() -> Element {
    let toast_api = use_toast();
    let mut current = use_signal(|| None as Option<ToastId>);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    let id = toast_api
                        .info(
                            "Syncing...".to_string(),
                            ToastOptions::new().permanent(true),
                        );
                    current.set(Some(id));
                },
                "Show"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    if let Some(id) = current() {
                        toast_api
                            .update(
                                id,
                                ToastUpdate::new()
                                    .toast_type(ToastType::Success)
                                    .title("Synced")
                                    .permanent(false),
                            );
                    }
                },
                "Update"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    if let Some(id) = current() {
                        toast_api.close(id);
                    }
                },
                "Close"
            }
        }
    }
}
