use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's `input-otp-pattern`: letters and digits. One typed `pattern` drives the keyboard
/// (`inputmode="text"`), the `pattern` attribute and the filter on typed and pasted text.
#[component]
pub fn Demo() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; align-items: flex-start;",
            InputOtp {
                id: "otp-alnum",
                max_length: 6,
                pattern: OtpPattern::Alphanumeric,
                aria_label: "Alphanumeric passcode",
                on_value_change: move |v: String| value.set(v),
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
            p { id: "input-otp-alnum-value", "Value: {value}" }
        }
    }
}
