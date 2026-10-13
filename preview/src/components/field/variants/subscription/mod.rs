use super::super::component::*;
use crate::components::button::Button;
use crate::components::form::DemoForm;
use crate::components::native_select::NativeSelect;
use crate::components::radio_group::{RadioGroup, RadioItem};
use crate::components::switch::Switch;
use crate::components::textarea::Textarea;
use dioxus::prelude::*;

/// shadcn's composite form: a fieldset of choice-card plans, a switch row, a native select and a
/// textarea. The cards are `FieldLabel`s wrapping a `Field`, so each radio and the switch carry
/// their own `aria_label` (a `Field` is `role="group"`, which hides the label text from axe).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 24rem;",
            DemoForm { id: "field-subscription-form",
                FieldSet {
                    FieldLegend { "Subscription plan" }
                    FieldDescription { "Pick the plan that fits your team. You can change it later." }
                    RadioGroup {
                        default_value: "pro".to_string(),
                        name: "field-subscription-plan",
                        aria_label: "Subscription plan",
                        FieldLabel { html_for: "field-subscription-plan-starter",
                            Field { orientation: FieldOrientation::Horizontal,
                                FieldContent {
                                    "Starter"
                                    FieldDescription { "Free for personal projects." }
                                }
                                RadioItem {
                                    id: "field-subscription-plan-starter",
                                    aria_label: "Starter plan",
                                    value: "starter".to_string(),
                                    index: 0usize,
                                }
                            }
                        }
                        FieldLabel { html_for: "field-subscription-plan-pro",
                            Field { orientation: FieldOrientation::Horizontal,
                                FieldContent {
                                    "Pro"
                                    FieldDescription { "Unlimited projects for small teams." }
                                }
                                RadioItem {
                                    id: "field-subscription-plan-pro",
                                    aria_label: "Pro plan",
                                    value: "pro".to_string(),
                                    index: 1usize,
                                }
                            }
                        }
                        FieldLabel { html_for: "field-subscription-plan-scale",
                            Field { orientation: FieldOrientation::Horizontal,
                                FieldContent {
                                    "Scale"
                                    FieldDescription { "Advanced controls for large organizations." }
                                }
                                RadioItem {
                                    id: "field-subscription-plan-scale",
                                    aria_label: "Scale plan",
                                    value: "scale".to_string(),
                                    index: 2usize,
                                }
                            }
                        }
                    }
                    FieldSeparator {}
                    FieldGroup {
                        Field { orientation: FieldOrientation::Horizontal,
                            FieldContent {
                                FieldLabel { html_for: "field-subscription-yearly", "Bill yearly" }
                                FieldDescription { "Save two months by paying once a year." }
                            }
                            Switch { id: "field-subscription-yearly", aria_label: "Bill yearly" }
                        }
                        Field {
                            FieldLabel { html_for: "field-subscription-seats", "Team size" }
                            NativeSelect { id: "field-subscription-seats", name: "seats",
                                option { value: "1-5", "1-5 people" }
                                option { value: "6-20", "6-20 people" }
                                option { value: "21-100", "21-100 people" }
                                option { value: "100+", "More than 100 people" }
                            }
                        }
                        Field {
                            FieldLabel { html_for: "field-subscription-notes", "Anything we should know?" }
                            Textarea {
                                id: "field-subscription-notes",
                                name: "notes",
                                placeholder: "Billing contacts, invoice details, or questions for our team",
                            }
                        }
                        Field {
                            Button { r#type: "submit", "Start subscription" }
                        }
                    }
                }
            }
        }
    }
}
