use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldLabel, FieldOrientation,
};
use dioxus::prelude::*;

// `aria-invalid` on the control draws the shared invalid recipe (a destructive
// edge and ring, `--dx-ring-invalid-field` in the theme, as shadcn does);
// `Field`'s `invalid` turns the label and the error text red.
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
