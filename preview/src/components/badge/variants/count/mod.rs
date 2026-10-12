use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { class: "dx-badge-example",
            span { style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                "Inbox"
                Badge { "8" }
            }
            span { style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                "Notifications"
                Badge { variant: BadgeVariant::Secondary, "99+" }
            }
            span { style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                "Failed builds"
                Badge { variant: BadgeVariant::Destructive, "3" }
            }
        }
    }
}
