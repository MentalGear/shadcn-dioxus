use super::super::component::*;
use dioxus::prelude::*;

// shadcn's separator-demo: a heading and one-line description above the
// separator (`h4` + `p`, text-sm, the muted description colour).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; width: 100%; max-width: 24rem;",
            div { style: "display: flex; flex-direction: column; gap: 0.25rem; align-self: flex-start;",
                h4 { style: "margin: 0; font-size: 0.875rem; font-weight: 500; line-height: 1;",
                    "Radix Primitives"
                }
                p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                    "An open-source UI component library."
                }
            }
            div { style: "display: flex; flex-direction: column; align-items: center; width: 100%;",
                "One thing"
                Separator {
                    style: "margin: 25px 10px; width: 50%;",
                    horizontal: true,
                    decorative: true,
                }
                "Another thing"
            }
        }
    }
}
