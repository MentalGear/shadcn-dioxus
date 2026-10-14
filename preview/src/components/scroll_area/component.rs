use dioxus::prelude::*;
// Re-exported so `use crate::components::scroll_area::*;` reaches the `fade` prop's enum.
#[allow(unused_imports)] // a consumer-facing surface; this crate's own demos use a subset
pub use dioxus_primitives::scroll_area::ScrollFade;
use dioxus_primitives::scroll_area::{self, ScrollAreaProps};

#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    scroll_area::ScrollArea(props)
}
