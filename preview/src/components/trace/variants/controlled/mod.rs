use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::Brain;

/// Controlled: the step is open while it is "streaming" and the reader can still toggle it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| true);
    let mut running = use_signal(|| true);
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            TraceCollapsible {
                open: Some(open()),
                on_open_change: move |next| open.set(next),
                TraceCollapsibleTrigger {
                    TraceLine {
                        status: if running() { Some(TraceStatus::Running) } else { None },
                        TraceIcon {
                            Brain {}
                        }
                        TraceTitle {
                            if running() {
                                "Thinking"
                            } else {
                                "Thought"
                            }
                        }
                    }
                }
                TraceCollapsibleContent {
                    p { style: "margin: 0;", "Weighing two ways to anchor the new turn..." }
                }
            }
            div { style: "align-self: flex-start;",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        running.set(false);
                        open.set(false);
                    },
                    "Finish thinking"
                }
            }
        }
    }
}
