use super::super::component::*;
use dioxus::prelude::*;

/// The typed `size`: 12, 16 (default), 24 and 32 px.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1.5rem;",
            Spinner { size: SpinnerSize::Sm, label: "Loading small" }
            Spinner { label: "Loading default" }
            Spinner { size: SpinnerSize::Lg, label: "Loading large" }
            Spinner { size: SpinnerSize::Xl, label: "Loading extra large" }
        }
    }
}
