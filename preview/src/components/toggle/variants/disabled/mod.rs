use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `toggle-disabled`: a default and an outline toggle, both disabled.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
            Toggle { aria_label: "Toggle disabled", disabled: true, "Disabled" }
            Toggle {
                aria_label: "Toggle disabled outline",
                variant: ToggleVariant::Outline,
                disabled: true,
                "Disabled"
            }
        }
    }
}
