use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronLeft, ChevronRight};
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture: a pagination-style group under `dir="rtl"`. The group's
/// logical-property borders/radii mirror, so the first button sits on the
/// right. Arabic copy, as in shadcn's RTL example.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                ButtonGroup { "aria-label": "التنقل بين الصفحات",
                    Button {
                        variant: ButtonVariant::Outline,
                        "aria-label": "الصفحة السابقة",
                        ChevronRight {}
                    }
                    Button { variant: ButtonVariant::Outline, "الصفحة الأولى" }
                    Button { variant: ButtonVariant::Outline, "الصفحة الثانية" }
                    Button {
                        variant: ButtonVariant::Outline,
                        "aria-label": "الصفحة التالية",
                        ChevronLeft {}
                    }
                }
            }
        }
    }
}
