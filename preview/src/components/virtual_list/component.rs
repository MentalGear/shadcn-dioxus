use dioxus::prelude::*;
#[allow(unused_imports, reason = "a consumer-facing surface; this crate's own demos use a subset")]
pub use dioxus_primitives::virtual_list::{
    use_virtual_scroller, ScrollAlign, ScrollFade, VirtualListProps, VirtualScroller, VirtualizeMode,
};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Styled wrapper around the primitive `VirtualList`.
#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-list-container"
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        dioxus_primitives::virtual_list::VirtualList {
            attributes: merged,
            ..props
        }
    }
}
