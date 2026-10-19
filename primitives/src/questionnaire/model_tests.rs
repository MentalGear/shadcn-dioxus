//! Unit tests of the questionnaire state machine. Each test names the shadcn test it ports
//! (`packages/react/src/questionnaire/questionnaire.test.tsx` and `questionnaire.browser.test.tsx`); the DOM-only
//! halves (focus actually moving, native radio arrows, FormData) are in `playwright/questionnaire.spec.ts`.

use super::*;

fn always(_: &str, _: &AnswerKey) -> bool {
    true
}

fn all(_: &str) -> bool {
    true
}

fn run(
    state: &mut QuestionnaireState,
    cfg: &QuestionnaireConfig,
    command: Command,
) -> Vec<QuestionnaireEffect> {
    state.run(cfg, &command, &always, &all)
}

fn choose(
    state: &mut QuestionnaireState,
    cfg: &QuestionnaireConfig,
    item: &str,
    value: &str,
) -> Vec<QuestionnaireEffect> {
    run(
        state,
        cfg,
        Command::SetChoice {
            item: item.into(),
            value: value.into(),
            checked: true,
        },
    )
}

fn focus_item(name: &str) -> QuestionnaireEffect {
    QuestionnaireEffect::Focus(FocusTarget {
        item: name.into(),
        control: None,
    })
}

fn focus_control(name: &str, key: AnswerKey) -> QuestionnaireEffect {
    QuestionnaireEffect::Focus(FocusTarget {
        item: name.into(),
        control: Some(key),
    })
}

fn choice(value: &str) -> AnswerKey {
    AnswerKey::Choice(value.into())
}

fn three() -> Vec<ItemDef> {
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

fn key(name: KeyName) -> KeyInput {
    KeyInput::plain(name)
}

fn on_item(name: &str) -> KeyTarget {
    KeyTarget::Item(name.into())
}

fn on_answer(item: &str, key: AnswerKey) -> KeyTarget {
    KeyTarget::Answer {
        item: item.into(),
        key,
    }
}

// "owns its ordered items, progress, and navigation"
#[test]
fn owns_ordered_items_progress_and_navigation() {
    let items = three();
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(s.active(&cfg), Some("direction"));
    assert_eq!(
        s.progress(&cfg),
        ProgressState {
            current: 1,
            total: 3,
            first: true,
            last: false
        }
    );

    // A required, unanswered item blocks Next and moves focus to its first control.
    let effects = run(&mut s, &cfg, Command::Next);
    assert_eq!(
        effects,
        vec![focus_control("direction", choice("tool-calls"))]
    );
    assert_eq!(s.active(&cfg), Some("direction"));
    assert!(s.is_invalid(&cfg, "direction"));

    choose(&mut s, &cfg, "direction", "approvals");
    assert!(
        !s.is_invalid(&cfg, "direction"),
        "answering clears the shown error"
    );
    let effects = run(&mut s, &cfg, Command::Next);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::ItemChange("signals".into()),
            focus_item("signals")
        ]
    );
    assert_eq!(s.progress(&cfg).current, 2);

    let effects = run(&mut s, &cfg, Command::Previous);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::ItemChange("direction".into()),
            focus_item("direction")
        ]
    );
    assert!(s.progress(&cfg).first);
    assert!(
        run(&mut s, &cfg, Command::Previous).is_empty(),
        "no item before the first"
    );
}

// "moves between items with horizontal arrows outside radios and text entry"
#[test]
fn horizontal_arrows_outside_radios_and_text_entry() {
    let items = three();
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    choose(&mut s, &cfg, "direction", "approvals");

    // From a radio, ArrowRight is the native radio group's key.
    let on_radio = s.key_down(
        &cfg,
        &key(KeyName::ArrowRight),
        &on_answer("direction", choice("approvals")),
    );
    assert_eq!(on_radio, KeyOutcome::default());
    // From the freeform input it is the caret's.
    let on_text = s.key_down(
        &cfg,
        &key(KeyName::ArrowRight),
        &on_answer("direction", AnswerKey::Input),
    );
    assert_eq!(on_text, KeyOutcome::default());
    // From the fieldset (or a navigation button) it goes to the next item.
    let on_fieldset = s.key_down(&cfg, &key(KeyName::ArrowRight), &on_item("direction"));
    assert_eq!(
        on_fieldset,
        KeyOutcome {
            prevent_default: true,
            commands: vec![Command::Next]
        }
    );
    let on_action = s.key_down(&cfg, &key(KeyName::ArrowLeft), &KeyTarget::Action);
    assert_eq!(on_action.commands, vec![Command::Previous]);
    // A caller's own field inside the form keeps its keys.
    assert_eq!(
        s.key_down(&cfg, &key(KeyName::ArrowRight), &KeyTarget::Other),
        KeyOutcome::default()
    );
}

