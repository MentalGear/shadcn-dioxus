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
    vec![
        ItemDef::new("scope")
            .required()
            .choices(["bug", "feature", "refactor"]),
        ItemDef::new("checks")
            .required()
            .choices(["tests", "types", "both"]),
        ItemDef::new("output").required().choices(["patch", "pr"]),
    ]
}

fn label(item: &str) -> &'static str {
    match item {
        "scope" => "Scope",
        "checks" => "Verification",
        _ => "Output",
    }
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    let mut item = use_signal(|| "scope".to_string());
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            p { role: "status", style: "margin: 0; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "Current checkpoint: {label(&item())}" }
            div { role: "group", aria_label: "Jump to checkpoint", style: "display: flex; gap: var(--dx-space-2); flex-wrap: wrap;",
                for name in ["scope", "checks", "output"] {
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| item.set(name.to_string()), "{label(name)}" }
                }
            }
            Questionnaire {
                items: items(),
                item: Some(item()),
                on_item_change: move |next: String| item.set(next),
                on_submit: move |event: FormEvent| result.set(format!("Agent workflow configured. {}", answers(&event, &["scope", "checks", "output"]))),
                QuestionnaireItem { name: "scope",
                    QuestionnaireTitle { "What should the agent work on?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "bug", "Fix a bug" }
                        QuestionnaireChoice { value: "feature", "Build a feature" }
                        QuestionnaireChoice { value: "refactor", "Refactor code" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "checks",
                    QuestionnaireTitle { "Which checks must pass?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "tests", "Tests" }
                        QuestionnaireChoice { value: "types", "Type checks" }
                        QuestionnaireChoice { value: "both", "Tests and type checks" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "output",
                    QuestionnaireTitle { "What should it deliver?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "patch", "A patch" }
                        QuestionnaireChoice { value: "pr", "A pull request" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Configure workflow" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
