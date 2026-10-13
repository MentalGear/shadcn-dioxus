use super::super::component::*;
use crate::components::spinner::Spinner;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { class: "dx-badge-example",
            Badge { variant: BadgeVariant::Outline,
                Spinner {}
                "Syncing"
            }
        }
    }
}
