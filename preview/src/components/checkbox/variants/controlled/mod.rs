use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

#[component]
pub fn Demo() -> Element {
    let mut state = use_signal(|| CheckboxState::Unchecked);
    let status = if state() == CheckboxState::Checked {
        "checked"
    } else {
        "unchecked"
    };
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Checkbox {
                    id: "checkbox-controlled-updates",
                    checked: Some(state()),
                    on_checked_change: move |next| state.set(next),
                }
                FieldLabel { html_for: "checkbox-controlled-updates", "Email me product updates" }
            }
            p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                "Product updates are {status}."
            }
        }
    }
}
