use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bell, RefreshCw};

// Gradient fill is a plain background on the root, built from theme tokens
// (muted fading into the page background), as in shadcn's example.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Empty { style: "width: 100%; max-width: 28rem; background: linear-gradient(to bottom, color-mix(in oklab, var(--dx-muted) 50%, transparent) 30%, var(--dx-background));",
            EmptyHeader {
                EmptyMedia { variant: EmptyMediaVariant::Icon,
                    Bell {}
                }
                EmptyTitle { "No notifications" }
                EmptyDescription { "You're all caught up. New notifications will appear here." }
            }
            EmptyContent {
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                    RefreshCw {}
                    "Refresh"
                }
            }
        }
    }
}
