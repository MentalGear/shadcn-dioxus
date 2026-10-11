use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

/// The open state lives in a signal the page owns: the trigger and an external button both flip it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { style: "width: 100%; max-width: 20rem;",
            Collapsible {
                open: Some(open()),
                on_open_change: move |next: bool| open.set(next),
                CollapsibleTrigger {
                    b { "@peduarte starred 3 repositories" }
                }
                CollapsibleList {
                    CollapsibleItem { "@radix-ui/primitives" }
                    CollapsibleContent {
                        CollapsibleItem { "@radix-ui/colors" }
                        CollapsibleItem { "@stitches/react" }
                    }
                }
            }
            div { style: "display: flex; align-items: center; gap: 0.75rem; margin-top: 0.75rem;",
                Button {
                    variant: ButtonVariant::Outline,
                    r#type: "button",
                    onclick: move |_| open.toggle(),
                    if open() {
                        "Collapse from outside"
                    } else {
                        "Expand from outside"
                    }
                }
                span { style: "font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);",
                    if open() {
                        "State: open"
                    } else {
                        "State: closed"
                    }
                }
            }
        }
    }
}
