use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::form::DemoForm;
use dioxus::prelude::*;

/// shadcn's "Verify your login" card: a six-digit code inside a `DemoForm`, a Verify submit and a
/// link-style "Resend code" button (`type="button"`, so it never submits the form).
#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { style: "width: 100%; max-width: 24rem;",
            CardHeader {
                CardTitle { "Verify your login" }
                CardDescription { "Enter the verification code we sent to m@example.com." }
            }
            CardContent {
                DemoForm { id: "input-otp-verify-form",
                    InputOtp {
                        id: "input-otp-verify",
                        name: "code",
                        max_length: 6,
                        required: true,
                        aria_label: "Verification code",
                        InputOtpGroup {
                            InputOtpSlot { index: 0 }
                            InputOtpSlot { index: 1 }
                            InputOtpSlot { index: 2 }
                        }
                        InputOtpSeparator {}
                        InputOtpGroup {
                            InputOtpSlot { index: 3 }
                            InputOtpSlot { index: 4 }
                            InputOtpSlot { index: 5 }
                        }
                    }
                }
            }
            CardFooter { style: "flex-direction: column; align-items: stretch; gap: 0.5rem;",
                Button {
                    r#type: "submit",
                    form: "input-otp-verify-form",
                    style: "width: 100%;",
                    "Verify"
                }
                div { style: "display: flex; align-items: center; justify-content: center; gap: 0.25rem; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                    "Didn't receive the code?"
                    Button { r#type: "button", variant: ButtonVariant::Link, "Resend code" }
                }
            }
        }
    }
}
