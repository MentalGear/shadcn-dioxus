use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{BookOpenCheck, GitBranch, Search};

/// An icon next to its text in each variant, and a marker turned into a column (icon above text)
/// with a plain `flex-direction` on the root.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-12);",
            Marker {
                MarkerIcon {
                    GitBranch {}
                }
                MarkerContent { "Switched to a new branch" }
            }
            Marker { variant: MarkerVariant::Separator,
                MarkerIcon {
                    Search {}
                }
                MarkerContent { "Explored 4 files" }
            }
            Marker { style: "flex-direction: column;",
                MarkerIcon {
                    BookOpenCheck {}
                }
                MarkerContent { "Syncing completed" }
            }
        }
    }
}
