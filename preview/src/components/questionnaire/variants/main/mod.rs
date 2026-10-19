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
        ItemDef::new("direction")
            .required()
            .choices(["tool-calls", "approvals", "handoffs"])
            .freeform(),
        ItemDef::new("signals")
            .multiple()
            .choices(["progress", "decisions", "risks", "next-step"]),
        ItemDef::new("timing")
            .required()
            .choices(["now", "next-cycle", "backlog"]),
    ]
}

#[component]
pub fn Demo() -> Element {
    let mut result = use_signal(String::new);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            Questionnaire {
                items: items(),
                default_item: Some("direction".to_string()),
                shortcuts: Some(ShortcutMode::Letters),
                on_submit: move |event: FormEvent| result.set(format!("Agent plan saved. {}", answers(&event, &["direction", "signals", "timing"]))),
                QuestionnaireProgress {}
                QuestionnaireItem { name: "direction",
                    QuestionnaireTitle { "What should the agent build next?" }
                    QuestionnaireDescription { "Choose a direction or describe another task." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "tool-calls",
                            span { style: "font-weight: 500;", "Tool call timeline" }
                            QuestionnaireChoiceDescription { "Show what the agent ran and what came back." }
                        }
                        QuestionnaireChoice { value: "approvals",
                            span { style: "font-weight: 500;", "Approval checkpoints" }
                            QuestionnaireChoiceDescription { "Ask before sensitive or destructive actions." }
                        }
                        QuestionnaireChoice { value: "handoffs",
                            span { style: "font-weight: 500;", "Sub-agent handoffs" }
                            QuestionnaireChoiceDescription { "Make delegated work and results easier to follow." }
                        }
                        QuestionnaireInput { aria_label: "Another agent feature", placeholder: "Describe another feature…" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "signals",
                    QuestionnaireTitle { "What should every progress update include?" }
                    QuestionnaireDescription { "Select all that apply, or skip this question." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "progress", "Progress" }
                        QuestionnaireChoice { value: "decisions", "Decisions" }
                        QuestionnaireChoice { value: "risks", "Risks" }
                        QuestionnaireChoice { value: "next-step", "Next step" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "timing",
                    QuestionnaireTitle { "When should work begin?" }
                    QuestionnaireDescription { "Choose when the agent should begin the work." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "now", "Start now" }
                        QuestionnaireChoice { value: "next-cycle", "Next development cycle" }
                        QuestionnaireChoice { value: "backlog", "Add it to the backlog" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireSkip {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Save plan" }
                }
            }
            p { role: "status", style: "margin: 0; min-height: 1lh; color: var(--dx-muted-foreground); font-size: var(--dx-text-sm);", "{result}" }
        }
    }
}