// "does not treat ArrowRight as an implicit skip"
#[test]
fn arrow_right_is_not_an_implicit_skip() {
    let items = three();
    let cfg = QuestionnaireConfig {
        default_item: Some("signals"),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    let out = s.key_down(&cfg, &key(KeyName::ArrowRight), &on_item("signals"));
    assert!(out.prevent_default);
    assert!(
        out.commands.is_empty(),
        "an unanswered optional item is not skipped by ArrowRight"
    );
}

// "moves vertical focus across fixed and freeform answers"
#[test]
fn vertical_focus_across_fixed_and_freeform_answers() {
    let items = three();
    let cfg = QuestionnaireConfig::new(&items);
    let s = QuestionnaireState::new(&cfg);

    // From the fieldset, ArrowDown enters at the first answer and selects it (a radio follows focus).
    let out = s.key_down(&cfg, &key(KeyName::ArrowDown), &on_item("direction"));
    assert!(out.prevent_default);
    assert_eq!(
        out.commands,
        vec![
            Command::Focus(FocusTarget {
                item: "direction".into(),
                control: Some(choice("tool-calls"))
            }),
            Command::SetChoice {
                item: "direction".into(),
                value: "tool-calls".into(),
                checked: true
            },
        ]
    );
    // ArrowUp enters at the last answer: the freeform input, which is not selected by focus.
    let out = s.key_down(&cfg, &key(KeyName::ArrowUp), &on_item("direction"));
    assert_eq!(
        out.commands,
        vec![Command::Focus(FocusTarget {
            item: "direction".into(),
            control: Some(AnswerKey::Input)
        })]
    );
    // Radio to radio is left to the browser's own radio group.
    let out = s.key_down(
        &cfg,
        &key(KeyName::ArrowDown),
        &on_answer("direction", choice("tool-calls")),
    );
    assert_eq!(out, KeyOutcome::default());
    // The last radio moves into the freeform input.
    let out = s.key_down(
        &cfg,
        &key(KeyName::ArrowDown),
        &on_answer("direction", choice("handoffs")),
    );
    assert!(out.prevent_default);
    // An empty text input lets ArrowDown wrap to the first choice (and select it).
    let out = s.key_down(
        &cfg,
        &key(KeyName::ArrowDown),
        &on_answer("direction", AnswerKey::Input),
    );
    assert_eq!(out.commands.len(), 2);
}

#[test]
fn filled_or_number_input_keeps_native_arrows() {
    let items = vec![
        ItemDef::new("a").choices(["x"]).freeform(),
        ItemDef::new("n")
            .choices(["x"])
            .freeform_kind(InputKind::Number),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "a".into(),
            text: "typed".into(),
        },
    );
    assert_eq!(
        s.key_down(
            &cfg,
            &key(KeyName::ArrowUp),
            &on_answer("a", AnswerKey::Input)
        ),
        KeyOutcome::default()
    );

    // "keeps native arrow behavior for number inputs"
    let cfg = QuestionnaireConfig {
        default_item: Some("n"),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    assert_eq!(
        s.key_down(
            &cfg,
            &key(KeyName::ArrowUp),
            &on_answer("n", AnswerKey::Input)
        ),
        KeyOutcome::default()
    );
}

// "moves vertical focus through multiple choices without toggling"
#[test]
fn vertical_focus_through_checkboxes_without_toggling() {
    let items = three();
    let cfg = QuestionnaireConfig {
        default_item: Some("signals"),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    let out = s.key_down(
        &cfg,
        &key(KeyName::ArrowDown),
        &on_answer("signals", choice("progress")),
    );
    assert_eq!(
        out.commands,
        vec![Command::Focus(FocusTarget {
            item: "signals".into(),
            control: Some(choice("decisions"))
        })]
    );
    let out = s.key_down(
        &cfg,
        &key(KeyName::ArrowUp),
        &on_answer("signals", choice("progress")),
    );
    assert_eq!(
        out.commands,
        vec![Command::Focus(FocusTarget {
            item: "signals".into(),
            control: Some(choice("next-step"))
        })],
        "wraps from the first to the last"
    );
}

// "omits disabled answers from shortcuts, navigation, and validation focus"
#[test]
fn disabled_answers_leave_shortcuts_navigation_and_focus() {
    let items = vec![ItemDef::new("q")
        .required()
        .disabled_choice("off")
        .choices(["a", "b"])];
    let cfg = QuestionnaireConfig {
        shortcuts: Some(ShortcutMode::Letters),
        ..QuestionnaireConfig::new(&items)
    };
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(s.shortcut(&cfg, "q", "off"), None);
    assert_eq!(s.shortcut(&cfg, "q", "a").as_deref(), Some("A"));
    assert_eq!(s.shortcut(&cfg, "q", "b").as_deref(), Some("B"));
    assert_eq!(s.invalid_focus_target(&cfg, "q").control, Some(choice("a")));
    let out = s.key_down(&cfg, &key(KeyName::ArrowDown), &on_item("q"));
    assert_eq!(
        out.commands[0],
        Command::Focus(FocusTarget {
            item: "q".into(),
            control: Some(choice("a"))
        })
    );
    assert!(
        choose(&mut s, &cfg, "q", "off").is_empty(),
        "a disabled choice cannot be selected"
    );
    assert_eq!(s.status(&cfg, "q"), ItemStatus::Unanswered);
}

// "counts only enabled answers in item status and validation"
#[test]
fn only_enabled_answers_count() {
    let mut items = vec![ItemDef::new("q")
        .required()
        .choices(["a"])
        .default_choice("a")];
    let cfg = QuestionnaireConfig::new(&items);
    let s = QuestionnaireState::new(&cfg);
    assert_eq!(s.status(&cfg, "q"), ItemStatus::Answered);

    items[0].choices[0].disabled = true;
    let cfg = QuestionnaireConfig::new(&items);
    assert_eq!(
        s.status(&cfg, "q"),
        ItemStatus::Unanswered,
        "a disabled default does not answer the item"
    );
    assert!(!s.is_valid(&cfg, "q"));
}

// "returns to and blocks an externally invalid item"
#[test]
fn externally_invalid_item_blocks_and_is_returned_to() {
    let items = vec![
        ItemDef::new("detail")
            .required()
            .choices(["summary", "complete"])
            .default_choice("summary")
            .invalid(true),
        ItemDef::new("audience")
            .required()
            .choices(["team", "public"]),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert!(
        s.is_invalid(&cfg, "detail"),
        "an external error shows without an attempt"
    );
    let effects = run(&mut s, &cfg, Command::Next);
    assert_eq!(effects, vec![focus_control("detail", choice("summary"))]);

    // Submitting from the second item returns to the first invalid one and focuses its control.
    let ok_items = vec![items[0].clone().invalid(false), items[1].clone()];
    let ok = QuestionnaireConfig::new(&ok_items);
    run(&mut s, &ok, Command::Next);
    choose(&mut s, &ok, "audience", "public");
    let effects = run(&mut s, &cfg, Command::Submit);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::ItemChange("detail".into()),
            focus_control("detail", choice("summary"))
        ]
    );
    assert_eq!(s.active(&cfg), Some("detail"));
}

// "treats an intentional skip as valid for an optional external error"
#[test]
fn intentional_skip_is_valid_for_optional_external_error() {
    let items = vec![
        ItemDef::new("optional").choices(["a"]).invalid(true),
        ItemDef::new("last").choices(["b"]),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert!(s.is_invalid(&cfg, "optional"));
    let effects = run(&mut s, &cfg, Command::Skip);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::AnswerChange("optional".into()),
            QuestionnaireEffect::ItemChange("last".into()),
            focus_item("last")
        ]
    );
    assert_eq!(s.status(&cfg, "optional"), ItemStatus::Skipped);
    assert!(s.is_valid(&cfg, "optional"));
    assert!(!s.is_invalid(&cfg, "optional"));
}

