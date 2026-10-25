use super::super::component::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

const ITEMS: [(&str, &str, &str); 3] = [
    (
        "overview",
        "Overview",
        "A summary of the account, kept in sync with the buttons below.",
    ),
    (
        "pricing",
        "Pricing",
        "Plans start free and scale with the number of seats.",
    ),
    (
        "support",
        "Support",
        "Answers arrive within one working day, by email or chat.",
    ),
];

/// A controlled accordion (Radix's `value` / `onValueChange`): the open item lives in the demo's own
/// signal, so a button outside the accordion can open and close it, and a click on a trigger only
/// reports the new value, which the demo writes back.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| vec!["overview".to_string()]);
    let mut calls = use_signal(|| 0u32);
    let current = open().first().cloned().unwrap_or_else(|| "none".to_string());

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%; max-width: 15rem;",
            Accordion {
                value: Some(open()),
                on_value_change: move |next: Vec<String>| {
                    calls += 1;
                    open.set(next);
                },
                for (value , title , body) in ITEMS {
                    AccordionItem { value: Some(value.to_string()),
                        AccordionTrigger { "{title}" }
                        AccordionContent {
                            p { padding: "0", "{body}" }
                        }
                    }
                }
            }
            div { style: "display: flex; gap: var(--dx-space-2);",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    r#type: "button",
                    onclick: move |_| open.set(vec!["support".to_string()]),
                    "Open support"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    r#type: "button",
                    onclick: move |_| open.set(Vec::new()),
                    "Close all"
                }
            }
            p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                "Open: {current}. Callback calls: {calls}."
            }
        }
    }
}
