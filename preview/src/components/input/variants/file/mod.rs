use super::super::component::*;
use crate::components::field::{Field, FieldDescription, FieldLabel};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Field {
                FieldLabel { html_for: "input-file-picture", "Picture" }
                Input { id: "input-file-picture", r#type: "file", accept: "image/*" }
                FieldDescription { "Select a picture to upload." }
            }
        }
    }
}
