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
        ItemDef::new("scope")
            .required()
            .choices(["small", "medium", "large"]),
        ItemDef::new("strategy")
            .required()
            .choices(["single", "logical", "squash"]),
        ItemDef::new("tests")
            .required()
            .choices(["targeted", "package", "workspace"]),
        ItemDef::new("delivery")
            .required()
            .choices(["patch", "commit", "branch"]),
    ]
}

/// A segmented bar inside the named progressbar. The segments are styled by `data-state` classes, never by a
/// `style` string that changes with the step (Dioxus patches a changed `style` string property by property and
/// drops the properties the new string does not repeat).
#[component]
fn Checkpoints() -> Element {
    let progress = use_questionnaire().progress();
    rsx! {
        QuestionnaireProgress { style: "width: 100%;",
            span { aria_hidden: "true", class: "dx-questionnaire-checkpoints",
                for index in 0..progress.total {
                    span {
                        key: "{index}",
                        class: "dx-questionnaire-checkpoint",
                        "data-slot": "questionnaire-checkpoint",
                        "data-state": if index < progress.current { "done" } else { "pending" },
                    }
                }
            }
            span { "Checkpoint {progress.current} of {progress.total}" }
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
                default_item: Some("scope".to_string()),
                on_submit: move |event: FormEvent| result.set(format!("Pull request plan ready. {}", answers(&event, &["scope", "strategy", "tests", "delivery"]))),
                Checkpoints {}
                QuestionnaireItem { name: "scope",
                    QuestionnaireTitle { "How large is the change?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "small", "Small patch" }
                        QuestionnaireChoice { value: "medium", "Feature-sized change" }
                        QuestionnaireChoice { value: "large", "Cross-package change" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "strategy",
                    QuestionnaireTitle { "How should commits be organized?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "single", "Single commit" }
                        QuestionnaireChoice { value: "logical", "Logical commits" }
                        QuestionnaireChoice { value: "squash", "Squash before review" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "tests",
                    QuestionnaireTitle { "Which tests should run?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "targeted", "Targeted tests" }
                        QuestionnaireChoice { value: "package", "Package suite" }
                        QuestionnaireChoice { value: "workspace", "Full workspace" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "delivery",
                    QuestionnaireTitle { "How should the work be delivered?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "patch", "Patch only" }
                        QuestionnaireChoice { value: "commit", "Committed locally" }
                        QuestionnaireChoice { value: "branch", "Push a review branch" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Finish plan" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
