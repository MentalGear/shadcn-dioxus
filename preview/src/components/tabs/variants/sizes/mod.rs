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
                    TabTrigger { value: "inbox".to_string(), index: 0usize, "Inbox" }
                    TabTrigger { value: "sent".to_string(), index: 1usize, "Sent" }
                    TabTrigger { value: "drafts".to_string(), index: 2usize, "Drafts" }
                }
                TabContent { index: 0usize, value: "inbox".to_string(), padding: "var(--dx-space-4)",
                    "Small: a 28px list with text-xs triggers."
                }
                TabContent { index: 1usize, value: "sent".to_string(), padding: "var(--dx-space-4)",
                    "Messages you have already sent."
                }
                TabContent { index: 2usize, value: "drafts".to_string(), padding: "var(--dx-space-4)",
                    "Messages you have started but not sent."
                }
            }
            Tabs {
                default_value: "photos".to_string(),
                horizontal: true,
                max_width: "24rem",
                TabList {
                    TabTrigger { value: "photos".to_string(), index: 0usize, "Photos" }
                    TabTrigger { value: "videos".to_string(), index: 1usize, "Videos" }
                    TabTrigger { value: "files".to_string(), index: 2usize, "Files" }
                }
                TabContent { index: 0usize, value: "photos".to_string(), padding: "var(--dx-space-4)",
                    "Default: the 32px list, unchanged."
                }
                TabContent { index: 1usize, value: "videos".to_string(), padding: "var(--dx-space-4)",
                    "Clips and recordings from your library."
                }
                TabContent { index: 2usize, value: "files".to_string(), padding: "var(--dx-space-4)",
                    "Documents, archives and everything else."
                }
            }
            Tabs {
                default_value: "daily".to_string(),
                horizontal: true,
                size: TabsSize::Lg,
                max_width: "24rem",
                TabList {
                    TabTrigger { value: "daily".to_string(), index: 0usize, "Daily" }
                    TabTrigger { value: "weekly".to_string(), index: 1usize, "Weekly" }
                    TabTrigger { value: "monthly".to_string(), index: 2usize, "Monthly" }
                }
                TabContent { index: 0usize, value: "daily".to_string(), padding: "var(--dx-space-4)",
                    "Large: a 36px list with roomier triggers."
                }
                TabContent { index: 1usize, value: "weekly".to_string(), padding: "var(--dx-space-4)",
                    "A summary delivered once a week."
                }
                TabContent { index: 2usize, value: "monthly".to_string(), padding: "var(--dx-space-4)",
                    "A full report on the first of the month."
                }
            }
        }
    }
}
