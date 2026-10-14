use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `progress-label`: `ProgressLabel` ("Upload progress") and `ProgressValue` (the percent) above the
/// bar. The bar is named by the label with no `aria-labelledby` wiring of your own.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Progress { value: 56.0,
                ProgressLabel { "Upload progress" }
                ProgressValue {}
            }
        }
    }
}
