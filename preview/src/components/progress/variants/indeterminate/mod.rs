use super::super::component::*;
use dioxus::prelude::*;

/// No `value`: the bar has no amount to show, so it sweeps (`data-state="indeterminate"`) and the readout is empty.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Progress { value: None,
                ProgressLabel { "Syncing workspace" }
                ProgressValue {}
            }
        }
    }
}
