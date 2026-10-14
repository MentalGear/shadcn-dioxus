use super::super::component::*;
use dioxus::prelude::*;

/// A disabled trigger: dimmed, skipped by the arrow keys and not activatable.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "account".to_string(),
            horizontal: true,
            max_width: "24rem",
            TabList {
                TabTrigger { value: "account".to_string(), index: 0usize, "Account" }
                TabTrigger { value: "billing".to_string(), index: 1usize, "Billing" }
                TabTrigger {
                    value: "invoices".to_string(),
                    index: 2usize,
                    disabled: true,
                    "Invoices"
                }
            }
            TabContent { index: 0usize, value: "account".to_string(), padding: "var(--dx-space-4)",
                "Update your name, email address and profile photo."
            }
            TabContent { index: 1usize, value: "billing".to_string(), padding: "var(--dx-space-4)",
                "Review your plan, payment method and next renewal date."
            }
            TabContent { index: 2usize, value: "invoices".to_string(), padding: "var(--dx-space-4)",
                "Invoices are available on paid plans."
            }
        }
    }
}
