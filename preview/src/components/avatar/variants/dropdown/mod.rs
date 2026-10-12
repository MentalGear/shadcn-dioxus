use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;

const SARAH_CHEN: Asset = asset!("/assets/avatars/sarah-chen.svg");

/// An avatar as a dropdown trigger: `DropdownMenuTrigger` renders `as` a ghost icon `Button` that
/// wraps the avatar, and the button (not the avatar) carries the accessible name.
#[component]
pub fn Demo() -> Element {
    let mut selected = use_signal(|| "Nothing selected yet".to_string());
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem;",
            DropdownMenu {
                DropdownMenuTrigger {
                    as: move |attributes: Vec<Attribute>| rsx! {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            "aria-label": "Open account menu",
                            attributes,
                            ImageAvatar {
                                size: AvatarImageSize::Small,
                                src: SARAH_CHEN.to_string(),
                                alt: "Sarah Chen",
                                "SC"
                            }
                        }
                    },
                }
                DropdownMenuContent {
                    for (i , name) in ["Profile", "Billing", "Settings", "Sign out"].into_iter().enumerate() {
                        DropdownMenuItem::<String> {
                            value: name.to_string(),
                            index: i,
                            text_value: name,
                            on_select: move |value: String| selected.set(value),
                            "{name}"
                        }
                    }
                }
            }
            p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "Account menu: {selected}"
            }
        }
    }
}
