use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::slider::ThumbAlignment;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 2rem; width: 20rem;",
            Slider {
                label: "Edge Slider",
                min: 0.0,
                max: 100.0,
                step: 1.0,
                default_value: 0.0,
                thumb_alignment: ThumbAlignment::Edge,
            }
            RangeSlider {
                label: "Edge Range",
                min: 0.0,
                max: 100.0,
                step: 1.0,
                default_value: 0.0..100.0,
                thumb_alignment: ThumbAlignment::Edge,
            }
        }
    }
}
