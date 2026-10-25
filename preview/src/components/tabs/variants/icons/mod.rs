use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bell, LayoutDashboard, User};

/// Triggers with a leading icon and a label (the trigger's `svg` gets 1rem sizing and a gap).
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "dashboard".to_string(),
            horizontal: true,
            max_width: "24rem",
            TabList {
                TabTrigger { value: "dashboard".to_string(), 
                    LayoutDashboard {}
                    "Dashboard"
                }
                TabTrigger { value: "alerts".to_string(), 
                    Bell {}
                    "Alerts"
                }
                TabTrigger { value: "profile".to_string(), 
                    User {}
                    "Profile"
                }
            }
            TabContent { value: "dashboard".to_string(), padding: "var(--dx-space-4)",
                "Pin the charts and shortcuts you use every day."
            }
            TabContent { value: "alerts".to_string(), padding: "var(--dx-space-4)",
                "You are all caught up. New alerts will show up here."
            }
            TabContent { value: "profile".to_string(), padding: "var(--dx-space-4)",
                "Choose how your name and avatar appear to teammates."
            }
        }
    }
}
