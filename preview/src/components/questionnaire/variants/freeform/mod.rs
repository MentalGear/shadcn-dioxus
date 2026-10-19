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
    vec![ItemDef::new("approach")
        .required()
        .choices(["incremental", "module", "rewrite"])
        .freeform()]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                shortcuts: Some(ShortcutMode::Letters),
                on_submit: move |event: FormEvent| result.set(format!("Approach selected. {}", answers(&event, &["approach"]))),
                QuestionnaireItem { name: "approach",
                    QuestionnaireTitle { "How should the agent approach this refactor?" }
                    QuestionnaireDescription { "Choose a strategy or write a more specific instruction." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "incremental", "Make the smallest safe change" }
                        QuestionnaireChoice { value: "module", "Refactor one module at a time" }
                        QuestionnaireChoice { value: "rewrite", "Replace the implementation completely" }
                        QuestionnaireInput { aria_label: "Another refactoring approach", placeholder: "Describe another approach…" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnaireSubmit { "Use this approach" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
