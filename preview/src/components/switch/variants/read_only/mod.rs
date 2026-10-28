use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

/// A read-only switch stays focusable and is announced, but a click or Space does not change it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Field { orientation: FieldOrientation::Horizontal,
                Switch {
                    id: "switch-read-only",
                    read_only: true,
                    default_checked: true,
                }
                FieldLabel { html_for: "switch-read-only", "Managed by your admin (read-only)" }
            }
        }
    }
}
