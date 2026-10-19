use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::html::FormValue;
use dioxus::prelude::*;

/// What the native form submitted, item by item.
fn answers(event: &FormEvent, names: &[&str]) -> String {
    names
        .iter()
        .map(|name| {
            let values: Vec<String> = event
                .get(name)
                .into_iter()
                .filter_map(|value| match value {
                    FormValue::Text(text) => Some(text),
                    FormValue::File(_) => None,
                })
                .collect();
            let shown = if values.is_empty() {
                "None".to_string()
            } else {
                values.join(", ")
            };
            format!("{name}: {shown}")
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn items() -> Vec<ItemDef> {
    vec![ItemDef::new("action")
        .required()
        .choices(["inspect", "tests", "patch"])]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    let mut mode = use_signal(|| Some(ShortcutMode::Letters));
    let label = |m: Option<ShortcutMode>| match m {
        Some(ShortcutMode::Letters) => "Letters",
        Some(ShortcutMode::Numbers) => "Numbers",
        None => "No shortcuts",
    };
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            div { role: "group", aria_label: "Shortcut style", style: "display: flex; gap: var(--dx-space-2); flex-wrap: wrap;",
                for m in [None, Some(ShortcutMode::Letters), Some(ShortcutMode::Numbers)] {
                    Button {
                        variant: if mode() == m { ButtonVariant::Secondary } else { ButtonVariant::Ghost },
                        size: ButtonSize::Sm,
                        "aria-pressed": if mode() == m { "true" } else { "false" },
                        onclick: move |_| mode.set(m),
                        {label(m)}
                    }
                }
            }
            Questionnaire {
                items: items(),
                shortcuts: mode(),
                on_submit: move |event: FormEvent| result.set(format!("Next action selected. {}", answers(&event, &["action"]))),
                QuestionnaireItem { name: "action",
                    QuestionnaireTitle { "What should the agent do next?" }
                    QuestionnaireDescription { "Use the displayed shortcut or navigate with the keyboard." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "inspect", "Inspect the implementation" }
                        QuestionnaireChoice { value: "tests", "Run the relevant tests" }
                        QuestionnaireChoice { value: "patch", "Prepare the patch" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnaireSubmit { "Confirm action" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
