use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

// `Card` has no typed `size` prop; the stylesheet's `.dx-card[data-size="sm"]`
// hook (12px spacing, `text-sm` title) is reached through the passthrough
// attribute, the same way shadcn's `size="sm"` renders `data-size="sm"`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { "data-size": "sm", style: "width: 100%; max-width: 24rem;",
            CardHeader {
                CardTitle { "Storage almost full" }
                CardDescription { "You have used 92% of your 15 GB plan." }
            }
            CardContent {
                p { "Free up space or upgrade to keep syncing new files across your devices." }
            }
            CardFooter {
                Button { variant: ButtonVariant::Outline, "Manage storage" }
            }
        }
    }
}
