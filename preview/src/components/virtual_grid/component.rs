use dioxus::prelude::*;
#[allow(unused_imports)] // a consumer-facing surface; this crate's own demos use a subset
pub use dioxus_primitives::virtual_grid::{
    ScrollFade, TileRect, VirtualGridProps, VirtualJustifiedGridProps, VirtualWrapGridProps,
};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Styled wrapper around the primitive `VirtualGrid` (rows and columns windowed).
#[component]
pub fn VirtualGrid(props: VirtualGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualGrid {
            row_count: props.row_count,
            column_count: props.column_count,
            row_height: props.row_height,
            column_width: props.column_width,
            render_cell: props.render_cell,
            render_header: props.render_header,
            header_height: props.header_height,
            sticky_first_column: props.sticky_first_column,
            row_buffer: props.row_buffer,
            column_buffer: props.column_buffer,
            dir: props.dir,
            on_active_change: props.on_active_change,
            fade: props.fade,
            attributes: merged,
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
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualWrapGrid {
            count: props.count,
            columns: props.columns,
            estimate_row_height: props.estimate_row_height,
            render_item: props.render_item,
            buffer: props.buffer,
            dir: props.dir,
            on_active_change: props.on_active_change,
            fade: props.fade,
            attributes: merged,
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
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/virtual_grid/style.css") }
        dioxus_primitives::virtual_grid::VirtualJustifiedGrid {
            aspects: props.aspects,
            render_tile: props.render_tile,
            target_row_height: props.target_row_height,
            gap: props.gap,
            min_tile_width: props.min_tile_width,
            canvas_width: props.canvas_width,
            overscan: props.overscan,
            dir: props.dir,
            on_active_change: props.on_active_change,
            active: props.active,
            fade: props.fade,
            attributes: merged,
        }
    }
}
