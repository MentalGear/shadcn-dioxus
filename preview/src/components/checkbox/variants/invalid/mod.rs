use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

// The themed `Checkbox` has no `[aria-invalid]` style (shadcn draws a
// destructive border + ring on it), so only the `Field`'s label turns red
// here; the control itself keeps its default border.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal, invalid: true,
                Checkbox {
                    id: "checkbox-invalid-terms",
                    required: true,
                    "aria-invalid": "true",
                }
                FieldContent {
                    FieldLabel { html_for: "checkbox-invalid-terms", "Accept terms and conditions" }
                    FieldDescription { "You must accept the terms before you can continue." }
                    FieldError { "Please accept the terms and conditions to continue." }
                }
            }
        }
    }
}
