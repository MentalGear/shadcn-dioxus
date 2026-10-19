//! The questionnaire's state machine, as plain Rust with no Dioxus in it.
//!
//! Everything shadcn's `@shadcn/react/questionnaire` decides at runtime lives here: which item is active, which
//! items count (a disabled item is a conditional item that does not apply), what each item's answer is, whether
//! it is valid, where focus goes after a navigation or a failed validation, which key does what, which answer a
//! shortcut letter names and what the native form will serialize. The Dioxus components in the parent module
//! only render this state and forward events into it, so every rule here is unit tested with `cargo test`.
//!
//! # One source of truth (the construction)
//!
//! shadcn's headless package learns its items twice: from the optional `Root.items` collection and from the
//! `Questionnaire.Item`/`Questionnaire.Choice` elements that register themselves at runtime, and then spends
//! ~140 lines (`getCollectionRegistrationWarnings`) warning when the two disagree about `required`,
//! `disabled`, a choice's presence or the choice order. Here the [`ItemDef`] list is the ONLY place an item's
//! behaviour is declared: name, required, multiple, disabled (conditional), external `invalid`, its choices
//! (order, disabled), its freeform input, and its default answers. The item and choice components take only a
//! name or a value and read the rest, so the whole "props disagree with the collection" class cannot happen.
//! What is left of shadcn's warnings (duplicate names, duplicate choice values, an unusable `default_item`) is
//! [`definition_warnings`].

use std::collections::{HashMap, HashSet};

/// Which keys select answers: shadcn's `shortcuts="letters" | "numbers"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutMode {
    /// `A` to `Z`, case-insensitive.
    Letters,
    /// `1` to `9`.
    Numbers,
}

impl ShortcutMode {
    /// The value of the root's `data-shortcuts` attribute.
    pub fn as_str(self) -> &'static str {
        match self {
            ShortcutMode::Letters => "letters",
            ShortcutMode::Numbers => "numbers",
        }
    }

    /// The shortcut keys in assignment order.
    pub fn keys(self) -> Vec<String> {
        match self {
            ShortcutMode::Letters => (b'A'..=b'Z').map(|c| (c as char).to_string()).collect(),
            ShortcutMode::Numbers => (1..=9).map(|n| n.to_string()).collect(),
        }
    }

    /// The shortcut a pressed key names, if any (`a` names `A` in letter mode).
    pub fn shortcut_from_key(self, key: &str) -> Option<String> {
        let normalized = match self {
            ShortcutMode::Letters => key.to_uppercase(),
            ShortcutMode::Numbers => key.to_string(),
        };
        self.keys().into_iter().find(|k| *k == normalized)
    }
}

/// The `type` of a freeform answer input (shadcn's `QuestionnaireInputType`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputKind {
    /// `type="text"`.
    #[default]
    Text,
    /// `type="email"`.
    Email,
    /// `type="password"`.
    Password,
    /// `type="search"`.
    Search,
    /// `type="tel"`.
    Tel,
    /// `type="url"`.
    Url,
    /// `type="number"`.
    Number,
    /// `type="date"`.
    Date,
    /// `type="datetime-local"`.
    DatetimeLocal,
    /// `type="month"`.
    Month,
    /// `type="time"`.
    Time,
    /// `type="week"`.
    Week,
}

impl InputKind {
    /// The HTML `type` attribute value.
    pub fn as_str(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Email => "email",
            InputKind::Password => "password",
            InputKind::Search => "search",
            InputKind::Tel => "tel",
            InputKind::Url => "url",
            InputKind::Number => "number",
            InputKind::Date => "date",
            InputKind::DatetimeLocal => "datetime-local",
            InputKind::Month => "month",
            InputKind::Time => "time",
            InputKind::Week => "week",
        }
    }

    /// Whether ArrowUp/ArrowDown may leave this input while it is empty. A number, date or time input keeps
    /// its native arrow behaviour (stepping the value), as in shadcn's `isEmptyNavigableInput`.
    pub fn arrows_leave_when_empty(self) -> bool {
        matches!(
            self,
            InputKind::Text
                | InputKind::Email
                | InputKind::Password
                | InputKind::Search
                | InputKind::Tel
                | InputKind::Url
        )
    }
}

/// One fixed answer of an item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceDef {
    /// The value the native form submits under the item's name.
    pub value: String,
    /// A disabled choice is rendered but cannot be selected, gets no shortcut and does not count as an answer.
    pub disabled: bool,
}

/// The freeform "other" answer of an item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputDef {
    /// The input's `type`.
    pub kind: InputKind,
    /// A disabled input cannot answer the item.
    pub disabled: bool,
}

/// Everything that decides how one question behaves. See the module docs for why this is the only place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemDef {
    /// The item's name: its `fieldset`, and the name its answers are submitted under.
    pub name: String,
    /// A required item cannot be skipped and blocks Next/Submit until it is answered.
    pub required: bool,
    /// Checkboxes (any number of answers) instead of radios (one answer).
    pub multiple: bool,
    /// A disabled item does not apply: it leaves the navigation, the progress count and validation. This is
    /// how conditional items are expressed (compute it from earlier answers).
    pub disabled: bool,
    /// An external validation error (e.g. from a schema) that blocks the item until it is cleared.
    pub invalid: bool,
    /// The fixed answers, in the order they are rendered (shortcuts follow this order).
    pub choices: Vec<ChoiceDef>,
    /// The freeform answer, if the item has one.
    pub input: Option<InputDef>,
    /// Choices selected when the questionnaire starts and after a reset (resume).
    pub default_choices: Vec<String>,
    /// The freeform answer's initial text (resume). A non-blank default selects the input.
    pub default_input: String,
}