// "activates the first enabled item without reporting initialization as navigation"
#[test]
fn initial_item_skips_disabled_and_unknown_defaults() {
    let items = vec![
        ItemDef::new("off").disabled(true),
        ItemDef::new("on"),
        ItemDef::new("later"),
    ];
    let s = QuestionnaireState::new(&QuestionnaireConfig::new(&items));
    assert_eq!(s.active(&QuestionnaireConfig::new(&items)), Some("on"));
    for default in ["off", "missing"] {
        let cfg = QuestionnaireConfig {
            default_item: Some(default),
            ..QuestionnaireConfig::new(&items)
        };
        assert_eq!(QuestionnaireState::new(&cfg).active(&cfg), Some("on"));
    }
    let cfg = QuestionnaireConfig {
        default_item: Some("later"),
        ..QuestionnaireConfig::new(&items)
    };
    assert_eq!(
        QuestionnaireState::new(&cfg).active(&cfg),
        Some("later"),
        "resume on a saved item"
    );
}

// "supports controlled active-item navigation"
#[test]
fn controlled_navigation_reports_and_waits_for_the_caller() {
    let items = vec![
        ItemDef::new("a").choices(["x"]).default_choice("x"),
        ItemDef::new("b").choices(["y"]),
    ];
    let cfg = QuestionnaireConfig {
        controlled_item: Some("a"),
        ..QuestionnaireConfig::new(&items)
    };
    let mut s = QuestionnaireState::new(&cfg);
    let effects = run(&mut s, &cfg, Command::Next);
    assert_eq!(
        effects,
        vec![QuestionnaireEffect::ItemChange("b".into())],
        "no focus until the caller switches"
    );
    assert_eq!(s.active(&cfg), Some("a"), "the caller owns the active item");

    let moved = QuestionnaireConfig {
        controlled_item: Some("b"),
        ..cfg
    };
    assert_eq!(
        s.controlled_item_changed(&moved, Some("a")),
        vec![focus_item("b")]
    );
    // A caller-driven switch (e.g. back to an invalid item) focuses the new item too.
    assert_eq!(
        s.controlled_item_changed(&cfg, Some("b")),
        vec![focus_item("a")]
    );
    assert!(s.controlled_item_changed(&cfg, Some("a")).is_empty());

    let missing = QuestionnaireConfig {
        controlled_item: Some("nope"),
        ..cfg
    };
    assert_eq!(s.controlled_fallback(&missing).as_deref(), Some("a"));
    assert_eq!(s.controlled_fallback(&cfg), None);
}

