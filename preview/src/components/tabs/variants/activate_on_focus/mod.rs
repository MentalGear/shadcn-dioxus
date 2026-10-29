use super::super::component::*;
use dioxus::prelude::*;

/// Automatic activation: an arrow key, Home or End selects the tab it moves focus to.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "account".to_string(),
            activate_on_focus: true,
            horizontal: true,
            max_width: "24rem",
            TabList {
                TabTrigger { value: "account".to_string(), "Account" }
                TabTrigger { value: "billing".to_string(), "Billing" }
                TabTrigger { value: "team".to_string(), "Team" }
            }
            TabContent { value: "account".to_string(), padding: "var(--dx-space-4)",
                "Update your name, email address and profile photo."
            }
            TabContent { value: "billing".to_string(), padding: "var(--dx-space-4)",
                "Review your plan, payment method and next renewal date."
            }
            TabContent { value: "team".to_string(), padding: "var(--dx-space-4)",
                "Invite teammates and manage their roles."
            }
        }
    }
}
