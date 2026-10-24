use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::ChevronDown;

/// A dropdown menu trigger living in the trailing addon, choosing the input's scope.
#[component]
pub fn Demo() -> Element {
    let mut scope = use_signal(|| "Everywhere".to_string());
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 20rem;",
            InputGroup {
                InputGroupInput {
                    id: "input-group-dropdown-query",
                    placeholder: "Search",
                    "aria-label": "Search term",
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    DropdownMenu {
                        // `as` renders the trigger AS a compact ghost `Button` (xs, like
                        // shadcn's `InputGroupButton`): 14px text, 12px chevron, 4px gap.
                        DropdownMenuTrigger {
                            as: move |el: AsChild| rsx! {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Xs,
                                    attributes: el.attributes,
                                    "{scope}"
                                    ChevronDown {}
                                }
                            },
                        }
                        DropdownMenuContent {
                            for (i , name) in ["Everywhere", "This project", "Archived"].into_iter().enumerate() {
                                DropdownMenuItem::<String> {
                                    value: name.to_string(),
                                    index: i,
                                    text_value: name,
                                    on_select: move |value: String| scope.set(value),
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
