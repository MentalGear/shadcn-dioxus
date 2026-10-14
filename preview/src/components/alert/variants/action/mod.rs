use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

/// shadcn's `alert-action`: a title, a description and an `xs` default `Button` in an `AlertAction`, pinned to
/// the top-end corner while the alert reserves room for it.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 28rem;",
            Alert {
                AlertTitle { "Dark mode is now available" }
                AlertDescription { "Enable it under your profile settings to get started." }
                AlertAction {
                    Button { size: ButtonSize::Xs, variant: ButtonVariant::Primary, "Enable" }
                }
            }
        }
    }
}
