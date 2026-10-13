use super::super::component::*;
use crate::components::{input::Input, label::Label};
use dioxus::prelude::*;

/// The `main` demo's "Dimensions" form as an explicitly **modal** popover
/// (`is_modal: true`). Unlike the default (non-modal, shadcn's behaviour) a modal
/// popover is a native `<dialog>` opened with `showModal()`: focus is trapped, the
/// page behind it is inert, the page scroll is locked while it is open, and a
/// click outside only dismisses it (it does not reach the page). Like shadcn's
/// popover it does not dim the page: `overlay` defaults to `false` (the `overlay`
/// variant turns the shared scrim on).
///
/// This is the subject of every spec that needs the modal arm (focus trap, inert
/// page, scroll lock, backdrop dismiss, the native-dialog oracle's Rules 6 and 8).
/// The ids are prefixed `popover-modal-` so they never collide with `main`'s on
/// the shared component page.
#[component]
pub fn Demo() -> Element {
    rsx! {
        PopoverRoot { is_modal: true,
            PopoverTrigger { "Open modal popover" }
            PopoverContent { width: "20rem",
                div { display: "grid", gap: "1rem",
                    PopoverHeader { gap: "0.5rem",
                        PopoverTitle { "Dimensions" }
                        PopoverDescription { "Set the dimensions for the layer." }
                    }
                    div { display: "grid", gap: "0.5rem",
                        DimensionRow { id: "popover-modal-demo-width", label: "Width", value: "100%" }
                        DimensionRow { id: "popover-modal-demo-max-width", label: "Max. width", value: "300px" }
                        DimensionRow { id: "popover-modal-demo-height", label: "Height", value: "25px" }
                        DimensionRow { id: "popover-modal-demo-max-height", label: "Max. height", value: "none" }
                    }
                }
            }
        }
    }
}

/// One `grid-cols-3 items-center gap-4` row of shadcn's demo: the label in the
/// first column, the input spanning the other two.
#[component]
fn DimensionRow(id: &'static str, label: &'static str, value: &'static str) -> Element {
    rsx! {
        div {
            display: "grid",
            grid_template_columns: "repeat(3, minmax(0, 1fr))",
            align_items: "center",
            gap: "1rem",
            Label { html_for: id, "{label}" }
            Input { id, initial_value: value, grid_column: "span 2 / span 2" }
        }
    }
}
