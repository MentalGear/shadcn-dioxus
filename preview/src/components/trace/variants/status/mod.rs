use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{CircleAlert, Clock, Loader, Terminal};

/// Running steps shimmer, failed steps read in the destructive colour (and say so in words),
/// pending steps are dimmed.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 28rem; min-width: 0;",
            Trace { role: "status",
                TraceLine { status: TraceStatus::Running,
                    TraceIcon {
                        Loader {}
                    }
                    TraceTitle { "Reading src/main.rs" }
                }
                TraceLine { status: TraceStatus::Error,
                    TraceIcon {
                        CircleAlert {}
                    }
                    TraceTitle { "Failed to read config.toml" }
                    TraceDescription { "not found" }
                }
                TraceLine { status: TraceStatus::Pending,
                    TraceIcon {
                        Clock {}
                    }
                    TraceTitle { "Run the tests" }
                }
                TraceLine {
                    TraceIcon {
                        Terminal {}
                    }
                    TraceTitle { "Ran cargo fmt" }
                }
            }
        }
    }
}
