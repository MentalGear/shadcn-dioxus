//! Defines the [`Questionnaire`] component and its parts: a multi-step question flow (shadcn's
//! `@shadcn/react/questionnaire`, 2026-08) with single and multiple choice, a freeform "other" answer, explicit
//! skip, previous/next/submit, required and external validation, controlled navigation, resume with default
//! answers, conditional items, optional letter or number answer shortcuts, and native `<form>` serialization.
//!
//! # Structure
//!
//! All behaviour is the pure state machine in [`model`] (re-exported here): the item list ([`ItemDef`]) is the one
//! source of truth for what each item is, and a [`QuestionnaireState`] holds the active item and the answers.
//! These components only render that state as native form controls and forward DOM events into it:
//!
//! - [`Questionnaire`] is the `<form>`. It owns the state, handles the keyboard (one `keydown` listener, the
//!   decisions are [`QuestionnaireState::key_down`]), validates on submit, and moves focus after a transition.
//! - [`QuestionnaireItem`] is a `fieldset` (inactive ones are `hidden` and `inert`); [`QuestionnaireTitle`] is
//!   its `legend`; [`QuestionnaireDescription`] and [`QuestionnaireError`] are wired to it with
//!   `aria-describedby` (the error only while the item is invalid).
//! - [`QuestionnaireChoice`] wraps a native `input[type=radio|checkbox]` ([`QuestionnaireChoiceInput`]) in a
//!   `label`; [`QuestionnaireInput`] is the freeform answer.
//! - [`QuestionnaireProgress`] is a named `progressbar`; the four navigation buttons hide (and leave the tab
//!   order) when they do not apply.
//!
//! # Native form serialization
//!
//! Every answer is a real form control: a checked radio or checkbox submits its value under the item's name, the
//! freeform input submits its text under the item's name only while it is the selected answer (otherwise it
//! has no `name` and `form=""`, so a draft is never submitted or validated), and a skipped item's choices lose
//! their `name`. `on_submit` receives the `FormEvent`, whose `values()` are the browser's own `FormData`.
//!
//! The form has `method="dialog"` (overridable) and its submit is always `preventDefault`ed: a Dioxus app
//! reads the answers from the event rather than navigating, and a dialog-method form outside a `<dialog>`
//! cannot navigate even before hydration or with scripting off.

use std::collections::HashSet;

use crate::direction::{use_direction, Direction};
use crate::{merge_attributes, use_id_or, use_unique_id};
use dioxus::core::use_drop;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

pub mod model;
pub use model::*;

/// What `on_answer_change` reports: one item's answer after it changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnswerChange {
    /// The item's name.
    pub item: String,
    /// What the native form would submit for it now, in render order.
    pub values: Vec<String>,
    /// Its status now.
    pub status: ItemStatus,
}

/// The root context: the state and the caller's inputs, read together through [`Ctx::with`].
#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<QuestionnaireState>,
    items: ReadSignal<Vec<ItemDef>>,
    controlled: ReadSignal<Option<String>>,
    default_item: ReadSignal<Option<String>>,
    shortcuts: ReadSignal<Option<ShortcutMode>>,
    native_validation: ReadSignal<bool>,
    form_id: Memo<String>,
    /// The element focus is on, as the parts report it (see [`Questionnaire`]'s `onfocusin`).
    focused: CopyValue<KeyTarget>,
    /// A part claimed the current focus change before the form's `focusin` saw it.
    claimed: CopyValue<bool>,
    /// DOM work for after the next render: focus moves, validity reports, a requested submit. Not a
    /// signal: [`Ctx::dom_tick`] is what the effect that drains it subscribes to.
    dom_queue: CopyValue<Vec<QuestionnaireEffect>>,
    /// Bumped whenever DOM work is queued.
    dom_tick: Signal<u64>,
    /// The names of the mounted items (submit validates these, like shadcn's registrations).
    rendered: Signal<Vec<String>>,
    on_item_change: Option<Callback<String>>,
    on_answer_change: Option<Callback<AnswerChange>>,
}

impl Ctx {
    /// Runs `f` with the state and a configuration built from the current props (subscribing the caller).
    fn with<R>(&self, f: impl FnOnce(&QuestionnaireState, &QuestionnaireConfig) -> R) -> R {
        let items = self.items.read();
        let controlled = self.controlled.read();
        let default_item = self.default_item.read();
        let cfg = QuestionnaireConfig {
            items: &items,
            controlled_item: controlled.as_deref(),
            default_item: default_item.as_deref(),
            shortcuts: *self.shortcuts.read(),
        };
        let state = self.state.read();
        f(&state, &cfg)
    }

    /// [`Ctx::with`] without subscribing: for effects and event handlers.
    fn with_peek<R>(&self, f: impl FnOnce(&QuestionnaireState, &QuestionnaireConfig) -> R) -> R {
        let items = self.items.peek();
        let controlled = self.controlled.peek();
        let default_item = self.default_item.peek();
        let cfg = QuestionnaireConfig {
            items: &items,
            controlled_item: controlled.as_deref(),
            default_item: default_item.as_deref(),
            shortcuts: *self.shortcuts.peek(),
        };
        let state = self.state.peek();
        f(&state, &cfg)
    }

    /// Queues DOM work for after the next render.
    fn queue_dom(&self, effects: Vec<QuestionnaireEffect>) {
        if effects.is_empty() {
            return;
        }
        let (mut queue, mut tick) = (self.dom_queue, self.dom_tick);
        queue.write().extend(effects);
        *tick.write() += 1;
    }

    fn claim_focus(&self, target: KeyTarget) {
        let (mut focused, mut claimed) = (self.focused, self.claimed);
        focused.set(target);
        claimed.set(true);
    }

