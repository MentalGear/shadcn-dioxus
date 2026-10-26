use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;

/// `promise`: one toast that moves from loading to success (or error) in place, announced once
/// through its live region (`status`, or `alert` for the error).
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Promise toast notifications", PromiseButtons {} }
    }
}

/// A stand-in for a request: resolves after 1.5 s (one `sleep`, not a loop).
async fn save(ok: bool) -> Result<&'static str, &'static str> {
    dioxus_primitives::sleep::sleep(std::time::Duration::from_millis(1500)).await;
    if ok {
        Ok("Event")
    } else {
        Err("Server unavailable")
    }
}

#[component]
fn PromiseButtons() -> Element {
    let toast_api = use_toast();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .promise(
                            save(true),
                            PromiseMessages::new(
                                "Saving event...",
                                |name: &&str| format!("{name} has been created"),
                                |e: &&str| format!("Could not save: {e}"),
                            ),
                        );
                },
                "Promise (success)"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .promise(
                            save(false),
                            PromiseMessages::new(
                                "Saving event...",
                                |name: &&str| format!("{name} has been created"),
                                |e: &&str| format!("Could not save: {e}"),
                            ),
                        );
                },
                "Promise (error)"
            }
        }
    }
}
