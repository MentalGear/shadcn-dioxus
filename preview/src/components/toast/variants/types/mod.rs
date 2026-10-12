use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

/// One button per toast type the primitive offers (`success`, `info`, `warning`, `error`).
/// The type only tints the toast (`data-type`); typed icons and a promise/loading toast are
/// not part of the API yet (backlog row 138).
///
/// A provider renders an always-present `role="region"` landmark whose default name is
/// "N notifications", so this page's two providers (the main demo's and this one) would be
/// identical landmarks (axe `landmark-unique`). The `aria_label` below replaces that default
/// (a caller's label wins in the primitive), which makes the two names distinct by
/// construction.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider { aria_label: "Toast type notifications", ToastTypeButtons {} }
    }
}

#[component]
fn ToastTypeButtons() -> Element {
    let toast_api = use_toast();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .success(
                            "Event has been created".to_string(),
                            ToastOptions::new().description("Sunday, December 03, 2023 at 9:00 AM"),
                        );
                },
                "Success"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .info(
                            "Be at the area 10 minutes before the event time".to_string(),
                            ToastOptions::new(),
                        );
                },
                "Notice"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .warning(
                            "Event start time cannot be earlier than 8am".to_string(),
                            ToastOptions::new(),
                        );
                },
                "Warning"
            }
            Button {
                r#type: "button",
                "data-style": "outline",
                onclick: move |_| {
                    toast_api
                        .error(
                            "Event could not be created".to_string(),
                            ToastOptions::new().description("Check your connection and try again."),
                        );
                },
                "Failure"
            }
        }
    }
}