// "treats Input as a freeform answer without selecting it on focus" + "does not require fixed radios when an
// Input can answer the item"
#[test]
fn freeform_input_answers_and_replaces_a_single_choice() {
    let items = three();
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    choose(&mut s, &cfg, "direction", "approvals");
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "direction".into(),
            text: "Something else".into(),
        },
    );
    assert!(
        !s.is_checked(&cfg, "direction", "approvals"),
        "typing replaces the radio answer"
    );
    assert!(s.input_selected(&cfg, "direction"));
    assert_eq!(
        s.values(&cfg, "direction"),
        vec!["Something else".to_string()]
    );
    assert_eq!(s.status(&cfg, "direction"), ItemStatus::Answered);

    // Choosing a radio again keeps the text as an unsubmitted draft.
    choose(&mut s, &cfg, "direction", "handoffs");
    assert!(!s.input_selected(&cfg, "direction"));
    assert_eq!(s.input_text(&cfg, "direction"), "Something else");
    assert_eq!(s.values(&cfg, "direction"), vec!["handoffs".to_string()]);

    // Clearing the text unselects it.
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "direction".into(),
            text: "   ".into(),
        },
    );
    assert!(!s.input_selected(&cfg, "direction"));
    assert_eq!(
        s.status(&cfg, "direction"),
        ItemStatus::Answered,
        "a blank draft does not drop the radio"
    );
}

// "allows fixed and freeform answers in a multiple item"
#[test]
fn multiple_item_keeps_fixed_and_freeform_answers() {
    let items = vec![ItemDef::new("m").multiple().choices(["a", "b"]).freeform()];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    choose(&mut s, &cfg, "m", "b");
    choose(&mut s, &cfg, "m", "a");
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "m".into(),
            text: "c".into(),
        },
    );
    assert_eq!(
        s.values(&cfg, "m"),
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "render order"
    );
    run(
        &mut s,
        &cfg,
        Command::ToggleChoice {
            item: "m".into(),
            value: "a".into(),
        },
    );
    assert_eq!(s.values(&cfg, "m"), vec!["b".to_string(), "c".to_string()]);
}

