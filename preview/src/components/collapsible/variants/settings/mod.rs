use super::super::component::*;
use crate::components::field::{
    Field, FieldContent, FieldDescription, FieldGroup, FieldLabel, FieldOrientation,
};
use crate::components::input::Input;
use crate::components::switch::Switch;
use dioxus::prelude::*;

/// A settings panel: the everyday field is always visible, the rest hides behind "Advanced settings".
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            Collapsible {
                FieldGroup {
                    Field {
                        FieldLabel { html_for: "collapsible-settings-name", "Project name" }
                        Input { id: "collapsible-settings-name", value: "shadcn-dioxus" }
                    }
                }
                CollapsibleTrigger {
                    b { "Advanced settings" }
                }
                CollapsibleContent {
                    FieldGroup {
                        Field {
                            FieldLabel { html_for: "collapsible-settings-domain", "Custom domain" }
                            Input {
                                id: "collapsible-settings-domain",
                                placeholder: "docs.example.com",
                            }
                            FieldDescription { "Point a CNAME record at your project first." }
                        }
                        Field { orientation: FieldOrientation::Horizontal,
                            Switch {
                                id: "collapsible-settings-previews",
                                default_checked: true,
                            }
                            FieldContent {
                                FieldLabel { html_for: "collapsible-settings-previews", "Preview deployments" }
                                FieldDescription { "Build every pull request on its own URL." }
                            }
                        }
                        Field { orientation: FieldOrientation::Horizontal,
                            Switch { id: "collapsible-settings-analytics" }
                            FieldContent {
                                FieldLabel { html_for: "collapsible-settings-analytics", "Usage analytics" }
                                FieldDescription { "Share anonymous traffic data with your team." }
                            }
                        }
                    }
                }
            }
        }
    }
}
