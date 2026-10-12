use super::super::component::*;
use dioxus::prelude::*;

/// Alerts without a leading icon: with no `svg` child the title and description take the full
/// width (`.dx-alert:not(:has(> svg))`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 24rem;",
            Alert {
                AlertTitle { "Your changes have been saved" }
                AlertDescription { "Everyone on the team can now see the updated version." }
            }
            Alert { variant: AlertVariant::Destructive,
                AlertTitle { "Your session has expired" }
                AlertDescription { "Sign in again to keep editing this document." }
            }
        }
    }
}
