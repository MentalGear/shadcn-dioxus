use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: a bare spinner and a disabled button with a spinner, under
/// `dir="rtl"`, with Arabic copy as in shadcn's RTL example.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                div { display: "flex", align_items: "center", gap: "1.5rem",
                    Spinner { label: "جارٍ التحميل" }
                    Button { variant: ButtonVariant::Outline, disabled: true,
                        Spinner { label: "جارٍ الإنشاء" }
                        "جارٍ الإنشاء..."
                    }
                }
            }
        }
    }
}
