use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronLeft, ChevronRight};

/// shadcn's "Nested" example: a group of groups -- the page numbers in one
/// `ButtonGroup`, the previous/next arrows in another, side by side with a gap.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { style: "gap: 0.5rem;", "aria-label": "Pagination",
            ButtonGroup { "aria-label": "Pages",
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "1" }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "2" }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "3" }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "4" }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "5" }
            }
            ButtonGroup { "aria-label": "Page navigation",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::IconSm,
                    "aria-label": "Previous page",
                    ChevronLeft {}
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::IconSm,
                    "aria-label": "Next page",
                    ChevronRight {}
                }
            }
        }
    }
}
