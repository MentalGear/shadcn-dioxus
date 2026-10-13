use super::super::component::*;
use dioxus::prelude::*;

/// A `NativeSelect` with `<optgroup>`s, one of them disabled. Plain native
/// children, so nothing but the component's own `option`/`optgroup` rules is
/// involved.
///
/// Own `id` and label text for the same page-wide locators the `disabled`
/// variant's doc names (`native_select.spec.ts`).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 16rem;",
            label { r#for: "native-select-groups-timezone", "Time zone" }
            NativeSelect { id: "native-select-groups-timezone",
                optgroup { label: "North America",
                    option { value: "est", "Eastern Standard Time (EST)" }
                    option { value: "cst", "Central Standard Time (CST)" }
                    option { value: "mst", "Mountain Standard Time (MST)" }
                    option { value: "pst", "Pacific Standard Time (PST)" }
                }
                optgroup { label: "Europe",
                    option { value: "gmt", "Greenwich Mean Time (GMT)" }
                    option { value: "cet", "Central European Time (CET)" }
                    option { value: "eet", "Eastern European Time (EET)" }
                }
                optgroup { label: "Asia",
                    option { value: "ist", "India Standard Time (IST)" }
                    option { value: "jst", "Japan Standard Time (JST)" }
                }
                optgroup { label: "Antarctica (no residents)", disabled: true,
                    option { value: "nzst", "McMurdo Station (NZST)" }
                }
            }
        }
    }
}
