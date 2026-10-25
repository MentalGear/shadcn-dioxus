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
                TabTrigger { value: "account".to_string(), "Account" }
                TabTrigger { value: "billing".to_string(), "Billing" }
                TabTrigger {
                    value: "invoices".to_string(),
                    disabled: true,
                    "Invoices"
                }
            }
            TabContent { value: "account".to_string(), padding: "var(--dx-space-4)",
                "Update your name, email address and profile photo."
            }
            TabContent { value: "billing".to_string(), padding: "var(--dx-space-4)",
                "Review your plan, payment method and next renewal date."
            }
            TabContent { value: "invoices".to_string(), padding: "var(--dx-space-4)",
                "Invoices are available on paid plans."
            }
        }
    }
}