impl ItemDef {
    /// An optional, single-choice item with no choices yet.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            multiple: false,
            disabled: false,
            invalid: false,
            choices: Vec::new(),
            input: None,
            default_choices: Vec::new(),
            default_input: String::new(),
        }
    }

    /// Makes the item required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Makes the item accept several answers (checkboxes).
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    /// Disables the item when `disabled` is true: it does not apply (a conditional item).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the item externally invalid when `invalid` is true.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Adds an enabled choice.
    pub fn choice(mut self, value: impl Into<String>) -> Self {
        self.choices.push(ChoiceDef {
            value: value.into(),
            disabled: false,
        });
        self
    }

    /// Adds several enabled choices.
    pub fn choices<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for value in values {
            self = self.choice(value);
        }
        self
    }

    /// Adds a disabled choice.
    pub fn disabled_choice(mut self, value: impl Into<String>) -> Self {
        self.choices.push(ChoiceDef {
            value: value.into(),
            disabled: true,
        });
        self
    }

    /// Adds a freeform text answer.
    pub fn freeform(self) -> Self {
        self.freeform_kind(InputKind::Text)
    }

    /// Adds a freeform answer of the given input type.
    pub fn freeform_kind(mut self, kind: InputKind) -> Self {
        self.input = Some(InputDef {
            kind,
            disabled: false,
        });
        self
    }

    /// Selects `value` initially and after a reset.
    pub fn default_choice(mut self, value: impl Into<String>) -> Self {
        self.default_choices.push(value.into());
        self
    }

    /// Fills the freeform answer initially and after a reset.
    pub fn default_input(mut self, text: impl Into<String>) -> Self {
        self.default_input = text.into();
        self
    }

    fn choice_def(&self, value: &str) -> Option<&ChoiceDef> {
        self.choices.iter().find(|c| c.value == value)
    }

    fn choice_enabled(&self, value: &str) -> bool {
        !self.disabled && self.choice_def(value).is_some_and(|c| !c.disabled)
    }

    fn input_enabled(&self) -> bool {
        !self.disabled && self.input.as_ref().is_some_and(|i| !i.disabled)
    }
}

/// An item's answer status, shadcn's `QuestionnaireItemStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemStatus {
    /// No enabled answer is selected.
    Unanswered,
    /// At least one enabled answer is selected.
    Answered,
    /// The item was intentionally skipped.
    Skipped,
}

impl ItemStatus {
    /// The `data-status` value.
    pub fn as_str(self) -> &'static str {
        match self {
            ItemStatus::Unanswered => "unanswered",
            ItemStatus::Answered => "answered",
            ItemStatus::Skipped => "skipped",
        }
    }
}

/// One answer control inside an item.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnswerKey {
    /// The fixed choice with this value.
    Choice(String),
    /// The freeform input.
    Input,
}

/// What a focus effect lands on.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FocusTarget {
    /// The item.
    pub item: String,
    /// One of its answer controls, or `None` for the item's `fieldset` itself.
    pub control: Option<AnswerKey>,
}

/// The element that has focus when a key is pressed, as the components track it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyTarget {
    /// Nothing tracked.
    #[default]
    None,
    /// An item's `fieldset`.
    Item(String),
    /// An answer control.
    Answer {
        /// Its item.
        item: String,
        /// Which control.
        key: AnswerKey,
    },
    /// A Previous/Skip/Next/Submit button.
    Action,
    /// Some other element inside the form. Treated as text entry (no shortcuts, no horizontal navigation),
    /// because a caller's own field inside the form must keep its keys.
    Other,
}

/// The key of a keydown, reduced to what the questionnaire reacts to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyName {
    /// Enter.
    Enter,
    /// ArrowUp.
    ArrowUp,
    /// ArrowDown.
    ArrowDown,
    /// ArrowLeft.
    ArrowLeft,
    /// ArrowRight.
    ArrowRight,
    /// A printable key.
    Character(String),
    /// Anything else.
    Other,
}

/// A keydown, as plain data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyInput {
    /// The key.
    pub key: KeyName,
    /// Meta (Command) held.
    pub meta: bool,
    /// Control held.
    pub ctrl: bool,
    /// Alt held.
    pub alt: bool,
    /// Shift held.
    pub shift: bool,
    /// An auto-repeat.
    pub repeat: bool,
    /// Part of an IME composition.
    pub composing: bool,
}

impl KeyInput {
    /// A plain press of `key`.
    pub fn plain(key: KeyName) -> Self {
        Self {
            key,
            meta: false,
            ctrl: false,
            alt: false,
            shift: false,
            repeat: false,
            composing: false,
        }
    }
}

/// A state transition a component asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Go to the previous item.
    Previous,
    /// Validate the active item and go to the next one.
    Next,
    /// Validate the active item, then go to the next one or submit on the last (Enter, Mod+Enter).
    Confirm,
    /// Skip the active optional item, then go on or submit on the last.
    Skip,
    /// The form's submit event: validate every rendered item.
    Submit,
    /// The form's reset event: restore the default answers and the initial item.
    Reset,
    /// Move focus.
    Focus(FocusTarget),
    /// Check (or uncheck) a choice, as a click on it would.
    SetChoice {
        /// The item.
        item: String,
        /// The choice value.
        value: String,
        /// Checked or not.
        checked: bool,
    },
    /// Toggle a checkbox choice, as a click on it would.
    ToggleChoice {
        /// The item.
        item: String,
        /// The choice value.
        value: String,
    },
    /// The freeform input's text changed.
    SetInput {
        /// The item.
        item: String,
        /// The new text.
        text: String,
    },
}

