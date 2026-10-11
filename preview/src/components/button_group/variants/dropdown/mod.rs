use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::Ellipsis;

/// shadcn's "Dropdown Menu" example: a "Follow" button beside an overflow
/// button that opens a `DropdownMenu` of items.
///
/// The overflow trigger is a `Button` (via the trigger's `as` render prop) and
/// joins the group's seam through the `--dx-group-radius-*` variables (see
/// `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    let mut last_action = use_signal(|| None::<String>);
    let items = ["Add to favorites", "Move to folder", "Share link"];
    rsx! {
        ButtonGroup { "aria-label": "Message actions",
            Button { variant: ButtonVariant::Outline, "Reply" }
            DropdownMenu {
                // `as` renders the trigger AS a `Button` (the same pattern as
                // `SheetClose`), so it matches the neighbouring button.
                DropdownMenuTrigger {
                    "aria-label": "More message actions",
                    as: |attributes: Vec<Attribute>| rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Icon,
                            attributes,
                            Ellipsis {}
                        }
                    },
                }
                DropdownMenuContent {
                    for (i , label) in items.into_iter().enumerate() {
                        DropdownMenuItem::<String> {
                            value: label.to_string(),
                            index: i,
                            text_value: label,
                            on_select: move |value: String| last_action.set(Some(value)),
                            "{label}"
                        }
                    }
                }
            }
        }
        if let Some(action) = last_action() {
            p { style: "margin-top: 0.75rem;", "Chose: {action}" }
        }
    }
}
