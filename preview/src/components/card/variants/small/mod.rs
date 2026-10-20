use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

// The typed `size: CardSize::Sm` prop renders `data-size="sm"` (12px spacing, `text-sm` title),
// as shadcn's `size="sm"` does.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { size: CardSize::Sm, style: "width: 100%; max-width: 24rem;",
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