impl Command {
    /// Whether running it validates, so a component with native validation must first collect the controls'
    /// validity.
    pub fn validates(&self) -> bool {
        matches!(self, Command::Next | Command::Confirm | Command::Submit)
    }
}

/// What a transition asks the outside world to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionnaireEffect {
    /// Focus an element (after the DOM shows the new state).
    Focus(FocusTarget),
    /// Focus a natively invalid control and show the browser's validation message.
    ReportValidity(FocusTarget),
    /// Submit the form (`requestSubmit`, so the submit event and its validation run).
    RequestSubmit,
    /// The active item changed (or, when controlled, should change) to this one: call `on_item_change`.
    ItemChange(String),
    /// This item's answer changed: call `on_answer_change`.
    AnswerChange(String),
    /// Validation passed for a submit: call `on_submit`.
    Submit,
}

/// The result of a keydown: whether to `preventDefault`, and what to run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyOutcome {
    /// Call `preventDefault()` on the event.
    pub prevent_default: bool,
    /// Commands to run, in order.
    pub commands: Vec<Command>,
}

/// The progress values: shadcn's `QuestionnaireRootState`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgressState {
    /// 1-based position of the active item among the items that apply (0 when there is none).
    pub current: usize,
    /// How many items apply.
    pub total: usize,
    /// The active item is the first.
    pub first: bool,
    /// The active item is the last.
    pub last: bool,
}

/// The caller-owned inputs of the machine, borrowed for one query or transition.
#[derive(Clone, Copy, Debug)]
pub struct QuestionnaireConfig<'a> {
    /// The item definitions, in order.
    pub items: &'a [ItemDef],
    /// The controlled active item, if the caller controls it.
    pub controlled_item: Option<&'a str>,
    /// The item to start on (and to return to on reset).
    pub default_item: Option<&'a str>,
    /// The answer shortcut keys.
    pub shortcuts: Option<ShortcutMode>,
}

impl<'a> QuestionnaireConfig<'a> {
    /// An uncontrolled configuration with no default item and no shortcuts.
    pub fn new(items: &'a [ItemDef]) -> Self {
        Self {
            items,
            controlled_item: None,
            default_item: None,
            shortcuts: None,
        }
    }

    fn def(&self, name: &str) -> Option<&'a ItemDef> {
        self.items.iter().find(|d| d.name == name)
    }

    fn enabled(&self) -> impl Iterator<Item = &'a ItemDef> + 'a {
        self.items.iter().filter(|d| !d.disabled)
    }

    fn enabled_index(&self, name: &str) -> Option<usize> {
        self.enabled().position(|d| d.name == name)
    }

    /// shadcn's `getInitialItemName`: the default item if it applies, else the first item that applies.
    pub fn initial_item(&self) -> Option<&'a str> {
        if let Some(default) = self.default_item.and_then(|n| self.def(n)) {
            if !default.disabled {
                return Some(default.name.as_str());
            }
        }
        self.enabled().next().map(|d| d.name.as_str())
    }
}

/// One item's live answer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ItemAnswer {
    /// Selected choice values (any order; reads go through the definition's order).
    choices: Vec<String>,
    /// The freeform text (kept while another answer is selected, as a draft).
    input: String,
    /// The freeform text is the (or an) answer.
    input_selected: bool,
    /// Intentionally skipped.
    skipped: bool,
    /// Validation has been attempted, so an invalid state is shown from now on.
    attempted: bool,
}

impl ItemAnswer {
    fn defaults(def: &ItemDef) -> Self {
        let mut choices: Vec<String> = def
            .choices
            .iter()
            .filter(|c| def.default_choices.contains(&c.value))
            .map(|c| c.value.clone())
            .collect();
        let filled = has_text(&def.default_input) && def.input.is_some();
        let input_selected = if def.multiple {
            filled
        } else {
            choices.truncate(1);
            choices.is_empty() && filled
        };
        Self {
            choices,
            input: def.default_input.clone(),
            input_selected,
            skipped: false,
            attempted: false,
        }
    }
}

fn has_text(text: &str) -> bool {
    !text.trim().is_empty()
}

/// The questionnaire's mutable state: the uncontrolled active item and every item's answer. Pair it with a
/// [`QuestionnaireConfig`] for each query or transition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuestionnaireState {
    active: Option<String>,
    answers: HashMap<String, ItemAnswer>,
    /// Controlled mode: the focus to apply once the caller's `item` arrives (shadcn's `pendingFocusRef`).
    pending_focus: Option<(String, FocusKind)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusKind {
    Item,
    Invalid,
}

impl QuestionnaireState {
    /// The state a questionnaire starts in: the initial item active and every item at its default answer.
    pub fn new(cfg: &QuestionnaireConfig) -> Self {
        let mut state = Self {
            active: cfg.initial_item().map(str::to_string),
            ..Self::default()
        };
        state.materialize(cfg);
        state
    }

    /// Records the default answer of every item that has none yet, so a later change of an item's defaults
    /// only applies on reset (shadcn: "keeps live answers when default props change and uses new defaults on
    /// reset"). Call it when the definitions change.
    pub fn materialize(&mut self, cfg: &QuestionnaireConfig) {
        for def in cfg.items {
            self.answers
                .entry(def.name.clone())
                .or_insert_with(|| ItemAnswer::defaults(def));
        }
    }

    /// Whether every item already has a recorded answer (so [`QuestionnaireState::materialize`] would change
    /// nothing).
    pub fn is_materialized(&self, cfg: &QuestionnaireConfig) -> bool {
        cfg.items.iter().all(|d| self.answers.contains_key(&d.name))
    }

