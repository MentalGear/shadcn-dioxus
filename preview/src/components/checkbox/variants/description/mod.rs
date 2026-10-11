use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox { id: "checkbox-description-terms", name: "checkbox-description-terms" }
                FieldContent {
                    FieldLabel { html_for: "checkbox-description-terms", "Accept terms and conditions" }
                    FieldDescription {
                        "By clicking this checkbox, you agree to the terms and conditions."
                    }
                }
            }
        }
    }
}
