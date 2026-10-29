use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

/// A read-only checkbox stays focusable and is announced, but a click or Space does not change it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox {
                    id: "checkbox-read-only",
                    read_only: true,
                    default_checked: CheckboxState::Checked,
                }
                FieldLabel { html_for: "checkbox-read-only", "Terms accepted (read-only)" }
            }
        }
    }
}
