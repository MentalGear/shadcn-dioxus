use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_primitives::{ContentAlign, ContentSide};

/// A tooltip above and below its trigger at each alignment: whatever the alignment, the arrow
/// points at the trigger's centre.
#[component]
pub fn Demo() -> Element {
    let cases = [
        ("Top start", ContentSide::Top, ContentAlign::Start),
        ("Top center", ContentSide::Top, ContentAlign::Center),
        ("Top end", ContentSide::Top, ContentAlign::End),
        ("Bottom start", ContentSide::Bottom, ContentAlign::Start),
        ("Bottom center", ContentSide::Bottom, ContentAlign::Center),
        ("Bottom end", ContentSide::Bottom, ContentAlign::End),
    ];
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 1rem;",
            for (label , side , align) in cases {
                Tooltip {
                    TooltipTrigger {
                        as: move |el: AsChild| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes: el.attributes, "{label}" }
                        },
                    }
                    TooltipContent { side, align, style: "width: 14rem;",
                        "The arrow points at the {label} trigger"
                    }
                }
            }
        }
    }
}
