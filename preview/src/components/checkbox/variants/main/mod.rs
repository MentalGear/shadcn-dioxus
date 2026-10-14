use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Field { orientation: FieldOrientation::Horizontal,
            Checkbox { id: "tos-check", name: "tos-check" }
            FieldLabel { html_for: "tos-check", "Accept terms and conditions" }
        }
    }
}
