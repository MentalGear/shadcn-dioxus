use dioxus::prelude::*;
#[allow(unused_imports, reason = "a consumer-facing surface; this crate's own demos use a subset")]
pub use dioxus_primitives::virtual_grid::{
    use_virtual_scroller, ScrollAlign, ScrollFade, SortDirection, TileRect, VirtualGridProps,
    VirtualJustifiedGridProps, VirtualScroller, VirtualWrapGridProps, VirtualizeMode,
    DEFAULT_AUTO_THRESHOLD,
};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Styled wrapper around the primitive `VirtualGrid` (rows and columns windowed).
#[component]
pub fn VirtualGrid(props: VirtualGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualGrid {
            attributes: merged,
            ..props
        }
    }
}

/// Styled wrapper around the primitive `VirtualWrapGrid` (items wrapping into rows of a fixed
/// column count, rows windowed).
#[component]
pub fn VirtualWrapGrid(props: VirtualWrapGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid dx-virtual-grid-wrap", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualWrapGrid {
            attributes: merged,
            ..props
        }
    }
}

/// Styled wrapper around the primitive `VirtualJustifiedGrid` (a photo wall of justified rows:
/// every tile its item's aspect, rows filling the width, rows and tiles windowed).
#[component]
pub fn VirtualJustifiedGrid(props: VirtualJustifiedGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid dx-virtual-grid-justified", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualJustifiedGrid {
            attributes: merged,
            ..props
        }
    }
}
