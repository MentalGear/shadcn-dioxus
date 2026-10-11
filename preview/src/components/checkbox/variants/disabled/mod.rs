use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox { id: "checkbox-disabled-unchecked", disabled: true }
                FieldLabel { html_for: "checkbox-disabled-unchecked", "Enable notifications" }
            }
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox {
                    id: "checkbox-disabled-checked",
                    disabled: true,
                    default_checked: CheckboxState::Checked,
                }
                FieldLabel { html_for: "checkbox-disabled-checked", "Sync across devices" }
            }
        }
    }
}
