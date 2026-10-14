use super::super::component::*;
use crate::components::field::{Field, FieldGroup, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

/// shadcn's `switch-sizes`: a `sm` and a `default` switch, each labelled by a `Field`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 10rem;",
            FieldGroup {
                Field { orientation: FieldOrientation::Horizontal,
                    Switch { id: "switch-size-sm", size: SwitchSize::Sm }
                    FieldLabel { html_for: "switch-size-sm", "Small" }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    Switch { id: "switch-size-default", size: SwitchSize::Default }
                    FieldLabel { html_for: "switch-size-default", "Default" }
                }
            }
        }
    }
}
