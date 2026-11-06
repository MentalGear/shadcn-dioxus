use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::slider::ThumbCollision;
use std::ops::Range;

#[component]
pub fn Demo() -> Element {
    let mut current_value = use_signal(|| 30.0..40.0);
    let display = use_memo(move || {
        let v = current_value();
        format!("{:.0} – {:.0}", v.start, v.end)
    });

    rsx! {
        div { style: "margin-bottom: 15px; font-size: 16px; font-weight: bold;", "data-testid": "push-value",
            "{display}"
        }

        RangeSlider {
            label: "Pushing Range",
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default_value: 30.0..40.0,
            min_steps_between_thumbs: 5u32,
            thumb_collision: ThumbCollision::Push,
            on_value_change: move |value: Range<f64>| {
                current_value.set(value);
            },
        }
    }
}