    fn answer(&self, def: &ItemDef) -> ItemAnswer {
        self.answers
            .get(&def.name)
            .cloned()
            .unwrap_or_else(|| ItemAnswer::defaults(def))
    }

    fn answer_mut(&mut self, def: &ItemDef) -> &mut ItemAnswer {
        self.answers
            .entry(def.name.clone())
            .or_insert_with(|| ItemAnswer::defaults(def))
    }

    // ---- queries -------------------------------------------------------------------------------------

    /// The active item's name. A controlled item wins; an item that no longer applies falls back to the
    /// first item that does (shadcn's layout effect does the same).
    pub fn active<'a>(&'a self, cfg: &QuestionnaireConfig<'a>) -> Option<&'a str> {
        let raw = cfg.controlled_item.or(self.active.as_deref());
        match raw {
            Some(name) if cfg.enabled_index(name).is_some() => Some(name),
            Some(_) => cfg.enabled().next().map(|d| d.name.as_str()),
            None => cfg.initial_item(),
        }
    }

    /// In controlled mode, the item the caller should switch to because its `item` does not apply (or is
    /// missing): shadcn's `setItem(logicalItems[0])`. `None` when nothing needs to change.
    pub fn controlled_fallback(&self, cfg: &QuestionnaireConfig) -> Option<String> {
        let controlled = cfg.controlled_item?;
        if cfg.enabled_index(controlled).is_some() {
            return None;
        }
        cfg.enabled().next().map(|d| d.name.clone())
    }

    /// Whether `name` is the active item.
    pub fn is_active(&self, cfg: &QuestionnaireConfig, name: &str) -> bool {
        self.active(cfg) == Some(name)
    }

    /// The progress values.
    pub fn progress(&self, cfg: &QuestionnaireConfig) -> ProgressState {
        let total = cfg.enabled().count();
        let index = self.active(cfg).and_then(|n| cfg.enabled_index(n));
        ProgressState {
            current: index.map_or(0, |i| i + 1),
            total,
            first: total > 0 && index == Some(0),
            last: total > 0 && index == Some(total - 1),
        }
    }

    /// Whether the active item is required (`None` when there is no active item). Drives Skip's visibility.
    pub fn active_required(&self, cfg: &QuestionnaireConfig) -> Option<bool> {
        self.active(cfg)
            .and_then(|n| cfg.def(n))
            .map(|d| d.required)
    }

    /// The item's status.
    pub fn status(&self, cfg: &QuestionnaireConfig, name: &str) -> ItemStatus {
        let Some(def) = cfg.def(name) else {
            return ItemStatus::Unanswered;
        };
        let answer = self.answer(def);
        if answer.skipped {
            ItemStatus::Skipped
        } else if self
            .enabled_answers(def)
            .iter()
            .any(|key| self.key_selected(def, &answer, key))
        {
            ItemStatus::Answered
        } else {
            ItemStatus::Unanswered
        }
    }

    fn key_selected(&self, def: &ItemDef, answer: &ItemAnswer, key: &AnswerKey) -> bool {
        match key {
            AnswerKey::Choice(v) => answer.choices.contains(v) && def.choice_def(v).is_some(),
            AnswerKey::Input => answer.input_selected && has_text(&answer.input),
        }
    }

    /// shadcn's `valid`: a disabled item, an intentionally skipped optional item, or an answered item without
    /// an external error.
    pub fn is_valid(&self, cfg: &QuestionnaireConfig, name: &str) -> bool {
        let Some(def) = cfg.def(name) else {
            return true;
        };
        let status = self.status(cfg, name);
        def.disabled
            || (status == ItemStatus::Skipped && !def.required)
            || (!def.invalid && status == ItemStatus::Answered)
    }

    /// shadcn's `invalid`: shown once validation was attempted (or always, for an external error), except on
    /// a disabled or intentionally skipped item.
    pub fn is_invalid(&self, cfg: &QuestionnaireConfig, name: &str) -> bool {
        let Some(def) = cfg.def(name) else {
            return false;
        };
        let status = self.status(cfg, name);
        let skipped_optional = status == ItemStatus::Skipped && !def.required;
        !def.disabled
            && !skipped_optional
            && (def.invalid || (self.answer(def).attempted && !self.is_valid(cfg, name)))
    }

    /// Whether a choice is checked. A skipped item shows nothing checked.
    pub fn is_checked(&self, cfg: &QuestionnaireConfig, name: &str, value: &str) -> bool {
        cfg.def(name).is_some_and(|def| {
            let answer = self.answer(def);
            !answer.skipped && answer.choices.iter().any(|v| v == value)
        })
    }

    /// The freeform input's current text.
    pub fn input_text(&self, cfg: &QuestionnaireConfig, name: &str) -> String {
        cfg.def(name)
            .map(|def| self.answer(def).input)
            .unwrap_or_default()
    }

    /// Whether the freeform input is the selected answer (named and submitted). Shadcn's `selected`.
    pub fn input_selected(&self, cfg: &QuestionnaireConfig, name: &str) -> bool {
        cfg.def(name).is_some_and(|def| {
            let answer = self.answer(def);
            !answer.skipped && answer.input_selected
        })
    }

    /// Whether the freeform input holds non-blank text (shadcn's `filled`, `data-filled`).
    pub fn input_filled(&self, cfg: &QuestionnaireConfig, name: &str) -> bool {
        has_text(&self.input_text(cfg, name))
    }

