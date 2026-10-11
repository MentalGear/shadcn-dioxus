use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use dioxus::prelude::*;

/// shadcn's "Input" example: a text input and a "Search" button in one group.
///
/// The input joins the group's seam through the `--dx-group-radius-*`
/// variables (see `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { "aria-label": "Search", style: "width: 100%;",
            Input {
                id: "button-group-with-input-search",
                placeholder: "Search...",
                "aria-label": "Search query",
                style: "flex: 1;",
            }
            Button { variant: ButtonVariant::Outline, "Search" }
        }
    }
}
