use super::super::component::*;
use dioxus::prelude::*;

// shadcn's separator-demo: a heading and one-line description, a horizontal
// separator (`my-4`), then a `h-5 gap-4 text-sm` row of three links split by
// vertical separators. The title is a `div`, not an `h4`: the docs page has no
// `h3` above the demo, so a heading would skip a level (axe `heading-order`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            div { style: "display: flex; flex-direction: column; gap: 0.25rem;",
                div { style: "font-size: 0.875rem; font-weight: 500; line-height: 1;",
                    "Radix Primitives"
                }
                p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                    "An open-source UI component library."
                }
            }
            Separator { style: "margin: 1rem 0;", horizontal: true, decorative: true }
            div { style: "display: flex; height: 1.25rem; align-items: center; gap: 1rem; font-size: 0.875rem;",
                div { "Blog" }
                Separator { horizontal: false, decorative: true }
                div { "Docs" }
                Separator { horizontal: false, decorative: true }
                div { "Source" }
            }
        }
    }
}
