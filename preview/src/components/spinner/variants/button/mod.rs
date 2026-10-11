use super::super::component::*;
use crate::components::button::Button;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { display: "flex", align_items: "center", gap: "1.5rem",
            Button { disabled: true,
                Spinner { label: "Generating" }
                "Generating..."
            }
        }
    }
}