    /// Runs commands, collecting the effects; `invalid_ids` are the controls the browser reports invalid
    /// (native validation), by DOM id. Returns whether validation passed for a submit.
    fn dispatch(&self, commands: &[Command], invalid_ids: &HashSet<String>) -> bool {
        let mut effects = Vec::new();
        {
            let items = self.items.peek();
            let controlled = self.controlled.peek();
            let default_item = self.default_item.peek();
            let cfg = QuestionnaireConfig {
                items: &items,
                controlled_item: controlled.as_deref(),
                default_item: default_item.as_deref(),
                shortcuts: *self.shortcuts.peek(),
            };
            let form_id = self.form_id.peek().clone();
            let rendered = self.rendered.peek().clone();
            let natively_valid = |item: &str, key: &AnswerKey| {
                control_dom_id(&form_id, &cfg, item, key)
                    .map_or(true, |id| !invalid_ids.contains(&id))
            };
            let is_rendered = |item: &str| rendered.iter().any(|n| n == item);
            let mut state_signal = self.state;
            let mut state = state_signal.write();
            for command in commands {
                effects.extend(state.run(&cfg, command, &natively_valid, &is_rendered));
            }
        }
        let mut submit = false;
        let mut dom = Vec::new();
        for effect in effects {
            match effect {
                QuestionnaireEffect::ItemChange(name) => {
                    if let Some(cb) = self.on_item_change {
                        cb.call(name);
                    }
                }
                QuestionnaireEffect::AnswerChange(item) => {
                    if let Some(cb) = self.on_answer_change {
                        let (values, status) =
                            self.with_peek(|s, cfg| (s.values(cfg, &item), s.status(cfg, &item)));
                        cb.call(AnswerChange {
                            item,
                            values,
                            status,
                        });
                    }
                }
                QuestionnaireEffect::Submit => submit = true,
                other => dom.push(other),
            }
        }
        self.queue_dom(dom);
        submit
    }

    /// Runs commands now, or, when native validation is on and a command validates, after asking the
    /// browser which controls are invalid.
    fn execute(self, commands: Vec<Command>) {
        if *self.native_validation.peek() && commands.iter().any(Command::validates) {
            spawn(async move {
                let invalid = invalid_control_ids(self.form_id.peek().clone()).await;
                self.dispatch(&commands, &invalid);
            });
        } else {
            self.dispatch(&commands, &HashSet::new());
        }
    }
}

fn use_ctx() -> Ctx {
    use_context::<Ctx>()
}

/// Asks the browser for the ids of the form's controls that fail constraint validation.
async fn invalid_control_ids(form_id: String) -> HashSet<String> {
    let mut eval = document::eval(
        "const id = await dioxus.recv();
        const form = document.getElementById(id);
        const bad = form ? Array.from(form.elements).filter((e) => e.willValidate && !e.validity.valid).map((e) => e.id) : [];
        dioxus.send(bad);",
    );
    let _ = eval.send(form_id);
    eval.recv::<Vec<String>>()
        .await
        .map(|ids| ids.into_iter().collect())
        .unwrap_or_default()
}

/// Makes `raw` safe and unambiguous inside an `id`: ASCII letters, digits and `-` stay, anything else becomes
/// `_<hex>_`, so two different names never share an id.
fn id_fragment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
        } else {
            out.push_str(&format!("_{:x}_", ch as u32));
        }
    }
    out
}

/// The DOM id of an item's `fieldset`.
fn item_dom_id(form_id: &str, item: &str) -> String {
    format!("{form_id}-{}", id_fragment(item))
}

/// The DOM id of an answer control (`None` for a choice the definition does not have).
fn control_dom_id(
    form_id: &str,
    cfg: &QuestionnaireConfig,
    item: &str,
    key: &AnswerKey,
) -> Option<String> {
    let base = item_dom_id(form_id, item);
    match key {
        AnswerKey::Input => Some(format!("{base}-input")),
        AnswerKey::Choice(value) => {
            let def = cfg.items.iter().find(|d| d.name == item)?;
            let index = def.choices.iter().position(|c| c.value == *value)?;
            Some(format!("{base}-choice-{index}"))
        }
    }
}

fn focus_dom_id(form_id: &str, cfg: &QuestionnaireConfig, target: &FocusTarget) -> Option<String> {
    match &target.control {
        None => Some(item_dom_id(form_id, &target.item)),
        Some(key) => control_dom_id(form_id, cfg, &target.item, key),
    }
}