    /// The enabled answer controls of an item in render order: its enabled choices, then its input.
    fn enabled_answers(&self, def: &ItemDef) -> Vec<AnswerKey> {
        if def.disabled {
            return Vec::new();
        }
        let mut keys: Vec<AnswerKey> = def
            .choices
            .iter()
            .filter(|c| !c.disabled)
            .map(|c| AnswerKey::Choice(c.value.clone()))
            .collect();
        if def.input_enabled() {
            keys.push(AnswerKey::Input);
        }
        keys
    }

    /// Whether an answer control is filled: a checked choice, or the selected input with text.
    fn key_filled(&self, def: &ItemDef, key: &AnswerKey) -> bool {
        let answer = self.answer(def);
        !answer.skipped && self.key_selected(def, &answer, key)
    }

    /// The shortcut assigned to a choice: the enabled choices of the item, in definition order, take the keys
    /// in order (shadcn's `getShortcutByChoiceValue`).
    pub fn shortcut(&self, cfg: &QuestionnaireConfig, name: &str, value: &str) -> Option<String> {
        let mode = cfg.shortcuts?;
        let def = cfg.def(name)?;
        def.choices
            .iter()
            .filter(|c| !c.disabled)
            .zip(mode.keys())
            .find(|(c, _)| c.value == value)
            .map(|(_, key)| key)
    }

    fn choice_for_shortcut(
        &self,
        cfg: &QuestionnaireConfig,
        def: &ItemDef,
        shortcut: &str,
    ) -> Option<String> {
        let mode = cfg.shortcuts?;
        if def.disabled {
            return None;
        }
        def.choices
            .iter()
            .filter(|c| !c.disabled)
            .zip(mode.keys())
            .find(|(_, key)| key == shortcut)
            .map(|(c, _)| c.value.clone())
    }

    /// The choice's `aria-keyshortcuts`: its shortcut, plus `Enter` while it is checked and enabled.
    pub fn choice_key_shortcuts(
        &self,
        cfg: &QuestionnaireConfig,
        name: &str,
        value: &str,
    ) -> Option<String> {
        let enter = cfg.def(name).is_some_and(|d| d.choice_enabled(value))
            && self.is_checked(cfg, name, value);
        join_shortcuts([
            self.shortcut(cfg, name, value),
            enter.then(|| "Enter".to_string()),
        ])
    }

    /// The freeform input's `aria-keyshortcuts`: `Enter` while it is the selected, filled, enabled answer.
    pub fn input_key_shortcuts(&self, cfg: &QuestionnaireConfig, name: &str) -> Option<String> {
        let enabled = cfg.def(name).is_some_and(ItemDef::input_enabled);
        let enter = enabled && self.input_selected(cfg, name) && self.input_filled(cfg, name);
        join_shortcuts([enter.then(|| "Enter".to_string())])
    }

    /// The item's `aria-keyshortcuts` (shadcn's `keyShortcuts`), for the active item only.
    pub fn item_key_shortcuts(&self, cfg: &QuestionnaireConfig, name: &str) -> Option<String> {
        if !self.is_active(cfg, name) {
            return None;
        }
        let def = cfg.def(name)?;
        let progress = self.progress(cfg);
        let has_answers = !self.enabled_answers(def).is_empty();
        let status = self.status(cfg, name);
        join_shortcuts([
            Some("Meta+Enter Control+Enter".to_string()),
            has_answers.then(|| "ArrowUp ArrowDown".to_string()),
            (!progress.first).then(|| "ArrowLeft".to_string()),
            (!progress.last && status != ItemStatus::Unanswered).then(|| "ArrowRight".to_string()),
        ])
    }

    /// What the native form submits for this item, in render order: checked enabled choices, then the
    /// selected input's text. A skipped item submits nothing.
    pub fn values(&self, cfg: &QuestionnaireConfig, name: &str) -> Vec<String> {
        let Some(def) = cfg.def(name) else {
            return Vec::new();
        };
        let answer = self.answer(def);
        if answer.skipped {
            return Vec::new();
        }
        let mut values: Vec<String> = def
            .choices
            .iter()
            .filter(|c| answer.choices.contains(&c.value))
            .map(|c| c.value.clone())
            .collect();
        if answer.input_selected && has_text(&answer.input) {
            values.push(answer.input.clone());
        }
        values
    }

    /// Where focus goes when the item fails validation (shadcn's `focusInvalid`): the selected, filled
    /// input; else the first enabled answer control; else the fieldset.
    pub fn invalid_focus_target(&self, cfg: &QuestionnaireConfig, name: &str) -> FocusTarget {
        let control = cfg.def(name).and_then(|def| {
            let answers = self.enabled_answers(def);
            if answers.contains(&AnswerKey::Input) && self.key_filled(def, &AnswerKey::Input) {
                Some(AnswerKey::Input)
            } else {
                answers.into_iter().next()
            }
        });
        FocusTarget {
            item: name.to_string(),
            control,
        }
    }

    // ---- keys ----------------------------------------------------------------------------------------

