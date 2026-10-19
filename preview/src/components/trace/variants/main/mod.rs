use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{FileText, Search, Terminal};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 28rem; min-width: 0;",
            Trace {
                TraceLine {
                    TraceIcon {
                        Search {}
                    }
                    TraceTitle { "Searched the web" }
                    TraceDescription { "3 results" }
                }
                TraceLine {
                    TraceIcon {
                        FileText {}
                    }
                    TraceTitle { "Read 4 files" }
                    TraceDescription { "1.2s" }
                }
                TraceCollapsible {
                    TraceCollapsibleTrigger {
                        TraceLine {
                            TraceIcon {
                                Terminal {}
                            }
                            TraceTitle { "Ran cargo check" }
                            TraceDescription { "8.4s" }
                        }
                    }
                    TraceCollapsibleContent {
                        pre { style: "margin: 0; font-family: var(--dx-font-mono); font-size: var(--dx-text-xs); white-space: pre-wrap;",
                            "Checking preview v0.1.0\nFinished dev profile in 8.37s"
                        }
                    }
                }
            }
        }
    }
}
