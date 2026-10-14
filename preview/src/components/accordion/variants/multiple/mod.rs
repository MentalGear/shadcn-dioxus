use super::super::component::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use dioxus::prelude::*;

const ITEMS: [(&str, &str); 3] = [
    (
        "What are your support hours?",
        "Our team answers email and chat from 8am to 8pm Eastern, Monday to Friday.",
    ),
    (
        "Do you offer onboarding help?",
        "Every team gets a 30 minute walkthrough, and larger teams can book a dedicated session.",
    ),
    (
        "Where is my data stored?",
        "Data lives in encrypted storage in the region you pick when you create your workspace.",
    ),
];

/// `allow_multiple_open: true`: opening an item leaves the others open. Every item starts closed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Accordion { allow_multiple_open: true, horizontal: false,
                for (i , (question , answer)) in ITEMS.into_iter().enumerate() {
                    AccordionItem { index: i,
                        AccordionTrigger { "{question}" }
                        AccordionContent {
                            p { padding: "0", "{answer}" }
                        }
                    }
                }
            }
        }
    }
}
