use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::form::DemoForm;
use dioxus::prelude::*;
use std::ops::Range;

/// Named sliders inside a form: each thumb submits a hidden input, a reset restores the defaults
/// and `on_value_commit` reports only a finished change.
#[component]
pub fn Demo() -> Element {
    let mut committed = use_signal(|| String::from("none"));

    rsx! {
        DemoForm {
            id: "slider-form",
            show_status: false,
            style: "display: grid; gap: 3rem; width: 100%; max-width: 24rem;",
            Slider {
                label: "Volume",
                name: "volume",
                default_value: 30.0,
                on_value_commit: move |value: f64| committed.set(format!("volume {value}")),
            }
            RangeSlider {
                label: "Price",
                name: "price",
                default_value: 20.0..80.0,
                min_steps_between_thumbs: 10u32,
                on_value_commit: move |value: Range<f64>| {
                    committed.set(format!("price {}-{}", value.start, value.end));
                },
            }
            output { id: "slider-form-committed", "aria-label": "Last committed value", "{committed}" }
            div { style: "display: flex; gap: 0.5rem;",
                Button { r#type: "submit", "Submit" }
                Button { r#type: "reset", variant: ButtonVariant::Outline, "Reset" }
            }
        }
    }
}
