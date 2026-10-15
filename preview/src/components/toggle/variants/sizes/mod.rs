use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `toggle-sizes`: `sm`, `default` and `lg` outline toggles (28, 32 and 36px).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;",
            Toggle {
                variant: ToggleVariant::Outline,
                aria_label: "Toggle small",
                size: ToggleSize::Sm,
                "Small"
            }
            Toggle {
                variant: ToggleVariant::Outline,
                aria_label: "Toggle default",
                size: ToggleSize::Default,
                "Default"
            }
            Toggle {
                variant: ToggleVariant::Outline,
                aria_label: "Toggle large",
                size: ToggleSize::Lg,
                "Large"
            }
        }
    }
}
