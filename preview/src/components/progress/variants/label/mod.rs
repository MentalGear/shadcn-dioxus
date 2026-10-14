use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's labelled progress: an "Uploading..." label and a percentage readout
/// above the bar. Both are plain text nodes; the bar is named by the label
/// through `aria-labelledby`. (No `ProgressLabel`/`ProgressValue` parts exist
/// yet, so this is spans and a flex row.)
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 24rem;",
            div { style: "display: flex; justify-content: space-between; font-size: 14px; font-weight: 500;",
                span { id: "progress-label-uploading", "Uploading..." }
                span { style: "color: var(--dx-muted-foreground); font-variant-numeric: tabular-nums;", "66%" }
            }
            Progress { aria_labelledby: "progress-label-uploading", value: 66.0 }
        }
    }
}