// "records an intentional skip separately from an unanswered item" + "clears controlled choices when an item is
// intentionally skipped"
#[test]
fn skip_clears_answers_and_is_its_own_status() {
    let items = vec![
        ItemDef::new("opt")
            .choices(["a"])
            .freeform()
            .default_choice("a"),
        ItemDef::new("req").required(),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(s.status(&cfg, "opt"), ItemStatus::Answered);
    let effects = run(&mut s, &cfg, Command::Skip);
    assert_eq!(effects[0], QuestionnaireEffect::AnswerChange("opt".into()));
    assert_eq!(s.status(&cfg, "opt"), ItemStatus::Skipped);
    assert!(!s.is_checked(&cfg, "opt", "a"));
    assert!(
        s.values(&cfg, "opt").is_empty(),
        "a skipped item submits nothing"
    );
    // Answering again leaves the skip.
    run(&mut s, &cfg, Command::Previous);
    choose(&mut s, &cfg, "opt", "a");
    assert_eq!(s.status(&cfg, "opt"), ItemStatus::Answered);
    // A required item cannot be skipped.
    run(&mut s, &cfg, Command::Next);
    assert!(run(&mut s, &cfg, Command::Skip).is_empty());
}

// "submits after skipping the final optional item"
#[test]
fn skipping_the_last_item_submits() {
    let items = vec![ItemDef::new("only").choices(["a"])];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    let effects = run(&mut s, &cfg, Command::Skip);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::AnswerChange("only".into()),
            QuestionnaireEffect::RequestSubmit
        ],
        "skipping changes the status, then submits"
    );
    assert_eq!(
        run(&mut s, &cfg, Command::Submit),
        vec![QuestionnaireEffect::Submit]
    );
}

// "validates unanswered items on native form submission"
#[test]
fn submit_validates_every_rendered_item_in_order() {
    let items = three();
    let cfg = QuestionnaireConfig {
        default_item: Some("timing"),
        ..QuestionnaireConfig::new(&items)
    };
    let mut s = QuestionnaireState::new(&cfg);
    choose(&mut s, &cfg, "timing", "now");
    let effects = run(&mut s, &cfg, Command::Submit);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::ItemChange("direction".into()),
            focus_control("direction", choice("tool-calls"))
        ]
    );
    assert!(s.is_invalid(&cfg, "direction"));
    // An item that is not rendered is not validated (shadcn validates registrations only).
    let mut s = QuestionnaireState::new(&cfg);
    choose(&mut s, &cfg, "timing", "now");
    choose(&mut s, &cfg, "signals", "risks");
    let effects = s.run(&cfg, &Command::Submit, &always, &|name| name != "direction");
    assert_eq!(effects, vec![QuestionnaireEffect::Submit]);
}

