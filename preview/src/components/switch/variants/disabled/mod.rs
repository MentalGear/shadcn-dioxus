use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Switch { id: "switch-disabled-airplane", disabled: true }
                FieldLabel { html_for: "switch-disabled-airplane", "Airplane mode" }
            }
        }
    }
}
