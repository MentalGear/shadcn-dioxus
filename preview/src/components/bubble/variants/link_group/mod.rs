use super::super::component::*;
use crate::components::toast::ToastProvider;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

/// Three tinted bubbles that are real buttons, grouped in one `BubbleGroup`. Each press answers
/// with a toast, so the choice is visible without leaving the conversation.
#[component]
pub fn Demo() -> Element {
    rsx! {
        ToastProvider {
            Choices {}
        }
    }
}

#[component]
fn Choices() -> Element {
    let toast = use_toast();
    let options = move || ToastOptions::new();

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; gap: var(--dx-space-8);",
            Bubble { variant: BubbleVariant::Muted,
                BubbleContent { "How can I help you today?" }
            }
            BubbleGroup {
                Bubble { variant: BubbleVariant::Tinted, align: BubbleAlign::End,
                    BubbleContent {
                        r#as: move |attrs: Vec<Attribute>| rsx! {
                            button {
                                r#type: "button",
                                onclick: move |_| toast.info("You clicked forgot password".to_string(), options()),
                                ..attrs,
                                "I forgot my password"
                            }
                        },
                    }
                }
                Bubble { variant: BubbleVariant::Tinted, align: BubbleAlign::End,
                    BubbleContent {
                        r#as: move |attrs: Vec<Attribute>| rsx! {
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    toast.info("You clicked help with subscription".to_string(), options())
                                },
                                ..attrs,
                                "I need help with my subscription"
                            }
                        },
                    }
                }
                Bubble { variant: BubbleVariant::Tinted, align: BubbleAlign::End,
                    BubbleContent {
                        r#as: move |attrs: Vec<Attribute>| rsx! {
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    toast
                                        .info(
                                            "You clicked something else. Talk to a human.".to_string(),
                                            options(),
                                        )
                                },
                                ..attrs,
                                "Something else. Talk to a human."
                            }
                        },
                    }
                }
            }
        }
    }
}
