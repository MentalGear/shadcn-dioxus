use super::super::component::*;
use dioxus::prelude::*;

/// `TabsSize::Sm`, `Default` and `Lg` stacked: the list is 28, 32 and 36px, the same ladder as
/// `Button`'s `sm`/`default`/`lg`. Labels avoid the other variants' ("Tab N", "One/Two/Three",
/// "Overview", "Summary", ...) so tab locators by name stay unambiguous on the shared page.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            gap: "1.5rem",
            width: "100%",
            Tabs {
                default_value: "inbox".to_string(),
                horizontal: true,
                size: TabsSize::Sm,
                max_width: "24rem",
                TabList {
                    TabTrigger { value: "inbox".to_string(), "Inbox" }
                    TabTrigger { value: "sent".to_string(), "Sent" }
                    TabTrigger { value: "drafts".to_string(), "Drafts" }
                }
                TabContent { value: "inbox".to_string(), padding: "var(--dx-space-4)",
                    "Small: a 28px list with text-xs triggers."
                }
                TabContent { value: "sent".to_string(), padding: "var(--dx-space-4)",
                    "Messages you have already sent."
                }
                TabContent { value: "drafts".to_string(), padding: "var(--dx-space-4)",
                    "Messages you have started but not sent."
                }
            }
            Tabs {
                default_value: "photos".to_string(),
                horizontal: true,
                max_width: "24rem",
                TabList {
                    TabTrigger { value: "photos".to_string(), "Photos" }
                    TabTrigger { value: "videos".to_string(), "Videos" }
                    TabTrigger { value: "files".to_string(), "Files" }
                }
                TabContent { value: "photos".to_string(), padding: "var(--dx-space-4)",
                    "Default: the 32px list, unchanged."
                }
                TabContent { value: "videos".to_string(), padding: "var(--dx-space-4)",
                    "Clips and recordings from your library."
                }
                TabContent { value: "files".to_string(), padding: "var(--dx-space-4)",
                    "Documents, archives and everything else."
                }
            }
            Tabs {
                default_value: "daily".to_string(),
                horizontal: true,
                size: TabsSize::Lg,
                max_width: "24rem",
                TabList {
                    TabTrigger { value: "daily".to_string(), "Daily" }
                    TabTrigger { value: "weekly".to_string(), "Weekly" }
                    TabTrigger { value: "monthly".to_string(), "Monthly" }
                }
                TabContent { value: "daily".to_string(), padding: "var(--dx-space-4)",
                    "Large: a 36px list with roomier triggers."
                }
                TabContent { value: "weekly".to_string(), padding: "var(--dx-space-4)",
                    "A summary delivered once a week."
                }
                TabContent { value: "monthly".to_string(), padding: "var(--dx-space-4)",
                    "A full report on the first of the month."
                }
            }
        }
    }
}
