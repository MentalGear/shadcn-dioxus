use super::super::component::*;
use dioxus::prelude::*;

/// The `size` prop: `Sm` (28px) beside `Default` (32px). Own ids and labels, no "Selected:" readout
/// (see the `disabled` variant's note on `native_select.spec.ts`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 16rem;",
            label { r#for: "native-select-size-sm", "Small size" }
            NativeSelect { id: "native-select-size-sm", size: NativeSelectSize::Sm,
                option { value: "free", "Free" }
                option { value: "pro", "Pro" }
            }
            label { r#for: "native-select-size-default", "Default size" }
            NativeSelect { id: "native-select-size-default", size: NativeSelectSize::Default,
                option { value: "free", "Free" }
                option { value: "pro", "Pro" }
            }
        }
    }
}
