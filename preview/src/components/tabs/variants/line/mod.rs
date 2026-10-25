use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `line` tabs: a transparent list with a 2px underline under the active trigger
/// (`TabsVariant::Line`). Labels avoid the `main` ("Tab N") and `until_found` ("Overview",
/// "Specs", "Reviews") variants' so tab locators by name stay unambiguous on the shared page.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "summary".to_string(),
            horizontal: true,
            variant: TabsVariant::Line,
            max_width: "24rem",
            TabList {
                TabTrigger { value: "summary".to_string(), "Summary" }
                TabTrigger { value: "analytics".to_string(), "Analytics" }
                TabTrigger { value: "reports".to_string(), "Reports" }
                TabTrigger { value: "settings".to_string(), "Settings" }
            }
            TabContent { value: "summary".to_string(),
                "A snapshot of your workspace: revenue, active users and open tasks at a glance."
            }
            TabContent { value: "analytics".to_string(),
                "Track visits, conversion and retention over the last thirty days."
            }
            TabContent { value: "reports".to_string(),
                "Download weekly and monthly reports, or schedule them to land in your inbox."
            }
            TabContent { value: "settings".to_string(),
                "Manage notifications, connected apps and workspace preferences."
            }
        }
    }
}
