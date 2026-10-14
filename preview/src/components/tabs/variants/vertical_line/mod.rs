use super::super::component::*;
use dioxus::prelude::*;

/// The `line` variant in the vertical orientation: no fill, and the active trigger carries a
/// 2px bar on the side that faces the panel (shadcn's `after:inset-y-0 after:-right-1 after:w-0.5`,
/// mirrored in RTL).
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "teams".to_string(),
            horizontal: false,
            variant: TabsVariant::Line,
            max_width: "28rem",
            TabList {
                TabTrigger { value: "teams".to_string(), index: 0usize, "Teams" }
                TabTrigger { value: "members".to_string(), index: 1usize, "Members" }
                TabTrigger { value: "roles".to_string(), index: 2usize, "Roles" }
            }
            TabContent { index: 0usize, value: "teams".to_string(), padding: "var(--dx-space-4)",
                "Group people by project and give each team its own channel."
            }
            TabContent { index: 1usize, value: "members".to_string(), padding: "var(--dx-space-4)",
                "Invite colleagues, or remove people who have left."
            }
            TabContent { index: 2usize, value: "roles".to_string(), padding: "var(--dx-space-4)",
                "Decide who can edit, who can comment and who can only read."
            }
        }
    }
}
