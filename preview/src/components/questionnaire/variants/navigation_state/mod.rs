use super::super::component::*;
use crate::components::button::ButtonVariant;
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
    vec![
        ItemDef::new("permission")
            .required()
            .choices(["files", "tests", "config"]),
        ItemDef::new("verification")
            .required()
            .choices(["tests", "types", "all"]),
    ]
}

/// Next and Submit, disabled while the active item is unanswered.
#[component]
fn GatedActions() -> Element {
    let questionnaire = use_questionnaire();
    let unanswered = questionnaire.active_status() == Some(ItemStatus::Unanswered);
    rsx! {
        QuestionnaireActions {
            QuestionnairePrevious {}
            QuestionnaireNext { variant: ButtonVariant::Secondary, disabled: unanswered }
            QuestionnaireSubmit { disabled: unanswered, "Save permissions" }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                on_submit: move |event: FormEvent| result.set(format!("Permissions saved. {}", answers(&event, &["permission", "verification"]))),
                QuestionnaireProgress {}
                QuestionnaireItem { name: "permission",
                    QuestionnaireTitle { "What may the agent modify?" }
                    QuestionnaireDescription { "Next is intentionally disabled until an answer is selected." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "files", "Project files" }
                        QuestionnaireChoice { value: "tests", "Project files and tests" }
                        QuestionnaireChoice { value: "config", "Files, tests, and configuration" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "verification",
                    QuestionnaireTitle { "What must pass before completion?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "tests", "Tests" }
                        QuestionnaireChoice { value: "types", "Tests and types" }
                        QuestionnaireChoice { value: "all", "Tests, types, and visual QA" }
                    }
                    QuestionnaireError {}
                }
                GatedActions {}
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
