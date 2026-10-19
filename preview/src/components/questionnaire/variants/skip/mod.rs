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
    vec![
        ItemDef::new("task")
            .required()
            .choices(["feature", "fix", "refactor"]),
        ItemDef::new("constraints")
            .choices(["no-dependencies", "no-migrations", "preserve-api"])
            .freeform(),
        ItemDef::new("review")
            .required()
            .choices(["tests", "diff", "both"]),
    ]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    let mut constraint_status = use_signal(|| ItemStatus::Unanswered);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                default_item: Some("task".to_string()),
                on_answer_change: move |change: AnswerChange| {
                    if change.item == "constraints" {
                        constraint_status.set(change.status);
                    }
                },
                on_submit: move |event: FormEvent| {
                    let constraints = if constraint_status() == ItemStatus::Skipped { "constraints: Skipped".to_string() } else { answers(&event, &["constraints"]) };
                    result.set(format!("Agent brief submitted. {} · {constraints} · {}", answers(&event, &["task"]), answers(&event, &["review"])));
                },
                QuestionnaireProgress {}
                QuestionnaireItem { name: "task",
                    QuestionnaireTitle { "What kind of change is this?" }
                    QuestionnaireDescription { "Choose the category that best describes the work." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "feature", "New feature" }
                        QuestionnaireChoice { value: "fix", "Bug fix" }
                        QuestionnaireChoice { value: "refactor", "Refactor" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "constraints",
                    QuestionnaireTitle { "Are there any implementation constraints?" }
                    QuestionnaireDescription { "Answer if needed, or intentionally skip this question." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "no-dependencies", "Do not add dependencies" }
                        QuestionnaireChoice { value: "no-migrations", "Do not change the database" }
                        QuestionnaireChoice { value: "preserve-api", "Preserve the public API" }
                        QuestionnaireInput { aria_label: "Another implementation constraint", placeholder: "Describe another constraint…" }
                    }
                }
                QuestionnaireItem { name: "review",
                    QuestionnaireTitle { "How should the work be reviewed?" }
                    QuestionnaireDescription { "Choose the checks the agent should complete before handoff." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "tests", "Run the test suite" }
                        QuestionnaireChoice { value: "diff", "Review the final diff" }
                        QuestionnaireChoice { value: "both", "Tests and diff review" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireSkip {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Submit brief" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
