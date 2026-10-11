use super::super::component::*;
use crate::components::field::{
    Field, FieldDescription, FieldGroup, FieldLabel, FieldLegend, FieldOrientation, FieldSet,
};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

const ITEMS: &[(&str, &str)] = &[
    ("recents", "Recents"),
    ("home", "Home"),
    ("applications", "Applications"),
    ("desktop", "Desktop"),
    ("downloads", "Downloads"),
    ("documents", "Documents"),
];

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldSet {
                FieldLegend { "Show these items on the desktop" }
                FieldDescription { "Select the items you want to show on the desktop." }
                FieldGroup {
                    for (id , label) in ITEMS.iter() {
                        Field { key: "{id}", orientation: FieldOrientation::Horizontal,
                            Checkbox {
                                id: "checkbox-group-{id}",
                                name: "checkbox-group-{id}",
                                default_checked: if *id == "recents" { CheckboxState::Checked } else { CheckboxState::Unchecked },
                            }
                            FieldLabel { html_for: "checkbox-group-{id}", "{label}" }
                        }
                    }
                }
            }
        }
    }
}
