use super::super::component::*;
use crate::components::input::Input;
use crate::components::switch::Switch;
use dioxus::prelude::*;

/// `Field { disabled }`: shadcn's `data-disabled` field dims its labels. The controls carry their own
/// `disabled`; the third field is enabled, for comparison.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            FieldGroup {
                Field { disabled: true,
                    FieldLabel { html_for: "field-disabled-workspace", "Workspace" }
                    Input {
                        id: "field-disabled-workspace",
                        value: "acme-inc",
                        disabled: true,
                    }
                    FieldDescription { "Only an owner can rename the workspace." }
                }
                Field { orientation: FieldOrientation::Horizontal, disabled: true,
                    FieldContent {
                        FieldLabel { html_for: "field-disabled-sso", "Single sign-on" }
                        FieldDescription { "Available on the Enterprise plan." }
                    }
                    Switch {
                        id: "field-disabled-sso",
                        aria_label: "Single sign-on",
                        disabled: true,
                    }
                }
                Field {
                    FieldLabel { html_for: "field-disabled-name", "Display name" }
                    Input { id: "field-disabled-name", placeholder: "Acme" }
                }
            }
        }
    }
}
