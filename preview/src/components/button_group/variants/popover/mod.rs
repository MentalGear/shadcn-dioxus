use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::popover::{
    PopoverContent, PopoverDescription, PopoverHeader, PopoverRoot, PopoverTitle, PopoverTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;

/// shadcn's "Popover" example: a "Copilot" button beside a chevron
/// `PopoverTrigger` that opens a popover panel.
///
/// The chevron trigger joins the group's seam through the `--dx-group-radius-*`
/// variables (see `button_group/docs.md`). Its icon is sized 16px by hand:
/// `PopoverTrigger` does not size icons the way `.dx-button svg` does, and the
/// icon's own 24px would make the trigger taller than the button beside it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { "aria-label": "Assistant",
            Button { variant: ButtonVariant::Outline, "Assistant" }
            PopoverRoot {
                PopoverTrigger { "aria-label": "Assistant options", ChevronDown { size: "16px" } }
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
