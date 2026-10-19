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

/// The schema: both answers are required, and a public answer needs the complete detail.
fn validate(detail: Option<&str>, audience: Option<&str>) -> Vec<(&'static str, &'static str)> {
    let mut errors = Vec::new();
    if detail.is_none() {
        errors.push(("detail", "Choose how much detail to include."));
    }
    if audience.is_none() {
        errors.push(("audience", "Choose who will read the answer."));
    }
    if audience == Some("public") && detail == Some("summary") {
        errors.push((
            "detail",
            "Public answers need enough context. Choose a complete answer.",
        ));
    }
    errors
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    let mut item = use_signal(|| "detail".to_string());
    let mut errors = use_signal(Vec::<(&'static str, &'static str)>::new);
    let error_of = move |name: &str| {
        errors
            .read()
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, m)| *m)
    };
    let mut clear = move |name: &'static str| errors.write().retain(|(n, _)| *n != name);
    let items = vec![
        ItemDef::new("detail")
            .required()
            .choices(["summary", "complete"])
            .invalid(error_of("detail").is_some()),
        ItemDef::new("audience")
            .required()
            .choices(["team", "public"])
            .invalid(error_of("audience").is_some()),
    ];
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items,
                item: Some(item()),
                on_item_change: move |next: String| item.set(next),
                on_answer_change: move |change: AnswerChange| {
                    if change.item == "detail" {
                        clear("detail");
                    } else if change.item == "audience" {
                        clear("audience");
                    }
                },
                on_submit: move |event: FormEvent| {
                    let first = |name: &str| match event.get_first(name) {
                        Some(FormValue::Text(text)) => Some(text),
                        _ => None,
                    };
                    let (detail, audience) = (first("detail"), first("audience"));
                    let found = validate(detail.as_deref(), audience.as_deref());
                    if let Some((name, _)) = found.first() {
                        item.set(name.to_string());
                        result.set(String::new());
                    } else {
                        result.set(format!("Agent response configured. {}", answers(&event, &["detail", "audience"])));
                    }
                    errors.set(found);
                },
                QuestionnaireProgress {}
                QuestionnaireItem { name: "detail",
                    QuestionnaireTitle { "How much detail should the answer include?" }
                    QuestionnaireDescription { "Choose the response depth." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "summary", "Concise summary" }
                        QuestionnaireChoice { value: "complete", "Complete answer" }
                    }
                    QuestionnaireError { {error_of("detail").unwrap_or("Choose an answer to continue.")} }
                }
                QuestionnaireItem { name: "audience",
                    QuestionnaireTitle { "Who will read the answer?" }
                    QuestionnaireDescription { "Public answers require complete context." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "team", "My team" }
                        QuestionnaireChoice { value: "public", "Public audience" }
                    }
                    QuestionnaireError { {error_of("audience").unwrap_or("Choose an answer to continue.")} }
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Validate answers" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
