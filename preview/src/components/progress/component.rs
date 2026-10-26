use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::progress::{self, ProgressLabelProps, ProgressProps, ProgressValueProps};

/// A progress bar. The root is the `progressbar` and a flex-wrap row (shadcn's `cn-progress-root`,
/// `flex flex-wrap gap-3`): put a [`ProgressLabel`] and a [`ProgressValue`] in it and they sit on the first
/// line, the label at the start and the value at the end, above the full-width track. With neither, it is the
/// bare 4px track.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let base = attributes!(div { class: "dx-progress", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/progress/style.css") }
        progress::Progress {
            attributes: merged,
            ..props,
            {children}
            div { class: "dx-progress-track",
                progress::ProgressIndicator { class: "dx-progress-indicator" }
            }
        }
    }
}

/// The visible name of the [`Progress`] (`text-sm font-medium`); the bar is named by it automatically.
#[component]
pub fn ProgressLabel(props: ProgressLabelProps) -> Element {
    let base = attributes!(span { class: "dx-progress-label" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/progress/style.css") }
        progress::ProgressLabel {
            attributes: merged,
            ..props
        }
    }
}

/// The percent readout of the [`Progress`] (`56%`; `text-muted-foreground`, tabular figures, pushed to the
/// end of the row). Empty while the progress is indeterminate.
#[component]
pub fn ProgressValue(props: ProgressValueProps) -> Element {
    let base = attributes!(span { class: "dx-progress-value" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/progress/style.css") }
        progress::ProgressValue {
            attributes: merged,
            ..props
        }
    }
}
