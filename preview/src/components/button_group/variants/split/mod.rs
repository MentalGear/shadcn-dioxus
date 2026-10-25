use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::ChevronDown;

/// shadcn's "Split" example: a primary "Follow" button plus a chevron button
/// that opens a `DropdownMenu` of related actions.
///
/// The chevron trigger is a `Button` (via the trigger's `as` render prop) and
/// joins the group's seam through the `--dx-group-radius-*` variables (see
/// `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    let mut last_action = use_signal(|| None::<String>);
    let items = ["Mute conversation", "Mark as read", "Block user"];
    rsx! {
        ButtonGroup { "aria-label": "Follow",
            Button { variant: ButtonVariant::Secondary, "Follow" }
            ButtonGroupSeparator {}
            DropdownMenu {
                // `as` renders the trigger AS a `Button` (the same pattern as
                // `SheetClose`), so it takes the neighbouring button's height and
                // variant.
                DropdownMenuTrigger {
                    "aria-label": "More follow options",
                    as: |el: AsChild| rsx! {
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Icon,
                            attributes: el.attributes,
                            ChevronDown {}
                        }
                    },
                }
                DropdownMenuContent {
                    for label in items.into_iter() {
                        DropdownMenuItem::<String> {
                            value: label.to_string(),
                            text_value: label,
                            on_select: move |value: String| last_action.set(Some(value)),
                            "{label}"
                        }
                    }
                }
            }
        }
        if let Some(action) = last_action() {
            p { style: "margin-top: 0.75rem;", "Selected: {action}" }
        }
    }
}
