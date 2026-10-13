use super::super::component::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use dioxus::prelude::*;

const FAQ: [(&str, &str); 4] = [
    (
        "How do I reset my password?",
        "Choose \"Forgot password\" on the sign-in page and enter your email address. We send a reset link that stays valid for one hour.",
    ),
    (
        "Can I change my plan later?",
        "Yes. Upgrade or downgrade at any time from the billing page. Changes apply on your next invoice, and unused credit carries over.",
    ),
    (
        "Which payment methods do you accept?",
        "We accept all major credit cards, and invoice billing is available on the annual Scale plan.",
    ),
    (
        "Is there a free trial?",
        "Every paid plan starts with a 14 day trial. No card is required until you decide to keep it.",
    ),
];

/// A realistic FAQ: single-open (`allow_multiple_open: false`), every item closed on load.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Accordion { allow_multiple_open: false, horizontal: false,
                for (i , (question , answer)) in FAQ.into_iter().enumerate() {
                    AccordionItem { index: i,
                        AccordionTrigger { "{question}" }
                        AccordionContent {
                            div { padding_bottom: "1rem",
                                p { padding: "0", "{answer}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
