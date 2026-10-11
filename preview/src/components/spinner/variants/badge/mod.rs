use super::super::component::*;
use crate::components::badge::{Badge, BadgeVariant};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { display: "flex", align_items: "center", gap: "1.5rem",
            Badge { variant: BadgeVariant::Secondary,
                Spinner { label: "Syncing" }
                "Syncing"
            }
        }
    }
}
