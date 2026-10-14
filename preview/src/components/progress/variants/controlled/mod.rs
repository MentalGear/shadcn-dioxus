use super::super::component::*;
use crate::components::slider::Slider;
use dioxus::prelude::*;

/// A determinate bar driven by a slider: the slider writes one signal, and the
/// bar and its percentage readout both read it.
#[component]
pub fn Demo() -> Element {
    let mut progress = use_signal(|| 40.0_f64);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1.5rem; width: 100%; max-width: 24rem;",
            Progress { aria_label: "Upload progress", value: progress() }
            div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
                div { style: "display: flex; justify-content: space-between; font-size: 13px; font-weight: 500;",
                    span { "Progress" }
                    span { style: "color: var(--dx-muted-foreground); font-variant-numeric: tabular-nums;",
                        "{progress():.0}%"
                    }
                }
                Slider {
                    label: "Progress value",
                    horizontal: true,
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    default_value: 40.0,
                    on_value_change: move |value: f64| progress.set(value),
                }
            }
        }
    }
}
