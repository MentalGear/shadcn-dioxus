use super::super::component::*;
use crate::components::button::Button;
use crate::components::checkbox::Checkbox;
use crate::components::native_select::NativeSelect;
use dioxus_primitives::checkbox::CheckboxState;
use crate::components::form::DemoForm;
use crate::components::textarea::Textarea;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    let mut name = use_signal(String::new);
    let mut bio = use_signal(String::new);
    let mut fruit = use_signal(|| "apple".to_string());
    let mut agree = use_signal(|| CheckboxState::Unchecked);
    let mut renders = use_signal(|| 0u32);
    rsx! {
        DemoForm { id: "input-reset-form", show_status: false, style: "display: flex; flex-direction: column; gap: 0.75rem; width: 100%; max-width: 20rem;",
            Input {
                id: "input-reset-controlled",
                placeholder: "Controlled",
                value: name,
                oninput: move |e: FormEvent| name.set(e.value()),
            }
            Input {
                id: "input-reset-uncontrolled",
                placeholder: "Uncontrolled",
                value: "preset",
            }
            Textarea {
                id: "input-reset-textarea",
                placeholder: "Controlled textarea",
                value: bio,
                oninput: move |e: FormEvent| bio.set(e.value()),
            }
            NativeSelect {
                id: "input-reset-select",
                aria_label: "Fruit",
                name: "fruit",
                value: "{fruit}",
                onchange: move |e: FormEvent| fruit.set(e.value()),
                option { value: "apple", "Apple" }
                option { value: "banana", "Banana" }
            }
            Checkbox {
                id: "input-reset-checkbox",
                aria_label: "Agree",
                name: "agree",
                checked: Some(agree()),
                on_checked_change: move |state| agree.set(state),
            }
            div { style: "display: flex; gap: 0.5rem;",
                Button { r#type: "reset", id: "input-reset-button", "Reset" }
                Button {
                    r#type: "button",
                    id: "input-reset-rerender",
                    onclick: move |_| renders += 1,
                    "Re-render"
                }
            }
            p { id: "input-reset-state", "name:{name} bio:{bio} fruit:{fruit} agree:{agree():?} renders:{renders}" }
        }
    }
}
