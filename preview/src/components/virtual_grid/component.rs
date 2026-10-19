use dioxus::prelude::*;
#[allow(unused_imports)] // a consumer-facing surface; this crate's own demos use a subset
pub use dioxus_primitives::virtual_grid::{VirtualGridProps, VirtualWrapGridProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Styled wrapper around the primitive `VirtualGrid` (rows and columns windowed).
#[component]
pub fn VirtualGrid(props: VirtualGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
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
            attributes: merged,
        }
    }
}

/// Styled wrapper around the primitive `VirtualWrapGrid` (items wrapping into rows of a fixed
/// column count, rows windowed).
#[component]
pub fn VirtualWrapGrid(props: VirtualWrapGridProps) -> Element {
    let base = attributes!(div {
        class: "dx-virtual-grid dx-virtual-grid-wrap"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        dioxus_primitives::virtual_grid::VirtualWrapGrid {
            count: props.count,
            columns: props.columns,
            estimate_row_height: props.estimate_row_height,
            render_item: props.render_item,
            buffer: props.buffer,
            dir: props.dir,
            on_active_change: props.on_active_change,
            attributes: merged,
        }
    }
}
