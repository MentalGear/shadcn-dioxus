use dioxus::prelude::*;
#[allow(unused_imports)] // a consumer-facing surface; this crate's own demos use a subset
pub use dioxus_primitives::virtual_list::{ScrollFade, VirtualListProps, VirtualizeMode};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Styled wrapper around the primitive `VirtualList`.
#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-list-container"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        dioxus_primitives::virtual_list::VirtualList {
            count: props.count,
            buffer: props.buffer,
            estimate_size: props.estimate_size,
            render_item: props.render_item,
            virtualize: props.virtualize,
            auto_threshold: props.auto_threshold,
            fade: props.fade,
            text_value: props.text_value,
            attributes: merged,
        }
    }
}
