use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::popover::{
    PopoverContent, PopoverDescription, PopoverHeader, PopoverRoot, PopoverTitle,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::popover::PopoverTrigger;

/// shadcn's "Popover" example: a "Copilot" button beside a chevron
/// `PopoverTrigger` that opens a popover panel.
///
/// The chevron trigger is shadcn's `<Button variant="outline" size="icon">`: the
/// popover primitive's bare trigger button wearing the Button recipe (no second
/// `<button>` nests inside it), so it is the same 32px square with a 16px icon
/// as an icon Button, and it joins the group's seam through the
/// `--dx-group-radius-*` variables (see `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { "aria-label": "Assistant",
            Button { variant: ButtonVariant::Outline, "Assistant" }
            PopoverRoot {
                PopoverTrigger {
                    class: "dx-button",
                    "data-style": "outline",
                    "data-size": "icon",
                    "aria-label": "Assistant options",
                    ChevronDown {}
                }
                PopoverContent { width: "18rem",
                    PopoverHeader {
                        PopoverTitle { "Start a new task" }
                        PopoverDescription {
                            "Describe what you want done and the assistant will draft a plan for you to review."
                        }
                    }
                }
            }
        }
    }
}