    /// Classifies a keydown on the form (shadcn's `handleKeyDown`), without changing anything.
    pub fn key_down(
        &self,
        cfg: &QuestionnaireConfig,
        key: &KeyInput,
        target: &KeyTarget,
    ) -> KeyOutcome {
        let mut out = KeyOutcome::default();
        let Some(active) = self.active(cfg) else {
            return out;
        };
        let Some(def) = cfg.def(active) else {
            return out;
        };
        if key.composing {
            return out;
        }

        if key.key == KeyName::Enter && (key.meta || key.ctrl) && !key.alt && !key.shift {
            out.prevent_default = true;
            if !key.repeat {
                out.commands.push(Command::Confirm);
            }
            return out;
        }
        if key.meta || key.ctrl || key.alt {
            return out;
        }

        if matches!(key.key, KeyName::ArrowUp | KeyName::ArrowDown) {
            if let Some(commands) =
                self.move_answer_focus(def, target, key.key == KeyName::ArrowDown)
            {
                out.prevent_default = true;
                out.commands = commands;
                return out;
            }
        }

        let text_entry = target_is_text_entry(target);
        if matches!(key.key, KeyName::ArrowLeft | KeyName::ArrowRight)
            && !text_entry
            && !target_is_radio(cfg, target)
        {
            out.prevent_default = true;
            if key.repeat {
                return out;
            }
            if key.key == KeyName::ArrowLeft {
                out.commands.push(Command::Previous);
            } else if self.status(cfg, active) != ItemStatus::Unanswered {
                out.commands.push(Command::Next);
            }
            return out;
        }

        if key.key == KeyName::Enter {
            let KeyTarget::Answer { item, key: answer } = target else {
                return out;
            };
            if item != active || !self.enabled_answers(def).contains(answer) {
                return out;
            }
            out.prevent_default = true;
            if !key.repeat && self.key_filled(def, answer) {
                out.commands.push(Command::Confirm);
            }
            return out;
        }

        if text_entry {
            return out;
        }
        let (Some(mode), KeyName::Character(ch)) = (cfg.shortcuts, &key.key) else {
            return out;
        };
        let Some(shortcut) = mode.shortcut_from_key(ch) else {
            return out;
        };
        let Some(value) = self.choice_for_shortcut(cfg, def, &shortcut) else {
            return out;
        };
        out.prevent_default = true;
        if key.repeat {
            return out;
        }
        out.commands.push(Command::Focus(FocusTarget {
            item: active.to_string(),
            control: Some(AnswerKey::Choice(value.clone())),
        }));
        out.commands.push(if def.multiple {
            Command::ToggleChoice {
                item: active.to_string(),
                value,
            }
        } else {
            Command::SetChoice {
                item: active.to_string(),
                value,
                checked: true,
            }
        });
        out
    }

    /// shadcn's `moveAnswerFocus`: ArrowUp/ArrowDown across an item's answers, wrapping. `None` leaves the
    /// key to the browser (a radio moving to a radio, a filled text input, a number input).
    fn move_answer_focus(
        &self,
        def: &ItemDef,
        target: &KeyTarget,
        down: bool,
    ) -> Option<Vec<Command>> {
        let answers = self.enabled_answers(def);
        if answers.is_empty() {
            return None;
        }
        let current = match target {
            KeyTarget::Answer { item, key } if *item == def.name => {
                answers.iter().position(|a| a == key)
            }
            _ => None,
        };
        if target_is_text_entry(target) {
            let empty_navigable_input = matches!(target, KeyTarget::Answer { item, key: AnswerKey::Input } if *item == def.name)
                && current.is_some()
                && def
                    .input
                    .as_ref()
                    .is_some_and(|i| i.kind.arrows_leave_when_empty())
                && !has_text(&self.answer(def).input);
            if !empty_navigable_input {
                return None;
            }
        }
        if current.is_none() && *target != KeyTarget::Item(def.name.clone()) {
            return None;
        }
        let next = match current {
            None => answers
                .iter()
                .find(|a| self.key_filled(def, a))
                .cloned()
                .unwrap_or_else(|| {
                    if down {
                        answers[0].clone()
                    } else {
                        answers[answers.len() - 1].clone()
                    }
                }),
            Some(i) => {
                let len = answers.len();
                let j = if down {
                    (i + 1) % len
                } else {
                    (i + len - 1) % len
                };
                answers[j].clone()
            }
        };
        if current.map(|i| &answers[i]) == Some(&next) {
            return None;
        }
        let is_radio = |key: &AnswerKey| !def.multiple && matches!(key, AnswerKey::Choice(_));
        if let Some(i) = current {
            if is_radio(&answers[i]) && is_radio(&next) {
                return None;
            }
        }
        let mut commands = vec![Command::Focus(FocusTarget {
            item: def.name.clone(),
            control: Some(next.clone()),
        })];
        if let (true, AnswerKey::Choice(value)) = (is_radio(&next), &next) {
            commands.push(Command::SetChoice {
                item: def.name.clone(),
                value: value.clone(),
                checked: true,
            });
        }
        Some(commands)
    }

    // ---- transitions ---------------------------------------------------------------------------------

    /// Runs a command. `natively_valid` answers whether a filled control passes the browser's constraint
    /// validation; pass `&|_, _| true` when native validation is off. `rendered` says whether an item's
    /// component is mounted (submit validates only rendered items, like shadcn's registrations).
    pub fn run(
        &mut self,
        cfg: &QuestionnaireConfig,
        command: &Command,
        natively_valid: &dyn Fn(&str, &AnswerKey) -> bool,
        rendered: &dyn Fn(&str) -> bool,
    ) -> Vec<QuestionnaireEffect> {
        match command {
            Command::Previous => self.go_previous(cfg),
            Command::Next => self.go_next(cfg, natively_valid),
            Command::Confirm => self.confirm(cfg, natively_valid),
            Command::Skip => self.skip(cfg),
            Command::Submit => self.submit(cfg, natively_valid, rendered),
            Command::Reset => self.reset(cfg),
            Command::Focus(target) => vec![QuestionnaireEffect::Focus(target.clone())],
            Command::SetChoice {
                item,
                value,
                checked,
            } => self.set_choice(cfg, item, value, *checked),
            Command::ToggleChoice { item, value } => {
                let checked = !self.is_checked(cfg, item, value);
                self.set_choice(cfg, item, value, checked)
            }
            Command::SetInput { item, text } => self.set_input(cfg, item, text),
        }
    }

