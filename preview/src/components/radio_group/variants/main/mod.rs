use super::super::component::*;
use crate::components::label::Label;
use dioxus::prelude::*;

// shadcn's radio-group-demo: three items, each a `RadioItem` paired with a
// `Label` (`html_for` == the item's `id`), `defaultValue="comfortable"`.
#[component]
pub fn Demo() -> Element {
    rsx! {
        RadioGroup { default_value: "comfortable".to_string(),
            div { style: "display: flex; align-items: center; gap: 0.75rem;",
                RadioItem {
                    id: "radio-group-main-default",
                    value: "default".to_string(),
                    index: 0usize,
                }
                Label { html_for: "radio-group-main-default", "Default" }
            }
            div { style: "display: flex; align-items: center; gap: 0.75rem;",
                RadioItem {
                    id: "radio-group-main-comfortable",
                    value: "comfortable".to_string(),
                    index: 1usize,
                }
                Label { html_for: "radio-group-main-comfortable", "Comfortable" }
            }
            div { style: "display: flex; align-items: center; gap: 0.75rem;",
                RadioItem {
                    id: "radio-group-main-compact",
                    value: "compact".to_string(),
                    index: 2usize,
                }
                Label { html_for: "radio-group-main-compact", "Compact" }
            }
        }
    }
}
