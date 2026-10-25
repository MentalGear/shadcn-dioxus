use super::super::component::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use dioxus::prelude::*;

const ITEMS: [(&str, &str); 3] = [
    (
        "Notification Settings",
        "Manage how you receive notifications. You can enable email alerts for updates or push notifications for mobile devices.",
    ),
    (
        "Privacy & Security",
        "Control your privacy settings and security preferences. Enable two-factor authentication, manage connected devices, review active sessions, and configure data sharing preferences. You can also download your data or delete your account.",
    ),
    (
        "Billing & Subscription",
        "View your current plan, payment history, and upcoming invoices. Update your payment method, change your subscription tier, or cancel your subscription.",
    ),
];

/// shadcn's `accordion-multiple` (`<Accordion multiple defaultValue={["notifications"]}>`): with
/// `allow_multiple_open: true` opening an item leaves the others open. The first item starts open.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 32rem;",
            Accordion { allow_multiple_open: true, horizontal: false,
                for (i , (question , answer)) in ITEMS.into_iter().enumerate() {
                    AccordionItem { default_open: i == 0,
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
