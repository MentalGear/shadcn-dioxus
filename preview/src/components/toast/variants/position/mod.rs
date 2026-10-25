use crate::components::button::Button;

use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;

const POSITIONS: [(ToastPosition, &str); 6] = [
    (ToastPosition::TopStart, "Top start"),
    (ToastPosition::TopCenter, "Top center"),
    (ToastPosition::TopEnd, "Top end"),
    (ToastPosition::BottomStart, "Bottom start"),
    (ToastPosition::BottomCenter, "Bottom center"),
    (ToastPosition::BottomEnd, "Bottom end"),
];

/// `position` places the toaster in one of six spots. Each button moves the toaster there and
/// shows a toast in it.
#[component]
pub fn Demo() -> Element {
    let position = use_signal(ToastPosition::default);

    rsx! {
        ToastProvider {
            aria_label: "Positioned toast notifications",
            position,
            PositionButtons { position }
        }
    }
}

#[component]
fn PositionButtons(position: Signal<ToastPosition>) -> Element {
    let toast_api = use_toast();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--dx-space-2)",
            for (spot , label) in POSITIONS {
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| {
                        position.set(spot);
                        toast_api
                            .info(
                                label.to_string(),
                                ToastOptions::new().duration(Duration::from_secs(10)),
                            );
                    },
                    "{label}"
                }
            }
        }
    }
}
