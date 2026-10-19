use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

/// A toast with an action button (shadcn's `action: { label: "Undo", onClick }`). Activating
/// it runs the callback and dismisses the toast.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Action toast notifications", ActionButton {} }
    }
}

#[component]
fn ActionButton() -> Element {
    let toast_api = use_toast();
    let mut undone = use_signal(|| 0u32);

    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.75rem;",
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .success(
                            "Event has been created".to_string(),
                            ToastOptions::new()
                                .duration(std::time::Duration::from_secs(60))
                                .action("Undo", move |_| undone += 1),
                        );
                },
                "Show action toast"
            }
            span { "Undone: {undone}" }
        }
    }
}
