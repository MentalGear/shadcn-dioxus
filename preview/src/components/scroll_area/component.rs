use dioxus::prelude::*;
// Re-exported so `use crate::components::scroll_area::*;` reaches the `fade` prop's enum.
#[allow(unused_imports, reason = "a consumer-facing surface; this crate's own demos use a subset")]
pub use dioxus_primitives::scroll_area::ScrollFade;
use dioxus_primitives::scroll_area::{self, ScrollAreaProps};

// The sheet gives `always_show_scrollbars` its effect (the visibility classes the primitive sets); see
// checkbox/component.rs's header comment for the asset!() + document::Link delivery (row 32).
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/scroll_area/style.css") }
        scroll_area::ScrollArea {
            ..props,
 ..props
        }
    }
}
