use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

/// shadcn's "Separator" example: two secondary buttons divided by a
/// `ButtonGroupSeparator` (Copy | Paste).
#[component]
pub fn Demo() -> Element {
    rsx! {
        ButtonGroup { "aria-label": "Clipboard",
            Button { variant: ButtonVariant::Secondary, "Copy" }
            ButtonGroupSeparator {}
            Button { variant: ButtonVariant::Secondary, "Paste" }
        }
    }
}
