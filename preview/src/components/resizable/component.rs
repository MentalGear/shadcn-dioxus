use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::resizable::{
    self, ResizableHandleProps, ResizablePanelGroupProps, ResizablePanelProps,
};

// Plain enum, not markup -- re-exported so demo code never needs its own
// `dioxus_primitives::resizable::` import (scripts/check-preview-composition.sh's
// allowlist covers this shape for other components, e.g. `ContentSide`, via a
// module-root re-export from the themed wrapper; doing the same here keeps every
// non-`component.rs` file in this crate free of raw `dioxus_primitives::` text at
// all, which is simpler than adding an allowlist entry to that script).
pub use dioxus_primitives::resizable::ResizableDirection;

#[component]
pub fn ResizablePanelGroup(props: ResizablePanelGroupProps) -> Element {
    let base = attributes!(div { class: "dx-resizable-panel-group" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/resizable/style.css") }
        resizable::ResizablePanelGroup {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ResizablePanel(props: ResizablePanelProps) -> Element {
    let base = attributes!(div { class: "dx-resizable-panel" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/resizable/style.css") }
        resizable::ResizablePanel {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ResizableHandle(props: ResizableHandleProps) -> Element {
    let base = attributes!(div { class: "dx-resizable-handle" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/resizable/style.css") }
        resizable::ResizableHandle {
            attributes: merged,
            ..props
        }
    }
}
