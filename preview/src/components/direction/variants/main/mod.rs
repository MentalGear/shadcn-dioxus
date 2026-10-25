use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use dioxus::prelude::*;

/// One signal drives both the `dir` attribute (layout, logical CSS) and the `DirectionProvider`
/// (keyboard and positioning logic), and the tabs follow it live.
#[component]
pub fn Demo() -> Element {
    let mut direction = use_signal(|| Direction::Ltr);
    let rtl = direction() == Direction::Rtl;
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 1rem; width: 100%;",
            Button {
                variant: ButtonVariant::Outline,
                r#type: "button",
                "aria-pressed": "{rtl}",
                onclick: move |_| {
                    direction
                        .set(if direction() == Direction::Rtl { Direction::Ltr } else { Direction::Rtl });
                },
                if rtl {
                    "Switch to LTR"
                } else {
                    "Switch to RTL"
                }
            }
            div { dir: direction().as_str(), style: "width: 100%;",
                DirectionProvider { direction: direction(),
                    Tabs { default_value: "a".to_string(), horizontal: true,
                        TabList {
                            TabTrigger { value: "a".to_string(), "First" }
                            TabTrigger { value: "b".to_string(), "Second" }
                            TabTrigger { value: "c".to_string(), "Third" }
                        }
                        TabContent { value: "a".to_string(),
                            "Focus a tab and press the arrow keys: ArrowRight moves to the next tab in LTR and to the previous one in RTL."
                        }
                        TabContent { value: "b".to_string(), "Second panel." }
                        TabContent { value: "c".to_string(), "Third panel." }
                    }
                }
            }
        }
    }
}
