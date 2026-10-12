use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; width: 100%; max-width: 24rem;",
            Slider {
                label: "Disabled Volume",
                disabled: true,
                min: 0.0,
                max: 100.0,
                step: 1.0,
                default_value: 50.0,
            }
        }
    }
}
