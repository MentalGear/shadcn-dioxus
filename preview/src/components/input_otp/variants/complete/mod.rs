use super::super::component::*;
use dioxus::prelude::*;

/// Letters only (`OtpPattern::Letters`, digits are filtered out), and `on_complete`, which runs
/// once when the last slot fills.
#[component]
pub fn Demo() -> Element {
    let mut completed = use_signal(|| None::<String>);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; align-items: flex-start;",
            InputOtp {
                id: "otp-complete",
                max_length: 4,
                pattern: OtpPattern::Letters,
                aria_label: "Letters-only code",
                on_complete: move |code: String| completed.set(Some(code)),
                InputOtpGroup {
                    InputOtpSlot { index: 0 }
                    InputOtpSlot { index: 1 }
                    InputOtpSlot { index: 2 }
                    InputOtpSlot { index: 3 }
                }
            }
            p { id: "input-otp-complete-status",
                if let Some(code) = completed() {
                    "Completed: {code}"
                } else {
                    "Type four letters."
                }
            }
        }
    }
}
