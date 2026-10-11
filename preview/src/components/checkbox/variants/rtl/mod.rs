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
                    Checkbox { id: "checkbox-rtl-terms", name: "checkbox-rtl-terms" }
                    FieldContent {
                        FieldLabel { html_for: "checkbox-rtl-terms", "قبول الشروط والأحكام" }
                        FieldDescription { "بالنقر على مربع الاختيار هذا، فإنك توافق على الشروط والأحكام." }
                    }
                }
            }
        }
    }
}
