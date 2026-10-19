use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Brain;

/// A reasoning step: the trigger reads "Thought", and the detail is the model's text.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 28rem; min-width: 0;",
            TraceCollapsible { default_open: true,
                TraceCollapsibleTrigger {
                    TraceLine {
                        TraceIcon {
                            Brain {}
                        }
                        TraceTitle { "Thought" }
                        TraceDescription { "4s" }
                    }
                }
                TraceCollapsibleContent {
                    p { style: "margin: 0;",
                        "The user wants the settings dialog to open from the gear icon. I should install Dialog, wrap the button in a trigger and put the form in the content."
                    }
                }
            }
        }
    }
}
