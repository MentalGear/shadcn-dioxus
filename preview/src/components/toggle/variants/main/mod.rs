use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Bookmark;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Toggle { style: "gap: 0.5rem; padding-inline: 0.75rem;",
            Bookmark {}
            "Bookmark"
        }
    }
}
