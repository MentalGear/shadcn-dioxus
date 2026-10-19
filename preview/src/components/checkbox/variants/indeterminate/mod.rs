use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox {
                    id: "checkbox-indeterminate",
                    default_checked: CheckboxState::Indeterminate,
                }
                FieldLabel { html_for: "checkbox-indeterminate", "Some items selected" }
            }
        }
    }
}
