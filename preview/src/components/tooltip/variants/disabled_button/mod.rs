use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// A tooltip on a disabled button. A disabled button takes no pointer or focus,
/// so the trigger is the wrapping `span` (focusable, `tabindex` from the
/// trigger), and the button inside it is disabled. The button's own
/// `pointer-events: none` lets the hover reach the span.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger {
                as: |el: AsChild| rsx! {
                    span { ..el.attributes,
                        Button { variant: ButtonVariant::Outline, disabled: true, "Export" }
                    }
                },
            }
            TooltipContent {
                "Upgrade to a paid plan to export your data."
            }
        }
    }
}