// "keeps validation active until an attempted item remains valid"
#[test]
fn validation_stays_active_after_an_attempt() {
    let items = vec![
        ItemDef::new("m").required().multiple().choices(["a", "b"]),
        ItemDef::new("z"),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert!(!s.is_invalid(&cfg, "m"), "no error before an attempt");
    run(&mut s, &cfg, Command::Next);
    assert!(s.is_invalid(&cfg, "m"));
    choose(&mut s, &cfg, "m", "a");
    assert!(!s.is_invalid(&cfg, "m"));
    run(
        &mut s,
        &cfg,
        Command::ToggleChoice {
            item: "m".into(),
            value: "a".into(),
        },
    );
    assert!(
        s.is_invalid(&cfg, "m"),
        "unchecking the last answer shows the error again"
    );
}

// "keeps live answers when default props change and uses new defaults on reset" + "resets answers, skips,
// validation, and the initial item"
#[test]
fn reset_restores_defaults_skips_validation_and_initial_item() {
    let items = vec![
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
            .default_input("Keep the API stable."),
    ];
    let cfg = QuestionnaireConfig {
        default_item: Some("verification"),
        ..QuestionnaireConfig::new(&items)
    };
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(s.active(&cfg), Some("verification"));
    assert_eq!(
        s.values(&cfg, "verification"),
        vec!["tests".to_string(), "typecheck".to_string()]
    );
    assert_eq!(
        s.values(&cfg, "notes"),
        vec!["Keep the API stable.".to_string()]
    );

    run(&mut s, &cfg, Command::Next);
    run(&mut s, &cfg, Command::Skip);
    run(&mut s, &cfg, Command::Previous);
    run(&mut s, &cfg, Command::Previous);
    choose(&mut s, &cfg, "change", "cutover");

    // New defaults do not touch live answers...
    let mut changed = items.clone();
    changed[0].default_choices = vec!["cutover".into()];
    changed[0].choices.push(ChoiceDef {
        value: "hybrid".into(),
        disabled: false,
    });
    changed[0].default_choices = vec!["hybrid".into()];
    let cfg2 = QuestionnaireConfig {
        default_item: Some("verification"),
        ..QuestionnaireConfig::new(&changed)
    };
    s.materialize(&cfg2);
    assert_eq!(s.values(&cfg2, "change"), vec!["cutover".to_string()]);

    // ...and reset uses them.
    let effects = run(&mut s, &cfg2, Command::Reset);
    assert!(effects.contains(&QuestionnaireEffect::AnswerChange("change".into())));
    assert!(
        effects.contains(&QuestionnaireEffect::AnswerChange("notes".into())),
        "the skip is undone"
    );
    assert!(effects.ends_with(&[
        QuestionnaireEffect::ItemChange("verification".into()),
        focus_item("verification")
    ]));
    assert_eq!(s.values(&cfg2, "change"), vec!["hybrid".to_string()]);
    assert_eq!(s.status(&cfg2, "notes"), ItemStatus::Answered);
}

// "skips disabled items" + conditional items
#[test]
fn conditional_items_leave_navigation_progress_and_validation() {
    let mut items = vec![
        ItemDef::new("runtime")
            .required()
            .choices(["local", "cloud"])
            .default_choice("local"),
        ItemDef::new("environment")
            .required()
            .choices(["preview", "staging"])
            .disabled(true),
        ItemDef::new("approval")
            .required()
            .choices(["writes", "commands"]),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(s.progress(&cfg).total, 2);
    run(&mut s, &cfg, Command::Next);
    assert_eq!(
        s.active(&cfg),
        Some("approval"),
        "the disabled item is passed over"
    );
    choose(&mut s, &cfg, "approval", "writes");
    assert_eq!(
        run(&mut s, &cfg, Command::Submit),
        vec![QuestionnaireEffect::Submit],
        "a disabled required item is valid"
    );

    // The earlier answer turns the item on: it is in the flow again and blocks submit.
    items[1].disabled = false;
    let cfg = QuestionnaireConfig::new(&items);
    assert_eq!(
        s.progress(&cfg),
        ProgressState {
            current: 3,
            total: 3,
            first: false,
            last: true
        }
    );
    assert_eq!(
        run(&mut s, &cfg, Command::Submit),
        vec![
            QuestionnaireEffect::ItemChange("environment".into()),
            focus_control("environment", choice("preview"))
        ]
    );

    // An active item that stops applying falls back to the first that does.
    items[1].disabled = true;
    let cfg = QuestionnaireConfig::new(&items);
    assert_eq!(s.active(&cfg), Some("runtime"));
}

// "assigns letter shortcuts to enabled answers in DOM order" + "selects answers with scoped shortcuts and
// confirms with Enter"
#[test]
fn letter_shortcuts_select_and_enter_confirms() {
    let items = three();
    let cfg = QuestionnaireConfig {
        shortcuts: Some(ShortcutMode::Letters),
        ..QuestionnaireConfig::new(&items)
    };
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(
        s.shortcut(&cfg, "direction", "handoffs").as_deref(),
        Some("C")
    );
    let out = s.key_down(
        &cfg,
        &key(KeyName::Character("b".into())),
        &on_item("direction"),
    );
    assert!(out.prevent_default);
    assert_eq!(
        out.commands,
        vec![
            Command::Focus(FocusTarget {
                item: "direction".into(),
                control: Some(choice("approvals"))
            }),
            Command::SetChoice {
                item: "direction".into(),
                value: "approvals".into(),
                checked: true
            }
        ]
    );
    for command in out.commands {
        run(&mut s, &cfg, command);
    }
    assert_eq!(
        s.choice_key_shortcuts(&cfg, "direction", "approvals")
            .as_deref(),
        Some("B Enter")
    );
    assert_eq!(
        s.choice_key_shortcuts(&cfg, "direction", "tool-calls")
            .as_deref(),
        Some("A")
    );

    // A letter past the item's choices, or typed into the freeform input, is not a shortcut.
    assert_eq!(
        s.key_down(
            &cfg,
            &key(KeyName::Character("z".into())),
            &on_item("direction")
        ),
        KeyOutcome::default()
    );
    assert_eq!(
        s.key_down(
            &cfg,
            &key(KeyName::Character("a".into())),
            &on_answer("direction", AnswerKey::Input)
        ),
        KeyOutcome::default()
    );

    // Enter on the checked answer confirms.
    let out = s.key_down(
        &cfg,
        &key(KeyName::Enter),
        &on_answer("direction", choice("approvals")),
    );
    assert_eq!(
        out,
        KeyOutcome {
            prevent_default: true,
            commands: vec![Command::Confirm]
        }
    );
}

// "toggles multiple answers with scoped shortcuts"
#[test]
fn shortcuts_toggle_checkboxes() {
    let items = three();
    let cfg = QuestionnaireConfig {
        shortcuts: Some(ShortcutMode::Letters),
        default_item: Some("signals"),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    let out = s.key_down(
        &cfg,
        &key(KeyName::Character("d".into())),
        &on_item("signals"),
    );
    assert_eq!(
        out.commands[1],
        Command::ToggleChoice {
            item: "signals".into(),
            value: "next-step".into()
        }
    );
}

// "supports number shortcuts and confirms only from a filled answer" + "does not implicitly submit from an
// unselected answer"
#[test]
fn number_shortcuts_and_enter_only_from_a_filled_answer() {
    let items = vec![ItemDef::new("q").required().choices(["a", "b"]).freeform()];
    let cfg = QuestionnaireConfig {
        shortcuts: Some(ShortcutMode::Numbers),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    assert_eq!(s.shortcut(&cfg, "q", "b").as_deref(), Some("2"));
    let out = s.key_down(&cfg, &key(KeyName::Character("2".into())), &on_item("q"));
    assert!(out.prevent_default);
    // Enter on an unchecked answer is swallowed (no implicit submit) but confirms nothing.
    let out = s.key_down(&cfg, &key(KeyName::Enter), &on_answer("q", choice("a")));
    assert_eq!(
        out,
        KeyOutcome {
            prevent_default: true,
            commands: vec![]
        }
    );
    // Enter on a caller's own element is left alone.
    assert_eq!(
        s.key_down(&cfg, &key(KeyName::Enter), &KeyTarget::Other),
        KeyOutcome::default()
    );
}

// "validates, advances, and submits with Command or Control plus Enter"
#[test]
fn mod_enter_validates_advances_and_submits() {
    let items = vec![
        ItemDef::new("a").required().choices(["x"]),
        ItemDef::new("b").choices(["y"]),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    for (meta, ctrl) in [(true, false), (false, true)] {
        let out = s.key_down(
            &cfg,
            &KeyInput {
                meta,
                ctrl,
                ..key(KeyName::Enter)
            },
            &KeyTarget::Other,
        );
        assert_eq!(
            out.commands,
            vec![Command::Confirm],
            "works from anywhere in the form"
        );
    }
    assert_eq!(
        run(&mut s, &cfg, Command::Confirm),
        vec![focus_control("a", choice("x"))]
    );
    choose(&mut s, &cfg, "a", "x");
    assert_eq!(
        run(&mut s, &cfg, Command::Confirm),
        vec![QuestionnaireEffect::ItemChange("b".into()), focus_item("b")]
    );
    // An unanswered optional item is not valid: it needs an answer or an explicit skip (shadcn's rule).
    assert_eq!(
        run(&mut s, &cfg, Command::Confirm),
        vec![focus_control("b", choice("y"))]
    );
    choose(&mut s, &cfg, "b", "y");
    assert_eq!(
        run(&mut s, &cfg, Command::Confirm),
        vec![QuestionnaireEffect::RequestSubmit],
        "the last item submits"
    );
}

// "does not handle modified, repeated, prevented, or composing keys"
#[test]
fn ignores_modified_repeated_and_composing_keys() {
    let items = three();
    let cfg = QuestionnaireConfig {
        shortcuts: Some(ShortcutMode::Letters),
        ..QuestionnaireConfig::new(&items)
    };
    let s = QuestionnaireState::new(&cfg);
    let a = || key(KeyName::Character("a".into()));
    assert_eq!(
        s.key_down(&cfg, &KeyInput { alt: true, ..a() }, &on_item("direction")),
        KeyOutcome::default()
    );
    assert_eq!(
        s.key_down(&cfg, &KeyInput { ctrl: true, ..a() }, &on_item("direction")),
        KeyOutcome::default()
    );
    assert_eq!(
        s.key_down(
            &cfg,
            &KeyInput {
                composing: true,
                ..a()
            },
            &on_item("direction")
        ),
        KeyOutcome::default()
    );
    let repeated = s.key_down(
        &cfg,
        &KeyInput {
            repeat: true,
            ..a()
        },
        &on_item("direction"),
    );
    assert!(
        repeated.prevent_default && repeated.commands.is_empty(),
        "a held key acts once"
    );
    let mod_enter_shift = KeyInput {
        meta: true,
        shift: true,
        ..key(KeyName::Enter)
    };
    assert_eq!(
        s.key_down(&cfg, &mod_enter_shift, &on_item("direction")),
        KeyOutcome::default()
    );
}

// "allows a freeform answer with native validation enabled" + "validates selected native controls without
// validating unselected drafts"
#[test]
fn native_validity_checks_only_selected_controls() {
    let items = vec![
        ItemDef::new("email")
            .required()
            .choices(["none"])
            .freeform_kind(InputKind::Email),
        ItemDef::new("next"),
    ];
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    let input_invalid = |_: &str, key: &AnswerKey| *key != AnswerKey::Input;
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "email".into(),
            text: "not-an-email".into(),
        },
    );
    let effects = s.run(&cfg, &Command::Next, &input_invalid, &all);
    assert_eq!(
        effects,
        vec![QuestionnaireEffect::ReportValidity(FocusTarget {
            item: "email".into(),
            control: Some(AnswerKey::Input)
        })]
    );
    // With a radio chosen, the invalid text is an unsubmitted draft and is not validated.
    choose(&mut s, &cfg, "email", "none");
    let effects = s.run(&cfg, &Command::Next, &input_invalid, &all);
    assert_eq!(
        effects,
        vec![
            QuestionnaireEffect::ItemChange("next".into()),
            focus_item("next")
        ]
    );
}

// "keeps Enter metadata on only the selected freeform answer"
#[test]
fn key_shortcut_metadata() {
    let items = three();
    let cfg = QuestionnaireConfig::new(&items);
    let mut s = QuestionnaireState::new(&cfg);
    assert_eq!(
        s.item_key_shortcuts(&cfg, "direction").as_deref(),
        Some("Meta+Enter Control+Enter ArrowUp ArrowDown")
    );
    assert_eq!(
        s.item_key_shortcuts(&cfg, "signals"),
        None,
        "inactive items announce nothing"
    );
    assert_eq!(s.input_key_shortcuts(&cfg, "direction"), None);
    run(
        &mut s,
        &cfg,
        Command::SetInput {
            item: "direction".into(),
            text: "x".into(),
        },
    );
    assert_eq!(
        s.input_key_shortcuts(&cfg, "direction").as_deref(),
        Some("Enter")
    );
    assert_eq!(
        s.item_key_shortcuts(&cfg, "direction").as_deref(),
        Some("Meta+Enter Control+Enter ArrowUp ArrowDown ArrowRight")
    );
    choose(&mut s, &cfg, "direction", "approvals");
    assert_eq!(
        s.input_key_shortcuts(&cfg, "direction"),
        None,
        "the draft is not the answer"
    );
    run(&mut s, &cfg, Command::Next);
    assert_eq!(
        s.item_key_shortcuts(&cfg, "signals").as_deref(),
        Some("Meta+Enter Control+Enter ArrowUp ArrowDown ArrowLeft")
    );
}

// "preserves a compatible selection when multiple changes"
#[test]
fn defaults_follow_single_or_multiple() {
    let single = ItemDef::new("s")
        .choices(["a", "b"])
        .default_choice("b")
        .default_choice("a")
        .freeform()
        .default_input("t");
    let items = vec![single.clone()];
    let cfg = QuestionnaireConfig::new(&items);
    let s = QuestionnaireState::new(&cfg);
    assert_eq!(
        s.values(&cfg, "s"),
        vec!["a".to_string()],
        "one answer, the first in render order"
    );
    assert_eq!(
        s.input_text(&cfg, "s"),
        "t",
        "the default text stays as a draft"
    );

    let items = vec![single.multiple()];
    let cfg = QuestionnaireConfig::new(&items);
    let s = QuestionnaireState::new(&cfg);
    assert_eq!(
        s.values(&cfg, "s"),
        vec!["a".to_string(), "b".to_string(), "t".to_string()]
    );
}

#[test]
fn definition_warnings_name_the_problems() {
    let items = vec![
        ItemDef::new("a").choices(["x", "x"]).default_choice("y"),
        ItemDef::new("a"),
        ItemDef::new("off").disabled(true),
    ];
    let warnings = definition_warnings(&items, Some("off"));
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    assert!(definition_warnings(&three(), Some("signals")).is_empty());
}

#[test]
fn shortcut_mode_keys() {
    assert_eq!(ShortcutMode::Letters.keys().len(), 26);
    assert_eq!(
        ShortcutMode::Numbers.keys(),
        (1..=9).map(|n| n.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(
        ShortcutMode::Letters.shortcut_from_key("q").as_deref(),
        Some("Q")
    );
    assert_eq!(ShortcutMode::Numbers.shortcut_from_key("0"), None);
    assert_eq!(ShortcutMode::Letters.shortcut_from_key("é"), None);
}