    fn set_choice(
        &mut self,
        cfg: &QuestionnaireConfig,
        name: &str,
        value: &str,
        checked: bool,
    ) -> Vec<QuestionnaireEffect> {
        let Some(def) = cfg.def(name) else {
            return Vec::new();
        };
        if !def.choice_enabled(value) {
            return Vec::new();
        }
        let multiple = def.multiple;
        let answer = self.answer_mut(def);
        let before = answer.clone();
        answer.skipped = false;
        if checked {
            if multiple {
                if !answer.choices.iter().any(|v| v == value) {
                    answer.choices.push(value.to_string());
                }
            } else {
                answer.choices = vec![value.to_string()];
                answer.input_selected = false;
            }
        } else {
            answer.choices.retain(|v| v != value);
        }
        if *answer == before {
            Vec::new()
        } else {
            vec![QuestionnaireEffect::AnswerChange(name.to_string())]
        }
    }

    fn set_input(
        &mut self,
        cfg: &QuestionnaireConfig,
        name: &str,
        text: &str,
    ) -> Vec<QuestionnaireEffect> {
        let Some(def) = cfg.def(name) else {
            return Vec::new();
        };
        if !def.input_enabled() {
            return Vec::new();
        }
        let multiple = def.multiple;
        let answer = self.answer_mut(def);
        let before = answer.clone();
        answer.input = text.to_string();
        answer.skipped = false;
        let filled = has_text(text);
        answer.input_selected = filled;
        if filled && !multiple {
            answer.choices.clear();
        }
        if *answer == before {
            Vec::new()
        } else {
            vec![QuestionnaireEffect::AnswerChange(name.to_string())]
        }
    }

    /// Activates `name` (shadcn's `setItem`). Controlled: report it and remember the focus for when the
    /// caller's `item` arrives. Uncontrolled: switch now and focus.
    fn set_item(
        &mut self,
        cfg: &QuestionnaireConfig,
        name: &str,
        kind: FocusKind,
    ) -> Vec<QuestionnaireEffect> {
        if self.active(cfg) == Some(name) {
            return Vec::new();
        }
        if cfg.controlled_item.is_some() {
            self.pending_focus = Some((name.to_string(), kind));
            return vec![QuestionnaireEffect::ItemChange(name.to_string())];
        }
        self.active = Some(name.to_string());
        vec![
            QuestionnaireEffect::ItemChange(name.to_string()),
            QuestionnaireEffect::Focus(self.focus_for(cfg, name, kind)),
        ]
    }

    fn focus_for(&self, cfg: &QuestionnaireConfig, name: &str, kind: FocusKind) -> FocusTarget {
        match kind {
            FocusKind::Item => FocusTarget {
                item: name.to_string(),
                control: None,
            },
            FocusKind::Invalid => self.invalid_focus_target(cfg, name),
        }
    }

    /// Controlled mode: the caller's `item` changed from `previous` to the config's `controlled_item`. Focus
    /// the new item, or apply the focus the transition that asked for it wanted (an invalid item's control).
    pub fn controlled_item_changed(
        &mut self,
        cfg: &QuestionnaireConfig,
        previous: Option<&str>,
    ) -> Vec<QuestionnaireEffect> {
        let Some(now) = self.active(cfg).map(str::to_string) else {
            return Vec::new();
        };
        let pending = self.pending_focus.take();
        match pending {
            Some((name, kind)) if name == now => {
                vec![QuestionnaireEffect::Focus(self.focus_for(cfg, &now, kind))]
            }
            _ if previous != Some(now.as_str()) => vec![QuestionnaireEffect::Focus(
                self.focus_for(cfg, &now, FocusKind::Item),
            )],
            _ => Vec::new(),
        }
    }

    /// Validates one item, marking validation attempted. `Err` carries the focus effect of the failure.
    fn validate(
        &mut self,
        cfg: &QuestionnaireConfig,
        name: &str,
        natively_valid: &dyn Fn(&str, &AnswerKey) -> bool,
    ) -> Result<(), QuestionnaireEffect> {
        let Some(def) = cfg.def(name) else {
            return Ok(());
        };
        self.answer_mut(def).attempted = true;
        if !self.is_valid(cfg, name) {
            return Err(QuestionnaireEffect::Focus(
                self.invalid_focus_target(cfg, name),
            ));
        }
        let invalid_control = self
            .enabled_answers(def)
            .into_iter()
            .find(|key| self.key_filled(def, key) && !natively_valid(name, key));
        match invalid_control {
            Some(key) => Err(QuestionnaireEffect::ReportValidity(FocusTarget {
                item: name.to_string(),
                control: Some(key),
            })),
            None => Ok(()),
        }
    }

    fn go_previous(&mut self, cfg: &QuestionnaireConfig) -> Vec<QuestionnaireEffect> {
        let Some(index) = self.active(cfg).and_then(|n| cfg.enabled_index(n)) else {
            return Vec::new();
        };
        if index == 0 {
            return Vec::new();
        }
        let previous = cfg.enabled().nth(index - 1).map(|d| d.name.clone());
        previous.map_or_else(Vec::new, |p| self.set_item(cfg, &p, FocusKind::Item))
    }

    fn go_next(
        &mut self,
        cfg: &QuestionnaireConfig,
        natively_valid: &dyn Fn(&str, &AnswerKey) -> bool,
    ) -> Vec<QuestionnaireEffect> {
        let Some(active) = self.active(cfg).map(str::to_string) else {
            return Vec::new();
        };
        let progress = self.progress(cfg);
        if progress.last || progress.total == 0 {
            return Vec::new();
        }
        if let Err(effect) = self.validate(cfg, &active, natively_valid) {
            return vec![effect];
        }
        self.advance(cfg, &active)
    }

