use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
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
        ItemDef::new("change")
            .required()
            .choices(["incremental", "cutover"])
            .default_choice("incremental"),
        ItemDef::new("verification")
            .required()
            .multiple()
            .choices(["tests", "typecheck", "manual"])
            .default_choice("tests")
            .default_choice("typecheck"),
        ItemDef::new("notes")
            .freeform()
            .default_input("Keep the existing public API stable."),
    ]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                default_item: Some("verification".to_string()),
                on_reset: move |_| result.set("Saved answers restored.".to_string()),
                on_submit: move |event: FormEvent| result.set(format!("Draft updated. {}", answers(&event, &["change", "verification", "notes"]))),
                QuestionnaireProgress {}
                QuestionnaireItem { name: "change",
                    QuestionnaireTitle { "What kind of migration is this?" }
                    QuestionnaireDescription { "This answer was saved during the previous session." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "incremental", "Incremental migration" }
                        QuestionnaireChoice { value: "cutover", "Single cutover" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "verification",
                    QuestionnaireTitle { "How should the migration be verified?" }
                    QuestionnaireDescription { "These checks were selected during the previous session." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "tests", "Run migration tests" }
                        QuestionnaireChoice { value: "typecheck", "Run the typecheck" }
                        QuestionnaireChoice { value: "manual", "Perform a manual smoke test" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "notes",
                    QuestionnaireTitle { "Anything else the agent should remember?" }
                    QuestionnaireDescription { "This note was saved with the draft." }
                    QuestionnaireInput { aria_label: "Saved migration note" }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    Button { r#type: "reset", variant: ButtonVariant::Outline, "Reset changes" }
                    QuestionnairePrevious {}
                    QuestionnaireSkip {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Update draft" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
