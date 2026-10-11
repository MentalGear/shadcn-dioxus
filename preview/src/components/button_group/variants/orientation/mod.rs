use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Minus, Plus};

/// shadcn's "Orientation" example: a vertical group of icon buttons (zoom in /
/// zoom out) -- `orientation_vertical` stacks the buttons and merges their
/// shared horizontal edges.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { orientation_vertical: true, "aria-label": "Zoom controls",
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                "aria-label": "Zoom in",
                Plus {}
            }
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                "aria-label": "Zoom out",
                Minus {}
            }
        }
    }
}
