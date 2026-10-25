use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::{BadgeCheck, ChevronLeft};
use dioxus_primitives::direction::{Direction, DirectionProvider};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            dir: "rtl",
            display: "flex",
            flex_direction: "column",
            gap: "1.5rem",
            width: "100%",
            max_width: "28rem",
            DirectionProvider { direction: Direction::Rtl,
                Item { variant: ItemVariant::Outline,
                    ItemContent {
                        ItemTitle { "عنصر أساسي" }
                        ItemDescription { "عنصر بسيط يحتوي على عنوان ووصف." }
                    }
                    ItemActions {
                        Button { variant: ButtonVariant::Outline, "إجراء" }
                    }
                }

                Item {
                    variant: ItemVariant::Outline,
                    size: ItemSize::Sm,
                    as: move |el: AsChild| rsx! {
                        a { href: "#", ..el.attributes,
                            {el.children}
                        }
                    },
                    ItemMedia { variant: ItemMediaVariant::Icon,
                        BadgeCheck { size: "20" }
                    }
                    ItemContent {
                        ItemTitle { "تم التحقق من ملفك الشخصي." }
                    }
                    ItemActions {
                        ChevronLeft { size: "16" }
                    }
                }
            }
        }
    }
}
