use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { style: "width: 100%; max-width: 24rem;",
            CardHeader {
                CardTitle { "Terms of service" }
                CardDescription { "Review and accept the terms below to continue." }
            }
            CardContent {
                div {
                    role: "region",
                    tabindex: "0",
                    aria_label: "Terms of service text",
                    style: "max-height: 9rem; overflow-y: auto; display: flex; flex-direction: column; gap: 0.75rem; color: var(--dx-muted-foreground);",
                    p {
                        "By creating an account you agree to use the service lawfully and to keep your credentials secure. You are responsible for all activity that happens under your account."
                    }
                    p {
                        "We may update these terms from time to time. When a change is material we will notify you by email at least thirty days before it takes effect."
                    }
                    p {
                        "You retain ownership of the content you upload. You grant us a limited licence to host and display it solely to operate the service for you."
                    }
                    p {
                        "Either party may end this agreement at any time. On termination we will delete your data within ninety days, unless the law requires us to keep it longer."
                    }
                }
            }
            CardFooter { style: "justify-content: flex-end; gap: 0.5rem;",
                Button { variant: ButtonVariant::Outline, "Decline" }
                Button { "Accept" }
            }
        }
    }
}
