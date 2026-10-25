use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `tabs-vertical` (`orientation="vertical"`): the list is a column beside the panel,
/// `ArrowUp`/`ArrowDown` move focus (`ArrowLeft`/`ArrowRight` do not), and the list reports
/// `aria-orientation="vertical"`. shadcn's example has triggers only; this one adds the panels.
/// See `vertical_line` for the `line` variant in this orientation.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tabs {
            default_value: "account".to_string(),
            horizontal: false,
            max_width: "28rem",
            TabList {
                TabTrigger { value: "account".to_string(), "Account" }
                TabTrigger { value: "password".to_string(), "Password" }
                TabTrigger { value: "notifications".to_string(), "Notifications" }
            }
            TabContent { value: "account".to_string(), padding: "var(--dx-space-4)",
                "Update your name, email and profile photo."
            }
            TabContent { value: "password".to_string(), padding: "var(--dx-space-4)",
                "Change your password, or sign out of every other device."
            }
            TabContent { value: "notifications".to_string(), padding: "var(--dx-space-4)",
                "Choose which events email you and which stay in the app."
            }
        }
    }
}
