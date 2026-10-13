use super::super::component::*;
use dioxus::prelude::*;

/// A disabled `NativeSelect`: the platform control dims and ignores input.
///
/// `native_select.spec.ts` reads the page's first `#component-preview-frame`
/// (the `main` variant) and `getByText("Selected: banana")`, so this variant
/// has its own `id`, its own label text and no "Selected:" readout.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 16rem;",
            label { r#for: "native-select-disabled-plan", "Billing plan" }
            NativeSelect { id: "native-select-disabled-plan", disabled: true,
                option { value: "free", "Free" }
                option { value: "pro", selected: true, "Pro" }
                option { value: "team", "Team" }
                option { value: "enterprise", "Enterprise" }
            }
        }
    }
}
