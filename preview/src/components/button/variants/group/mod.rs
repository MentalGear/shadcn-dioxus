use super::super::component::*;
use crate::components::button_group::ButtonGroup;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { display: "flex", align_items: "center", gap: "1.5rem",
            ButtonGroup {
                Button { variant: ButtonVariant::Outline, "Archive" }
                Button { variant: ButtonVariant::Outline, "Report" }
                Button { variant: ButtonVariant::Outline, "Snooze" }
            }
        }
    }
}
