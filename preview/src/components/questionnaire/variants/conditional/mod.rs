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

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    let mut runtime = use_signal(|| "local".to_string());
    let items = vec![
        ItemDef::new("runtime")
            .required()
            .choices(["local", "cloud"])
            .default_choice("local"),
        ItemDef::new("environment")
            .required()
            .choices(["preview", "staging", "isolated"])
            .disabled(runtime() != "cloud"),
        ItemDef::new("approval")
            .required()
            .choices(["writes", "commands", "sensitive"]),
    ];
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items,
                default_item: Some("runtime".to_string()),
                on_answer_change: move |change: AnswerChange| {
                    if change.item == "runtime" {
                        runtime.set(change.values.first().cloned().unwrap_or_default());
                    }
                },
                on_submit: move |event: FormEvent| result.set(format!("Execution plan saved. {}", answers(&event, &["runtime", "environment", "approval"]))),
                QuestionnaireProgress {}
                QuestionnaireItem { name: "runtime",
                    QuestionnaireTitle { "Where should the agent run?" }
                    QuestionnaireDescription { "Cloud runs add an environment question to this flow." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "local", "Local workspace" }
                        QuestionnaireChoice { value: "cloud", "Cloud workspace" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "environment",
                    QuestionnaireTitle { "Which cloud environment should it use?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "preview", "Preview" }
                        QuestionnaireChoice { value: "staging", "Staging" }
                        QuestionnaireChoice { value: "isolated", "Isolated sandbox" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "approval",
                    QuestionnaireTitle { "When should the agent request approval?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "writes", "Before writing files" }
                        QuestionnaireChoice { value: "commands", "Before running commands" }
                        QuestionnaireChoice { value: "sensitive", "Only for sensitive actions" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Save execution plan" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
