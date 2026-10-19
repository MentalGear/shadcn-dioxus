use super::super::component::*;
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
    vec![ItemDef::new("context")
        .required()
        .multiple()
        .choices(["source", "tests", "docs", "history"])]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                shortcuts: Some(ShortcutMode::Letters),
                on_submit: move |event: FormEvent| result.set(format!("Context selected. {}", answers(&event, &["context"]))),
                QuestionnaireItem { name: "context",
                    QuestionnaireTitle { "What context should the agent inspect?" }
                    QuestionnaireDescription { "Select every source that may affect the implementation." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "source", "Relevant source files" }
                        QuestionnaireChoice { value: "tests", "Existing tests" }
                        QuestionnaireChoice { value: "docs", "Architecture documentation" }
                        QuestionnaireChoice { value: "history", "Recent commit history" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnaireSubmit { "Share context" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
