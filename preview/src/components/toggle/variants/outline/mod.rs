use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bold, Italic};

/// shadcn's `toggle-outline`: two outline toggles with an icon and a label.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
            Toggle { variant: ToggleVariant::Outline, aria_label: "Toggle italic outline",
                Italic {}
                "Italic"
            }
            Toggle { variant: ToggleVariant::Outline, aria_label: "Toggle bold outline",
                Bold {}
                "Bold"
            }
        }
    }
}