    fn advance(&mut self, cfg: &QuestionnaireConfig, from: &str) -> Vec<QuestionnaireEffect> {
        let next = cfg
            .enabled_index(from)
            .and_then(|i| cfg.enabled().nth(i + 1))
            .map(|d| d.name.clone());
        next.map_or_else(Vec::new, |n| self.set_item(cfg, &n, FocusKind::Item))
    }

    fn confirm(
        &mut self,
        cfg: &QuestionnaireConfig,
        natively_valid: &dyn Fn(&str, &AnswerKey) -> bool,
    ) -> Vec<QuestionnaireEffect> {
        let Some(active) = self.active(cfg).map(str::to_string) else {
            return Vec::new();
        };
        if let Err(effect) = self.validate(cfg, &active, natively_valid) {
            return vec![effect];
        }
        if self.progress(cfg).last {
            return vec![QuestionnaireEffect::RequestSubmit];
        }
        self.advance(cfg, &active)
    }

    fn skip(&mut self, cfg: &QuestionnaireConfig) -> Vec<QuestionnaireEffect> {
        let Some(active) = self.active(cfg).map(str::to_string) else {
            return Vec::new();
        };
        let Some(def) = cfg.def(&active) else {
            return Vec::new();
        };
        if def.required {
            return Vec::new();
        }
        let answer = self.answer_mut(def);
        let before = answer.clone();
        answer.choices.clear();
        answer.input_selected = false;
        answer.skipped = true;
        let mut effects = Vec::new();
        if *answer != before {
            effects.push(QuestionnaireEffect::AnswerChange(active.clone()));
        }
        if self.progress(cfg).last {
            effects.push(QuestionnaireEffect::RequestSubmit);
        } else {
            effects.extend(self.advance(cfg, &active));
        }
        effects
    }

    fn submit(
        &mut self,
        cfg: &QuestionnaireConfig,
        natively_valid: &dyn Fn(&str, &AnswerKey) -> bool,
        rendered: &dyn Fn(&str) -> bool,
    ) -> Vec<QuestionnaireEffect> {
        let names: Vec<String> = cfg
            .enabled()
            .filter(|d| rendered(&d.name))
            .map(|d| d.name.clone())
            .collect();
        for name in names {
            if let Err(effect) = self.validate(cfg, &name, natively_valid) {
                if self.active(cfg) == Some(name.as_str()) {
                    return vec![effect];
                }
                let mut effects = self.set_item(cfg, &name, FocusKind::Invalid);
                if let (
                    QuestionnaireEffect::ReportValidity(_),
                    Some(QuestionnaireEffect::Focus(_)),
                ) = (&effect, effects.last())
                {
                    effects.pop();
                    effects.push(effect);
                }
                return effects;
            }
        }
        vec![QuestionnaireEffect::Submit]
    }

    fn reset(&mut self, cfg: &QuestionnaireConfig) -> Vec<QuestionnaireEffect> {
        let before: Vec<(String, Vec<String>)> = cfg
            .items
            .iter()
            .map(|d| (d.name.clone(), self.values(cfg, &d.name)))
            .collect();
        self.answers.clear();
        self.materialize(cfg);
        let mut effects: Vec<QuestionnaireEffect> = before
            .into_iter()
            .filter(|(name, values)| *values != self.values(cfg, name))
            .map(|(name, _)| QuestionnaireEffect::AnswerChange(name))
            .collect();
        if let Some(initial) = cfg.initial_item() {
            effects.extend(self.set_item(cfg, initial, FocusKind::Item));
        }
        effects
    }
}

fn target_is_text_entry(target: &KeyTarget) -> bool {
    matches!(
        target,
        KeyTarget::Other
            | KeyTarget::Answer {
                key: AnswerKey::Input,
                ..
            }
    )
}

fn target_is_radio(cfg: &QuestionnaireConfig, target: &KeyTarget) -> bool {
    match target {
        KeyTarget::Answer {
            item,
            key: AnswerKey::Choice(_),
        } => cfg.def(item).is_some_and(|d| !d.multiple),
        _ => false,
    }
}

fn join_shortcuts<const N: usize>(parts: [Option<String>; N]) -> Option<String> {
    let joined: Vec<String> = parts.into_iter().flatten().collect();
    (!joined.is_empty()).then(|| joined.join(" "))
}

/// Problems with a definition list that the type cannot rule out (shadcn's
/// `getCollectionDefinitionWarnings`): a name used twice, a choice value used twice in one item, a default
/// choice that is not a choice, and a `default_item` that does not name an item that applies.
pub fn definition_warnings(items: &[ItemDef], default_item: Option<&str>) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut names = HashSet::new();
    for item in items {
        if !names.insert(item.name.as_str()) {
            warnings.push(format!(
                "Item name \"{}\" is defined more than once.",
                item.name
            ));
        }
        let mut values = HashSet::new();
        for choice in &item.choices {
            if !values.insert(choice.value.as_str()) {
                warnings.push(format!(
                    "Choice value \"{}\" is defined more than once in item \"{}\".",
                    choice.value, item.name
                ));
            }
        }
        for value in &item.default_choices {
            if item.choice_def(value).is_none() {
                warnings.push(format!(
                    "Default choice \"{}\" of item \"{}\" is not one of its choices.",
                    value, item.name
                ));
            }
        }
    }
    if let Some(default) = default_item {
        if !items.iter().any(|d| d.name == default && !d.disabled) {
            warnings.push(format!(
                "default_item \"{default}\" does not identify an enabled item. The first enabled item will be used instead."
            ));
        }
    }
    warnings
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
