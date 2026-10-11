use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::FolderCode;
use dioxus_primitives::direction::{Direction, DirectionProvider};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                Empty { style: "border: 1px dashed var(--dx-border); width: 100%; max-width: 28rem;",
                    EmptyHeader {
                        EmptyMedia { variant: EmptyMediaVariant::Icon,
                            FolderCode {}
                        }
                        EmptyTitle { "لا توجد مشاريع بعد" }
                        EmptyDescription {
                            "لم تنشئ أي مشاريع بعد. ابدأ بإنشاء مشروعك الأول."
                        }
                    }
                    EmptyContent {
                        Button { size: ButtonSize::Sm, "إنشاء مشروع" }
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, "استيراد مشروع" }
                    }
                }
            }
        }
    }
}
