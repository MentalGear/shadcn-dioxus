use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem; height: 1.25rem;",
            span { "Blog" }
            Separator { horizontal: false, decorative: true }
            span { "Docs" }
            Separator { horizontal: false, decorative: true }
            span { "Source" }
        }
    }
}
