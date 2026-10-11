use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl", style: "width: 100%; max-width: 24rem;",
            DirectionProvider { direction: Direction::Rtl,
                Field { orientation: FieldOrientation::Horizontal,
                    FieldContent {
                        FieldLabel { html_for: "switch-rtl-sync", "المشاركة بين الأجهزة" }
                        FieldDescription {
                            "يتم مشاركة التركيز بين الأجهزة، ويتوقف عند مغادرة التطبيق."
                        }
                    }
                    Switch { id: "switch-rtl-sync" }
                }
            }
        }
    }
}
