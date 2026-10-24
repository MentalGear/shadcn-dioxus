use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronLeft, ChevronRight, Ellipsis};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 1.5rem;",
            ButtonGroup {
                // Icon-only buttons need an explicit accessible name -- the icon
                // is decorative markup, so without one axe's `button-name` rule
                // fires (critical) and a screen reader announces an unnamed
                // button. Every icon-only button in this demo carries one.
                Button {
                    variant: ButtonVariant::Outline,
                    "aria-label": "Previous",
                    ChevronLeft {}
                }
                Button { variant: ButtonVariant::Outline, "Today" }
                Button {
                    variant: ButtonVariant::Outline,
                    "aria-label": "Next",
                    ChevronRight {}
                }
            }
            ButtonGroup {
                Button { variant: ButtonVariant::Outline, "Archive" }
                Button { variant: ButtonVariant::Outline, "Report" }
                ButtonGroupSeparator {}
                Button { variant: ButtonVariant::Outline, "aria-label": "More", Ellipsis {} }
            }
        }
    }
}
