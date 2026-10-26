use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::slider::{self, RangeSliderProps, SliderProps};

#[component]
pub fn Slider(props: SliderProps) -> Element {
    let base = attributes!(div { class: "dx-slider", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/slider/style.css") }
        slider::Slider {
            attributes: merged,
            ..props,
            slider::SliderTrack { class: "dx-slider-track",
                slider::SliderRange { class: "dx-slider-range" }
                slider::SliderThumb { class: "dx-slider-thumb" }
            }
        }
    }
}

#[component]
pub fn RangeSlider(props: RangeSliderProps) -> Element {
    let base = attributes!(div { class: "dx-slider", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/slider/style.css") }
        slider::RangeSlider {
            attributes: merged,
            ..props,
            slider::SliderTrack { class: "dx-slider-track",
                slider::SliderRange { class: "dx-slider-range" }
                slider::SliderThumb { class: "dx-slider-thumb", index: 0usize }
                slider::SliderThumb { class: "dx-slider-thumb", index: 1usize }
            }
        }
    }
}
