use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuTrigger,
};
use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{BadgeCheck, Bell, CreditCard, LogOut, Sparkles};

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");

/// shadcn's avatar-trigger account menu: the trigger is a ghost icon `Button`
/// wrapping an `ImageAvatar` (via the trigger's `as` render prop), and the menu
/// opens with a header carrying the avatar, name and email, then the account
/// items, each with a leading icon.
///
/// The header is a `DropdownMenuLabel`. shadcn also draws a `Separator` between
/// the header, the "Upgrade" group and the account group; the component has no
/// separator or group part yet (backlog row 155 group 2), so the items simply
/// follow the header in one list.
#[component]
pub fn Demo() -> Element {
    let mut chosen = use_signal(|| None::<String>);

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger {
                "aria-label": "Account switcher",
                as: |attributes: Vec<Attribute>| rsx! {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        attributes,
                        ImageAvatar {
                            size: AvatarImageSize::Small,
                            src: AVERY_LIN.to_string(),
                            alt: "Avery Lin",
                            "AL"
                        }
                    }
                },
            }
            DropdownMenuContent {
                DropdownMenuLabel {
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem; min-width: 14rem;",
                        ImageAvatar {
                            size: AvatarImageSize::Small,
                            src: AVERY_LIN.to_string(),
                            alt: "Avery Lin",
                            "AL"
                        }
                        div { style: "display: grid; line-height: 1.25;",
                            span { style: "color: var(--dx-foreground);", "Avery Lin" }
                            span { "avery@example.com" }
                        }
                    }
                }
                DropdownMenuItem::<String> {
                    value: "Upgrade to Pro".to_string(),
                    index: 0usize,
                    text_value: "Upgrade to Pro",
                    on_select: move |value| chosen.set(Some(value)),
                    Sparkles { size: "16px" }
                    "Upgrade to Pro"
                }
                DropdownMenuItem::<String> {
                    value: "Account".to_string(),
                    index: 1usize,
                    text_value: "Account",
                    on_select: move |value| chosen.set(Some(value)),
                    BadgeCheck { size: "16px" }
                    "Account"
                }
                DropdownMenuItem::<String> {
                    value: "Billing".to_string(),
                    index: 2usize,
                    text_value: "Billing",
                    on_select: move |value| chosen.set(Some(value)),
                    CreditCard { size: "16px" }
                    "Billing"
                }
                DropdownMenuItem::<String> {
                    value: "Notifications".to_string(),
                    index: 3usize,
                    text_value: "Notifications",
                    on_select: move |value| chosen.set(Some(value)),
                    Bell { size: "16px" }
                    "Notifications"
                }
                DropdownMenuItem::<String> {
                    value: "Log out".to_string(),
                    index: 4usize,
                    text_value: "Log out",
                    on_select: move |value| chosen.set(Some(value)),
                    LogOut { size: "16px" }
                    "Log out"
                }
            }
        }
        if let Some(item) = chosen() {
            p { style: "margin-top: 0.75rem;", "Chose: {item}" }
        }
    }
}
