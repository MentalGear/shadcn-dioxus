use super::super::component::*;
use crate::components::input::Input;
use dioxus::prelude::*;

/// `keep_mounted`: closed content stays in the DOM under the `hidden` attribute, so the field inside keeps
/// what was typed while the panel was closed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Collapsible { keep_mounted: true,
                CollapsibleTrigger {
                    b { "Advanced search" }
                }
                CollapsibleContent {
                    Input {
                        id: "collapsible-keep-mounted-query",
                        placeholder: "Search query",
                        "aria-label": "Search query",
                    }
                }
            }
            p { style: "margin: 0.75rem 0 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                "Type a query, close the panel and reopen it: the text is still there."
            }
        }
    }
}
