use super::super::component::*;
use crate::components::switch::Switch;
use dioxus::prelude::*;

fn items() -> Vec<ItemDef> {
    vec![
        ItemDef::new("mood").required().choices(["yes", "no"]),
        ItemDef::new("areas").multiple().choices([
            "components",
            "primitives",
            "theme",
            "docs",
            "tests",
            "tooling",
        ]),
        ItemDef::new("notes").choices(["short"]).freeform(),
    ]
}

/// Three items of very different heights, and a switch that turns `animate_height` on and off while you
/// navigate: on, the form glides between the heights; off, it snaps.
#[component]
pub fn Demo() -> Element {
    let mut animate = use_signal(|| true);
    rsx! {
        div { style: "width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: var(--dx-space-4);",
            div { style: "display: flex; align-items: center; gap: var(--dx-space-2);",
                Switch {
                    id: "questionnaire-animate-height-switch",
                    aria_label: "Animate height",
                    checked: Some(animate()),
                    on_checked_change: move |value| animate.set(value),
                }
                label {
                    r#for: "questionnaire-animate-height-switch",
                    style: "font-size: var(--dx-text-sm); font-weight: 500;",
                    "Animate height"
                }
            }
            Questionnaire {
                items: items(),
                default_item: Some("mood".to_string()),
                animate_height: animate(),
                QuestionnaireProgress {}
                QuestionnaireItem { name: "mood",
                    QuestionnaireTitle { "Is this short?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "yes", "Yes" }
                        QuestionnaireChoice { value: "no", "No" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "areas",
                    QuestionnaireTitle { "Which areas does the change touch?" }
                    QuestionnaireDescription { "Pick any number. This item is much taller than the first." }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "components",
                            span { "Components" }
                            QuestionnaireChoiceDescription { "Styled parts in the registry." }
                        }
                        QuestionnaireChoice { value: "primitives",
                            span { "Primitives" }
                            QuestionnaireChoiceDescription { "Unstyled behaviour and ARIA." }
                        }
                        QuestionnaireChoice { value: "theme",
                            span { "Theme" }
                            QuestionnaireChoiceDescription { "Tokens shared by every component." }
                        }
                        QuestionnaireChoice { value: "docs", "Documentation" }
                        QuestionnaireChoice { value: "tests", "Tests" }
                        QuestionnaireChoice { value: "tooling", "Tooling" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireItem { name: "notes",
                    QuestionnaireTitle { "Anything else?" }
                    QuestionnaireChoices {
                        QuestionnaireChoice { value: "short", "Nothing, it is short" }
                        QuestionnaireInput { aria_label: "Another note", placeholder: "Add a note…" }
                    }
                    QuestionnaireError {}
                }
                QuestionnaireActions {
                    QuestionnairePrevious {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit {}
                }
            }
        }
    }
}