/// An `aria-keyshortcuts` list with the horizontal pair named as the user presses it: the model speaks LTR,
/// so under RTL its "ArrowLeft" (previous) is the key labelled ArrowRight.
fn rtl_keys(keys: String, direction: Direction) -> String {
    keys.split(' ')
        .map(|key| match key {
            "ArrowLeft" => direction.ltr_equivalent(&Key::ArrowLeft),
            "ArrowRight" => direction.ltr_equivalent(&Key::ArrowRight),
            _ => Key::Character(key.to_string()),
        })
        .map(|key| match key {
            Key::Character(text) => text,
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_empty(children: &Element) -> bool {
    children.as_ref().ok() == VNode::empty().as_ref().ok()
}

/// The model's key, with the horizontal pair mirrored under RTL (`ArrowLeft` is "next" there), so the model
/// only ever reasons in LTR terms.
fn key_input(event: &KeyboardEvent, direction: Direction) -> KeyInput {
    let modifiers = event.modifiers();
    let key = match direction.ltr_equivalent(&event.key()) {
        Key::Enter => KeyName::Enter,
        Key::ArrowUp => KeyName::ArrowUp,
        Key::ArrowDown => KeyName::ArrowDown,
        Key::ArrowLeft => KeyName::ArrowLeft,
        Key::ArrowRight => KeyName::ArrowRight,
        Key::Character(text) => KeyName::Character(text),
        _ => KeyName::Other,
    };
    KeyInput {
        // `Process` is the key value of a keydown that belongs to an IME composition (keyCode 229).
        composing: event.is_composing() || event.key() == Key::Process,
        key,
        meta: modifiers.contains(Modifiers::META),
        ctrl: modifiers.contains(Modifiers::CONTROL),
        alt: modifiers.contains(Modifiers::ALT),
        shift: modifiers.contains(Modifiers::SHIFT),
        repeat: event.is_auto_repeating(),
    }
}

/// The props for [`Questionnaire`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireProps {
    /// The items, in order: the one place each item's behaviour is declared (see [`ItemDef`]). Recompute it
    /// from earlier answers to turn conditional items on and off.
    pub items: ReadSignal<Vec<ItemDef>>,

    /// The item to start on and to return to on reset (resume). Falls back to the first item that applies.
    #[props(default)]
    pub default_item: ReadSignal<Option<String>>,

    /// The controlled active item. When set, navigation calls `on_item_change` and waits for this to change.
    #[props(default)]
    pub item: ReadSignal<Option<String>>,

    /// Called with the item to activate after a navigation.
    #[props(default)]
    pub on_item_change: Option<Callback<String>>,

    /// Called after an item's answer changes.
    #[props(default)]
    pub on_answer_change: Option<Callback<AnswerChange>>,

    /// Letter or number keys that select answers.
    #[props(default)]
    pub shortcuts: ReadSignal<Option<ShortcutMode>>,

    /// Use the browser's constraint validation for filled answers (shadcn's `noValidate={false}`): a selected
    /// `email`/`url`/... input with an invalid value blocks Next and shows the browser's message.
    #[props(default)]
    pub native_validation: ReadSignal<bool>,

    /// Called on a submit that passed validation, with the form event (its `values()` are the answers).
    #[props(default)]
    pub on_submit: Option<Callback<FormEvent>>,

    /// Called after a reset restored the default answers and the initial item.
    #[props(default)]
    pub on_reset: Option<Callback<()>>,

    /// The form's id; generated when absent.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the `form`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The progress, the items and the navigation buttons, in any layout.
    pub children: Element,
}

/// A multi-step questionnaire: a native `<form>` that shows one item at a time. See the module docs.
#[component]
pub fn Questionnaire(props: QuestionnaireProps) -> Element {
    let form_id = use_id_or(use_unique_id(), props.id);
    let items = props.items;
    let controlled = props.item;
    let default_item = props.default_item;
    let shortcuts = props.shortcuts;
    // The nearest `DirectionProvider`'s direction (LTR without one): under RTL the horizontal arrows swap.
    let direction = use_direction(None);
    let state = use_signal(|| {
        let items = items.peek();
        let default_item = default_item.peek();
        QuestionnaireState::new(&QuestionnaireConfig {
            items: &items,
            controlled_item: None,
            default_item: default_item.as_deref(),
            shortcuts: None,
        })
    });
    let ctx = use_context_provider(|| Ctx {
        state,
        items,
        controlled,
        default_item,
        shortcuts,
        native_validation: props.native_validation,
        form_id,
        focused: CopyValue::new(KeyTarget::None),
        claimed: CopyValue::new(false),
        dom_queue: CopyValue::new(Vec::new()),
        dom_tick: Signal::new(0),
        rendered: Signal::new(Vec::new()),
        on_item_change: props.on_item_change,
        on_answer_change: props.on_answer_change,
    });

    // New definitions: record the defaults of items that appeared (live answers keep their values), and
    // report what the type cannot rule out.
    use_effect(move || {
        let defs = items.read();
        let default_item = default_item.read();
        let cfg = QuestionnaireConfig {
            items: &defs,
            controlled_item: None,
            default_item: default_item.as_deref(),
            shortcuts: None,
        };
        let mut state = state;
        if !state.peek().is_materialized(&cfg) {
            state.write().materialize(&cfg);
        }
        for warning in definition_warnings(&defs, default_item.as_deref()) {
            tracing::warn!("[Questionnaire] {warning}");
        }
    });

    // Controlled mode: focus follows the caller's `item`, and an `item` that does not apply is replaced by the
    // first item that does (shadcn's layout effect). Subscribes to the props only; the state is peeked.
    let mut previous_controlled = use_signal(|| controlled.peek().clone());
    use_effect(move || {
        let now = controlled.read().clone();
        let _ = items.read();
        if now.is_none() {
            return;
        }
        let previous = previous_controlled.peek().clone();
        if previous == now {
            if let Some(fallback) = ctx.with_peek(|s, cfg| s.controlled_fallback(cfg)) {
                if let Some(cb) = ctx.on_item_change {
                    cb.call(fallback);
                }
            }
            return;
        }
        previous_controlled.set(now);
        let effects = {
            let items = items.peek();
            let controlled = controlled.peek();
            let default_item = default_item.peek();
            let cfg = QuestionnaireConfig {
                items: &items,
                controlled_item: controlled.as_deref(),
                default_item: default_item.as_deref(),
                shortcuts: None,
            };
            let mut state = state;
            let effects = state
                .write()
                .controlled_item_changed(&cfg, previous.as_deref());
            effects
        };
        ctx.queue_dom(effects);
    });

    // DOM work runs after the render that shows the new state: a hidden item cannot take focus.
    let dom_tick = ctx.dom_tick;
    use_effect(move || {
        let _ = dom_tick.read();
        let mut queue = ctx.dom_queue;
        let effects = std::mem::take(&mut *queue.write());
        if effects.is_empty() {
            return;
        }
        let form = form_id.peek().clone();
        let ops: Vec<(String, String)> = ctx.with_peek(|_, cfg| {
            effects
                .iter()
                .filter_map(|effect| match effect {
                    QuestionnaireEffect::Focus(target) => {
                        focus_dom_id(&form, cfg, target).map(|id| ("focus".to_string(), id))
                    }
                    QuestionnaireEffect::ReportValidity(target) => {
                        focus_dom_id(&form, cfg, target).map(|id| ("report".to_string(), id))
                    }
                    QuestionnaireEffect::RequestSubmit => {
                        Some(("submit".to_string(), form.clone()))
                    }
                    _ => None,
                })
                .collect()
        });
        if ops.is_empty() {
            return;
        }
        let eval = document::eval(
            "const ops = await dioxus.recv();
            for (const [kind, id] of ops) {
                const el = document.getElementById(id);
                if (!el) continue;
                if (kind === 'focus') el.focus();
                else if (kind === 'report') { el.focus(); el.reportValidity(); }
                else if (kind === 'submit' && el.requestSubmit) el.requestSubmit();
            }",
        );
        let _ = eval.send(ops);
    });

    let on_submit = props.on_submit;
    let on_reset = props.on_reset;
    let native = *props.native_validation.read();
    let shortcut_mode = *shortcuts.read();
    let base = attributes!(form {
        id: form_id,
        method: "dialog",
        novalidate: !native,
        "data-shortcuts": shortcut_mode.map(ShortcutMode::as_str),
        onsubmit: move |event: FormEvent| {
            event.prevent_default();
            if *ctx.native_validation.peek() {
                spawn(async move {
                    let invalid = invalid_control_ids(ctx.form_id.peek().clone()).await;
                    if ctx.dispatch(&[Command::Submit], &invalid) {
                        if let Some(cb) = on_submit {
                            cb.call(event);
                        }
                    }
                });
            } else if ctx.dispatch(&[Command::Submit], &HashSet::new()) {
                if let Some(cb) = on_submit {
                    cb.call(event);
                }
            }
        },
        onreset: move |event: FormEvent| {
            // The state is the source of the controls' values: let it restore them, not the browser's
            // defaultValue/defaultChecked bookkeeping.
            event.prevent_default();
            ctx.dispatch(&[Command::Reset], &HashSet::new());
            if let Some(cb) = on_reset {
                cb.call(());
            }
        },
        onkeydown: move |event: KeyboardEvent| {
            let target = ctx.focused.cloned();
            let outcome = ctx.with_peek(|s, cfg| s.key_down(cfg, &key_input(&event, direction), &target));
            if outcome.prevent_default {
                event.prevent_default();
            }
            if !outcome.commands.is_empty() {
                ctx.execute(outcome.commands);
            }
        },
        // A part's own `onfocus` runs first and claims the focus; anything else that takes focus inside the
        // form is a caller's element (treated as text entry).
        onfocusin: move |_| {
            let (mut focused, mut claimed) = (ctx.focused, ctx.claimed);
            if claimed.cloned() {
                claimed.set(false);
            } else {
                focused.set(KeyTarget::Other);
            }
        },
        onfocusout: move |_| {
            let mut focused = ctx.focused;
            focused.set(KeyTarget::None);
        },
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        form { ..merged, {props.children} }
    }
}

/// Read access to a questionnaire's state from inside it: for a custom progress indicator or navigation
/// that reacts to the active item's status. Reads subscribe the calling component.
#[derive(Clone, Copy)]
pub struct QuestionnaireHandle {
    ctx: Ctx,
}

impl QuestionnaireHandle {
    /// The progress values.
    pub fn progress(&self) -> ProgressState {
        self.ctx.with(|s, cfg| s.progress(cfg))
    }

    /// The active item's name.
    pub fn active(&self) -> Option<String> {
        self.ctx.with(|s, cfg| s.active(cfg).map(str::to_string))
    }

    /// An item's status.
    pub fn status(&self, item: &str) -> ItemStatus {
        self.ctx.with(|s, cfg| s.status(cfg, item))
    }

    /// The active item's status (`None` when there is no active item).
    pub fn active_status(&self) -> Option<ItemStatus> {
        self.ctx
            .with(|s, cfg| s.active(cfg).map(|n| s.status(cfg, n)))
    }

    /// What an item would submit now.
    pub fn values(&self, item: &str) -> Vec<String> {
        self.ctx.with(|s, cfg| s.values(cfg, item))
    }
}

/// The enclosing [`Questionnaire`]'s state. Panics outside one.
pub fn use_questionnaire() -> QuestionnaireHandle {
    QuestionnaireHandle { ctx: use_ctx() }
}

/// The props for [`QuestionnaireProgress`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireProgressProps {
    /// Additional attributes for the `div`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Replaces the default "Question 2 of 3" text.
    pub children: Element,
}

/// A named `progressbar` with the active item's position: `aria-valuenow` of `aria-valuemax`, and "Question
/// n of N" as its value text and (unless children replace it) its content. Polite live region.
#[component]
pub fn QuestionnaireProgress(props: QuestionnaireProgressProps) -> Element {
    let ctx = use_ctx();
    let progress = ctx.with(|s, cfg| s.progress(cfg));
    let label = (progress.total > 0)
        .then(|| format!("Question {} of {}", progress.current, progress.total));
    let base = attributes!(div {
        role: "progressbar",
        aria_label: "Questionnaire progress",
        aria_live: "polite",
        aria_valuemin: (progress.total > 0).then_some(1),
        aria_valuemax: (progress.total > 0).then_some(progress.total),
        aria_valuenow: (progress.total > 0).then_some(progress.current),
        aria_valuetext: label.clone(),
        "data-current": progress.current,
        "data-total": progress.total,
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let default_content = is_empty(&props.children);
    rsx! {
        div { ..merged,
            if default_content {
                {label}
            } else {
                {props.children}
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ItemCtx {
    name: Memo<String>,
    descriptions: Signal<usize>,
    errors: Signal<usize>,
}

/// The props for [`QuestionnaireItem`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireItemProps {
    /// The item's name: the [`ItemDef`] it renders.
    pub name: ReadSignal<String>,
    /// Additional attributes for the `fieldset`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The title, description, choices, input and error.
    pub children: Element,
}

/// One question: a `fieldset` that is visible only while active (inactive items are `hidden` and `inert`).
/// It takes focus (`tabindex="-1"`) after a navigation.
#[component]
pub fn QuestionnaireItem(props: QuestionnaireItemProps) -> Element {
    let ctx = use_ctx();
    let name_signal = props.name;
    let name = use_memo(move || name_signal.cloned());
    let descriptions = use_signal(|| 0usize);
    let errors = use_signal(|| 0usize);
    use_context_provider(|| ItemCtx {
        name,
        descriptions,
        errors,
    });
    // Registered while mounted: submit validates the rendered items.
    let registered = use_hook(move || {
        let current = name.peek().clone();
        let mut rendered = ctx.rendered;
        rendered.write().push(current.clone());
        current
    });
    use_drop(move || {
        let mut rendered = ctx.rendered;
        let mut list = rendered.write();
        if let Some(i) = list.iter().position(|n| *n == registered) {
            list.remove(i);
        }
    });

    let item = name();
    let direction = use_direction(None);
    let form_id = ctx.form_id.read().clone();
    let (known, active, disabled, invalid, status, required, multiple, key_shortcuts) =
        ctx.with(|s, cfg| {
            let def = cfg.items.iter().find(|d| d.name == item);
            (
                def.is_some(),
                s.is_active(cfg, &item),
                def.is_some_and(|d| d.disabled),
                s.is_invalid(cfg, &item),
                s.status(cfg, &item),
                def.is_some_and(|d| d.required),
                def.is_some_and(|d| d.multiple),
                s.item_key_shortcuts(cfg, &item)
                    .map(|keys| rtl_keys(keys, direction)),
            )
        });
    if !known {
        tracing::warn!("[Questionnaire] QuestionnaireItem \"{item}\" has no ItemDef in `items`.");
    }
    let id = item_dom_id(&form_id, &item);
    let mut described = Vec::new();
    if descriptions() > 0 {
        described.push(format!("{id}-description"));
    }
    if invalid && errors() > 0 {
        described.push(format!("{id}-error"));
    }
    let focus_name = item.clone();
    let base = attributes!(fieldset {
        id: id.clone(),
        tabindex: -1,
        disabled: disabled,
        hidden: !active,
        inert: (!active).then_some("true"),
        aria_describedby: (!described.is_empty()).then(|| described.join(" ")),
        aria_invalid: invalid.then_some("true"),
        aria_keyshortcuts: key_shortcuts,
        "data-active": active.then_some("true"),
        "data-status": status.as_str(),
        "data-invalid": invalid.then_some("true"),
        "data-required": required.then_some("true"),
        "data-multiple": multiple.then_some("true"),
        "data-disabled": disabled.then_some("true"),
        onfocus: move |_| ctx.claim_focus(KeyTarget::Item(focus_name.clone())),
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        fieldset { ..merged, {props.children} }
    }
}

fn use_item_ctx() -> ItemCtx {
    use_context::<ItemCtx>()
}

/// The props for the text parts ([`QuestionnaireTitle`], [`QuestionnaireDescription`], [`QuestionnaireChoices`],
/// [`QuestionnaireChoiceLabel`]).
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnairePartProps {
    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The content.
    pub children: Element,
}

/// The item's question: its `legend`, which names the `fieldset`.
#[component]
pub fn QuestionnaireTitle(props: QuestionnairePartProps) -> Element {
    rsx! {
        legend { ..props.attributes, {props.children} }
    }
}

/// The item's description, joined to the `fieldset`'s `aria-describedby` while it is mounted.
#[component]
pub fn QuestionnaireDescription(props: QuestionnairePartProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    use_hook(move || {
        let mut descriptions = item.descriptions;
        *descriptions.write() += 1;
    });
    use_drop(move || {
        let mut descriptions = item.descriptions;
        let mut count = descriptions.write();
        *count = count.saturating_sub(1);
    });
    let id = format!(
        "{}-description",
        item_dom_id(&ctx.form_id.read(), &item.name.read())
    );
    let merged = merge_attributes(vec![attributes!(p { id: id }), props.attributes]);
    rsx! {
        p { ..merged, {props.children} }
    }
}

/// The container of the item's answers (`data-shortcuts` names the shortcut mode).
#[component]
pub fn QuestionnaireChoices(props: QuestionnairePartProps) -> Element {
    let ctx = use_ctx();
    let mode = *ctx.shortcuts.read();
    let merged = merge_attributes(vec![
        attributes!(div { "data-shortcuts": mode.map(ShortcutMode::as_str) }),
        props.attributes,
    ]);
    rsx! {
        div { ..merged, {props.children} }
    }
}

/// What a [`QuestionnaireChoice`] tells its parts.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ChoiceState {
    item: String,
    value: String,
    dom_id: Option<String>,
    checked: bool,
    disabled: bool,
    invalid: bool,
    multiple: bool,
    skipped: bool,
    required: bool,
    shortcut: Option<String>,
    key_shortcuts: Option<String>,
}

#[derive(Clone, Copy)]
struct ChoiceCtx {
    value: Memo<String>,
}

fn choice_state(ctx: &Ctx, item: &str, value: &str) -> ChoiceState {
    let form_id = ctx.form_id.read().clone();
    ctx.with(|s, cfg| {
        let def = cfg.items.iter().find(|d| d.name == item);
        let choice = def.and_then(|d| d.choices.iter().find(|c| c.value == value));
        if def.is_some() && choice.is_none() {
            tracing::warn!("[Questionnaire] Choice \"{value}\" is not in item \"{item}\"'s ItemDef; it cannot be selected.");
        }
        let multiple = def.is_some_and(|d| d.multiple);
        ChoiceState {
            item: item.to_string(),
            value: value.to_string(),
            dom_id: control_dom_id(&form_id, cfg, item, &AnswerKey::Choice(value.to_string())),
            checked: s.is_checked(cfg, item, value),
            disabled: def.map_or(true, |d| d.disabled) || choice.map_or(true, |c| c.disabled),
            invalid: s.is_invalid(cfg, item),
            multiple,
            skipped: s.status(cfg, item) == ItemStatus::Skipped,
            required: def.is_some_and(|d| d.required && !multiple && d.input.is_none()),
            shortcut: s.shortcut(cfg, item, value),
            key_shortcuts: s.choice_key_shortcuts(cfg, item, value),
        }
    })
}

/// The props for [`QuestionnaireChoice`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireChoiceProps {
    /// The choice's value: one of its item's [`ChoiceDef`]s.
    pub value: ReadSignal<String>,
    /// Additional attributes for the `label`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// A [`QuestionnaireChoiceInput`] and the visible content.
    pub children: Element,
}

/// One fixed answer: a `label` around a native radio or checkbox, so the whole row selects it. Carries
/// `data-checked`/`data-unchecked`, `data-disabled`, `data-invalid`, `data-type` and `data-shortcut`.
#[component]
pub fn QuestionnaireChoice(props: QuestionnaireChoiceProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    let value_signal = props.value;
    let value = use_memo(move || value_signal.cloned());
    use_context_provider(|| ChoiceCtx { value });
    let state = choice_state(&ctx, &item.name.read(), &value.read());
    let base = attributes!(label {
        "data-checked": state.checked.then_some("true"),
        "data-unchecked": (!state.checked).then_some("true"),
        "data-disabled": state.disabled.then_some("true"),
        "data-invalid": state.invalid.then_some("true"),
        "data-type": if state.multiple { "checkbox" } else { "radio" },
        "data-shortcut": state.shortcut.clone(),
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        label { ..merged, {props.children} }
    }
}

/// The props for [`QuestionnaireChoiceInput`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireChoiceInputProps {
    /// Additional attributes for the `input`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The native `input[type=radio]` (single) or `input[type=checkbox]` (multiple) of a [`QuestionnaireChoice`].
#[component]
pub fn QuestionnaireChoiceInput(props: QuestionnaireChoiceInputProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    let choice = use_context::<ChoiceCtx>();
    let state = choice_state(&ctx, &item.name.read(), &choice.value.read());
    let (focus_item, focus_value) = (state.item.clone(), state.value.clone());
    let (change_item, change_value, multiple) =
        (state.item.clone(), state.value.clone(), state.multiple);
    let base = attributes!(input {
        r#type: if state.multiple { "checkbox" } else { "radio" },
        id: state.dom_id.clone(),
        name: (!state.skipped).then(|| state.item.clone()),
        value: state.value.clone(),
        checked: state.checked,
        disabled: state.disabled,
        required: state.required,
        aria_invalid: state.invalid.then_some("true"),
        aria_keyshortcuts: state.key_shortcuts.clone(),
        "data-checked": state.checked.then_some("true"),
        "data-unchecked": (!state.checked).then_some("true"),
        onfocus: move |_| {
            ctx.claim_focus(KeyTarget::Answer {
                item: focus_item.clone(),
                key: AnswerKey::Choice(focus_value.clone()),
            })
        },
        onchange: move |event: FormEvent| {
            // A radio's change only fires when it becomes checked; a checkbox reports its new state.
            let checked = !multiple || event.checked();
            ctx.execute(vec![Command::SetChoice {
                item: change_item.clone(),
                value: change_value.clone(),
                checked,
            }]);
        },
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        input { ..merged }
    }
}

/// The visible content of a [`QuestionnaireChoice`].
#[component]
pub fn QuestionnaireChoiceLabel(props: QuestionnairePartProps) -> Element {
    rsx! {
        span { ..props.attributes, {props.children} }
    }
}

/// The props for [`QuestionnaireChoiceShortcut`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireChoiceShortcutProps {
    /// Additional attributes for the `span`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Replaces the default content (the shortcut key).
    pub children: Element,
}

/// The choice's shortcut key, `aria-hidden` (the input's `aria-keyshortcuts` announces it); `hidden` when the
/// choice has none.
#[component]
pub fn QuestionnaireChoiceShortcut(props: QuestionnaireChoiceShortcutProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    let choice = use_context::<ChoiceCtx>();
    let state = choice_state(&ctx, &item.name.read(), &choice.value.read());
    let base = attributes!(span {
        aria_hidden: "true",
        hidden: state.shortcut.is_none(),
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let default_content = is_empty(&props.children);
    rsx! {
        span { ..merged,
            if default_content {
                {state.shortcut}
            } else {
                {props.children}
            }
        }
    }
}

/// The props for [`QuestionnaireInput`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireInputProps {
    /// Additional attributes for the `input` (give it an accessible name: `aria-label`, `aria-labelledby` or
    /// a visible label; a placeholder is not a label).
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    pub attributes: Vec<Attribute>,
}

/// The freeform answer. Typing selects it (and, in a single-choice item, unchecks the radios); choosing a
/// radio keeps the text as an unsubmitted draft. Carries `data-filled`/`data-empty`.
#[component]
pub fn QuestionnaireInput(props: QuestionnaireInputProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    let name = item.name.read().clone();
    let form_id = ctx.form_id.read().clone();
    let (id, kind, disabled, selected, filled, text, invalid, key_shortcuts) = ctx.with(|s, cfg| {
        let def = cfg.items.iter().find(|d| d.name == name);
        if def.is_some_and(|d| d.input.is_none()) {
            tracing::warn!("[Questionnaire] Item \"{name}\" renders a QuestionnaireInput but its ItemDef has no freeform input.");
        }
        let input = def.and_then(|d| d.input.as_ref());
        (
            control_dom_id(&form_id, cfg, &name, &AnswerKey::Input),
            input.map_or(InputKind::Text, |i| i.kind),
            def.map_or(true, |d| d.disabled) || input.map_or(true, |i| i.disabled),
            s.input_selected(cfg, &name),
            s.input_filled(cfg, &name),
            s.input_text(cfg, &name),
            s.is_invalid(cfg, &name),
            s.input_key_shortcuts(cfg, &name),
        )
    });
    let (focus_item, input_item) = (name.clone(), name.clone());
    let base = attributes!(input {
        r#type: kind.as_str(),
        id: id,
        name: selected.then(|| name.clone()),
        // Not the answer: outside the form entirely, so it is neither submitted nor validated.
        form: (!selected).then_some(""),
        value: text,
        disabled: disabled,
        aria_invalid: invalid.then_some("true"),
        aria_keyshortcuts: key_shortcuts,
        "data-filled": filled.then_some("true"),
        "data-empty": (!filled).then_some("true"),
        onfocus: move |_| {
            ctx.claim_focus(KeyTarget::Answer {
                item: focus_item.clone(),
                key: AnswerKey::Input,
            })
        },
        oninput: move |event: FormEvent| {
            ctx.execute(vec![Command::SetInput {
                item: input_item.clone(),
                text: event.value(),
            }]);
        },
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        input { ..merged }
    }
}

/// The props for [`QuestionnaireError`].
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireErrorProps {
    /// Additional attributes for the `p`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Replaces the default message ("Choose an answer to continue." for a required item, "Choose an answer
    /// or skip this question." otherwise), e.g. with an external validation message.
    pub children: Element,
}

/// The item's error: `hidden` until the item is invalid, then `role="alert"` and part of the `fieldset`'s
/// `aria-describedby`.
#[component]
pub fn QuestionnaireError(props: QuestionnaireErrorProps) -> Element {
    let ctx = use_ctx();
    let item = use_item_ctx();
    use_hook(move || {
        let mut errors = item.errors;
        *errors.write() += 1;
    });
    use_drop(move || {
        let mut errors = item.errors;
        let mut count = errors.write();
        *count = count.saturating_sub(1);
    });
    let name = item.name.read().clone();
    let (invalid, required) = ctx.with(|s, cfg| {
        (
            s.is_invalid(cfg, &name),
            cfg.items.iter().any(|d| d.name == name && d.required),
        )
    });
    let id = format!("{}-error", item_dom_id(&ctx.form_id.read(), &name));
    let base = attributes!(p {
        id: id,
        hidden: !invalid,
        role: invalid.then_some("alert"),
        "data-invalid": invalid.then_some("true"),
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let message = if required {
        "Choose an answer to continue."
    } else {
        "Choose an answer or skip this question."
    };
    let default_content = is_empty(&props.children);
    rsx! {
        p { ..merged,
            if default_content {
                {message}
            } else {
                {props.children}
            }
        }
    }
}

/// Which navigation button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestionnaireAction {
    /// Back one item.
    Previous,
    /// Skip the active optional item.
    Skip,
    /// Validate and go forward.
    Next,
    /// Submit the form (on the last item).
    Submit,
}

impl QuestionnaireAction {
    /// The button's default text.
    pub fn default_label(self) -> &'static str {
        match self {
            QuestionnaireAction::Previous => "Previous",
            QuestionnaireAction::Skip => "Skip",
            QuestionnaireAction::Next => "Next",
            QuestionnaireAction::Submit => "Submit",
        }
    }
}

/// A navigation button's state and attributes, from [`use_questionnaire_action`].
#[derive(Clone, PartialEq)]
pub struct QuestionnaireActionState {
    /// Whether the button applies now (Previous after the first item, Skip on an optional item, Next before
    /// the last, Submit on the last).
    pub visible: bool,
    /// The attributes to put on the `button`: `type`, `hidden`/`inert`/`aria-hidden` and `tabindex="-1"`
    /// while it does not apply, `aria-keyshortcuts="Enter"` on Next and Submit, `data-visible`/`data-hidden`,
    /// `data-status`, and the click and focus handlers.
    pub attributes: Vec<Attribute>,
}

/// The state and attributes of a navigation button, for rendering it as your own button component (the
/// styled questionnaire renders its `Button` with them). `disabled` opts into a disabled button.
pub fn use_questionnaire_action(
    action: QuestionnaireAction,
    disabled: bool,
) -> QuestionnaireActionState {
    let ctx = use_ctx();
    let (progress, active_required, status) = ctx.with(|s, cfg| {
        (
            s.progress(cfg),
            s.active_required(cfg),
            s.active(cfg).map(|n| s.status(cfg, n)),
        )
    });
    let visible = match action {
        QuestionnaireAction::Previous => progress.total > 1 && !progress.first,
        QuestionnaireAction::Skip => active_required == Some(false),
        QuestionnaireAction::Next => progress.total > 1 && !progress.last,
        QuestionnaireAction::Submit => progress.total > 0 && progress.last,
    };
    let shortcut = matches!(
        action,
        QuestionnaireAction::Next | QuestionnaireAction::Submit
    ) && visible
        && !disabled;
    let command = match action {
        QuestionnaireAction::Previous => Some(Command::Previous),
        QuestionnaireAction::Skip => Some(Command::Skip),
        QuestionnaireAction::Next => Some(Command::Next),
        // A real submit button: the browser fires the form's submit event.
        QuestionnaireAction::Submit => None,
    };
    let attributes = attributes!(button {
        r#type: if action == QuestionnaireAction::Submit { "submit" } else { "button" },
        disabled: disabled,
        hidden: !visible,
        inert: (!visible).then_some("true"),
        aria_hidden: (!visible).then_some("true"),
        tabindex: (!visible).then_some(-1),
        aria_keyshortcuts: shortcut.then_some("Enter"),
        "data-visible": visible.then_some("true"),
        "data-hidden": (!visible).then_some("true"),
        "data-status": status.map(ItemStatus::as_str),
        onclick: move |_| {
            if let Some(command) = command.clone() {
                ctx.execute(vec![command]);
            }
        },
        onfocus: move |_| ctx.claim_focus(KeyTarget::Action),
    });
    QuestionnaireActionState {
        visible,
        attributes,
    }
}

/// The props for the navigation buttons.
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireActionProps {
    /// Disable the button (shadcn's opt-in disabled navigation).
    #[props(default)]
    pub disabled: bool,
    /// Additional attributes for the `button`.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    /// Replaces the default text.
    pub children: Element,
}

fn action_button(action: QuestionnaireAction, props: QuestionnaireActionProps) -> Element {
    let state = use_questionnaire_action(action, props.disabled);
    let merged = merge_attributes(vec![state.attributes, props.attributes]);
    let default_content = is_empty(&props.children);
    rsx! {
        button { ..merged,
            if default_content {
                {action.default_label()}
            } else {
                {props.children}
            }
        }
    }
}

/// Goes back one item; hidden on the first.
#[component]
pub fn QuestionnairePrevious(props: QuestionnaireActionProps) -> Element {
    action_button(QuestionnaireAction::Previous, props)
}

/// Skips the active optional item (then goes on, or submits on the last); hidden on a required item.
#[component]
pub fn QuestionnaireSkip(props: QuestionnaireActionProps) -> Element {
    action_button(QuestionnaireAction::Skip, props)
}

/// Validates the active item and goes to the next; hidden on the last.
#[component]
pub fn QuestionnaireNext(props: QuestionnaireActionProps) -> Element {
    action_button(QuestionnaireAction::Next, props)
}

/// Submits the form (every rendered item is validated first); shown only on the last item.
#[component]
pub fn QuestionnaireSubmit(props: QuestionnaireActionProps) -> Element {
    action_button(QuestionnaireAction::Submit, props)
}

#[cfg(test)]
mod ssr_tests {
    use super::*;
    use crate::test_support::find_elements;

    #[test]
    fn rtl_mirrors_only_the_horizontal_pair_of_aria_keyshortcuts() {
        let keys = "Meta+Enter Control+Enter ArrowUp ArrowDown ArrowLeft ArrowRight".to_string();
        assert_eq!(rtl_keys(keys.clone(), Direction::Ltr), keys);
        assert_eq!(
            rtl_keys(keys, Direction::Rtl),
            "Meta+Enter Control+Enter ArrowUp ArrowDown ArrowRight ArrowLeft"
        );
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn items() -> Vec<ItemDef> {
        vec![
            ItemDef::new("direction")
                .required()
                .choices(["a", "b"])
                .freeform(),
            ItemDef::new("signals")
                .multiple()
                .choices(["x", "y"])
                .default_choice("y"),
        ]
    }

    #[test]
    fn server_render_has_the_complete_initial_state() {
        fn app() -> Element {
            rsx! {
                Questionnaire { items: items(), shortcuts: Some(ShortcutMode::Letters), id: Some("q".to_string()),
                    QuestionnaireProgress {}
                    for def in items() {
                        QuestionnaireItem { key: "{def.name}", name: def.name.clone(),
                            QuestionnaireTitle { "{def.name}" }
                            QuestionnaireChoices {
                                for c in def.choices.clone() {
                                    QuestionnaireChoice { key: "{c.value}", value: c.value.clone(),
                                        QuestionnaireChoiceInput {}
                                        QuestionnaireChoiceShortcut {}
                                    }
                                }
                                if def.input.is_some() {
                                    QuestionnaireInput { aria_label: "Other" }
                                }
                            }
                            QuestionnaireError {}
                        }
                    }
                    QuestionnairePrevious {}
                    QuestionnaireSkip {}
                    QuestionnaireNext {}
                    QuestionnaireSubmit { "Save" }
                }
            }
        }
        let html = render(app);
        let els = find_elements(&html, |_| true);
        let find = |pred: &dyn Fn(&crate::test_support::Element) -> bool| {
            els.iter().find(|e| pred(e)).cloned()
        };

        let form = find(&|e| e.name == "form").expect("form");
        assert_eq!(form.attrs.get("method").map(String::as_str), Some("dialog"));
        assert_eq!(
            form.attrs.get("data-shortcuts").map(String::as_str),
            Some("letters")
        );

        let progress = find(&|e| e.attrs.get("role").map(String::as_str) == Some("progressbar"))
            .expect("progressbar");
        assert_eq!(
            progress.attrs.get("aria-valuenow").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            progress.attrs.get("aria-valuemax").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            progress.attrs.get("aria-valuetext").map(String::as_str),
            Some("Question 1 of 2")
        );

        let first = find(&|e| e.attrs.get("id").map(String::as_str) == Some("q-direction"))
            .expect("first item");
        assert_eq!(first.name, "fieldset");
        assert!(!first.attrs.contains_key("hidden"));
        assert!(!first.attrs.contains_key("inert"));
        let second = find(&|e| e.attrs.get("id").map(String::as_str) == Some("q-signals"))
            .expect("second item");
        assert!(second.attrs.contains_key("hidden"));
        assert!(second.attrs.contains_key("inert"));

        let radio =
            find(&|e| e.attrs.get("id").map(String::as_str) == Some("q-direction-choice-0"))
                .expect("radio");
        assert_eq!(radio.attrs.get("type").map(String::as_str), Some("radio"));
        assert_eq!(
            radio.attrs.get("name").map(String::as_str),
            Some("direction")
        );
        assert_eq!(
            radio.attrs.get("aria-keyshortcuts").map(String::as_str),
            Some("A")
        );
        let checkbox =
            find(&|e| e.attrs.get("id").map(String::as_str) == Some("q-signals-choice-1"))
                .expect("checkbox");
        assert_eq!(
            checkbox.attrs.get("type").map(String::as_str),
            Some("checkbox")
        );
        assert!(
            checkbox.attrs.contains_key("checked"),
            "a default answer is server-rendered"
        );

        let freeform =
            find(&|e| e.attrs.get("id").map(String::as_str) == Some("q-direction-input"))
                .expect("input");
        assert!(
            !freeform.attrs.contains_key("name"),
            "an empty draft has no name"
        );
        assert_eq!(freeform.attrs.get("form").map(String::as_str), Some(""));

        let buttons: Vec<_> = els.iter().filter(|e| e.name == "button").collect();
        assert_eq!(buttons.len(), 4);
        let hidden: Vec<bool> = buttons
            .iter()
            .map(|b| b.attrs.contains_key("hidden"))
            .collect();
        assert_eq!(
            hidden,
            vec![true, true, false, true],
            "first required item: only Next"
        );
        assert!(html.contains(">Save</button>"));
        assert!(html.contains(">Next</button>"));
    }
}
