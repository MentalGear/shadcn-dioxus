use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; justify-content: center; height: 10rem;",
            Slider {
                label: "Vertical Volume",
                horizontal: false,
                min: 0.0,
                max: 100.0,
                step: 1.0,
                default_value: 30.0,
            }
        }
    }
}
