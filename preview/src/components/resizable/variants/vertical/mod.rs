use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::GripVertical;

fn panel_content(label: &'static str) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; height: 100%; padding: var(--dx-space-6); box-sizing: border-box;",
            span { style: "font-weight: 600;", "{label}" }
        }
    }
}

fn handle_grip() -> Element {
    rsx! {
        span { class: "dx-resizable-handle-grip", GripVertical { size: "0.625rem" } }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        ResizablePanelGroup {
            direction: ResizableDirection::Vertical,
            height: "20rem",
            max_width: "28rem",
            border: "1px solid var(--dx-input)",
            border_radius: "var(--dx-radius-lg)",
            overflow: "hidden",
            ResizablePanel { index: 0usize, default_size: 40.0, min_size: 20.0,
                {panel_content("Editor")}
            }
            ResizableHandle { index: 0usize, aria_label: "Editor", {handle_grip()} }
            ResizablePanel { index: 1usize, default_size: 60.0, min_size: 20.0,
                {panel_content("Output")}
            }
        }
    }
}
