use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::js_listener::{use_js_listeners, JsListeners};
use dioxus_primitives::merge_attributes;
use dioxus_primitives::questionnaire::{
    self as primitive, use_questionnaire_action, QuestionnaireAction, QuestionnaireChoiceProps,
    QuestionnaireErrorProps, QuestionnaireInputProps, QuestionnaireItemProps,
    QuestionnairePartProps, QuestionnaireProgressProps,
};
// The model types a caller builds `items` from and reads answers with; not all are used by the demos.
#[allow(unused_imports)]
pub use dioxus_primitives::questionnaire::{
    use_questionnaire, AnswerChange, ChoiceDef, InputDef, InputKind, ItemDef, ItemStatus,
    ProgressState, QuestionnaireHandle, ShortcutMode,
};

/// The props of the styled [`Questionnaire`]: the primitive's, plus `animate_height`.
#[derive(Props, Clone, PartialEq)]
pub struct StyledQuestionnaireProps {
    /// The items, in order: the one place each item's behaviour is declared (see [`ItemDef`]).
    pub items: ReadSignal<Vec<ItemDef>>,
    /// The item to start on and to return to on reset (resume).
    #[props(default)]
    pub default_item: ReadSignal<Option<String>>,
    /// The controlled active item.
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
    /// Use the browser's constraint validation for filled answers.
    #[props(default)]
    pub native_validation: ReadSignal<bool>,
    /// Called on a submit that passed validation.
    #[props(default)]
    pub on_submit: Option<Callback<FormEvent>>,
    /// Called after a reset restored the default answers and the initial item.
    #[props(default)]
    pub on_reset: Option<Callback<()>>,
    /// Glide the height between items with a CSS transition instead of snapping (default `true`).
    /// Skipped under `prefers-reduced-motion: reduce`. `false` keeps the snap.
    #[props(default = true)]
    pub animate_height: bool,
    /// The form's id; generated when absent.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes for the `form`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The progress, the items and the navigation buttons, in any layout.
    pub children: Element,
}

/// Height animation. Items swap with `hidden`, so the form jumps from one item's natural height to the
/// next. `height: auto` cannot transition (`interpolate-size` is Chromium-only), and `grid-template-rows:
/// 0fr/1fr` animates one collapsing child, not a swap. So a ResizeObserver (which runs after layout,
/// before paint: no flash) pins the old height inline, then sets the new one in pixels and lets the CSS
/// `transition: height` in `style.css` run; the pins are cleared when it ends. Works in every engine.
const HEIGHT_JS: &str = r#"
const id = await dioxus.recv();
const el = document.getElementById(id);
if (!el) return;
const reduce = matchMedia('(prefers-reduced-motion: reduce)');
const h = () => el.getBoundingClientRect().height;
let last = h();
let busy = false;
let timer = 0;
cleanup(() => { clearTimeout(timer); el.style.height = ''; el.style.overflow = ''; });
const ro = new ResizeObserver(() => {
  if (busy) return;
  const next = h();
  const from = last;
  last = next;
  if (reduce.matches || Math.abs(next - from) < 1 || !el.isConnected) return;
  const ms = (() => {
    const d = getComputedStyle(el).transitionDuration.split(',')[0].trim();
    return d.endsWith('ms') ? parseFloat(d) : parseFloat(d) * 1000;
  })();
  if (!(ms > 1)) return;
  busy = true;
  el.style.overflow = 'hidden';
  el.style.height = from + 'px';
  void el.offsetHeight;
  el.style.height = next + 'px';
  timer = setTimeout(() => {
    el.style.height = '';
    el.style.overflow = '';
    busy = false;
    last = h();
  }, ms + 60);
});
ro.observe(el);
cleanup(() => ro.disconnect());
"#;

/// Installs the height animation for the enclosing form (a child, so it can read the form's id).
#[component]
fn HeightAnimation() -> Element {
    let q = use_questionnaire();
    use_js_listeners(move || Some(JsListeners::install(HEIGHT_JS).arg(q.form_id())));
    rsx! {}
}

/// A multi-step questionnaire: a native `<form>` that shows one item at a time (Nova's `cn-questionnaire`,
/// `flex flex-col gap-4`). `items` declares every item's behaviour; the parts below render them.
#[component]
pub fn Questionnaire(props: StyledQuestionnaireProps) -> Element {
    let animate = props.animate_height;
    let base = attributes!(div { class: "dx-questionnaire", "data-slot": "questionnaire" });
    let merged = merge_attributes(vec![
        base,
        attributes!(div { "data-animate-height": animate.then_some("true") }),
        props.attributes,
    ]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/questionnaire/style.css") }
        primitive::Questionnaire {
            items: props.items,
            default_item: props.default_item,
            item: props.item,
            on_item_change: props.on_item_change,
            on_answer_change: props.on_answer_change,
            shortcuts: props.shortcuts,
            native_validation: props.native_validation,
            on_submit: props.on_submit,
            on_reset: props.on_reset,
            id: props.id,
            attributes: merged,
            if animate {
                HeightAnimation {}
            }
            {props.children}
        }
    }
}

/// "Question 2 of 3" in muted, tabular figures: the named `progressbar`. Children replace the text (read
/// the numbers with [`use_questionnaire`] for a custom indicator).
#[component]
pub fn QuestionnaireProgress(props: QuestionnaireProgressProps) -> Element {
    let base = attributes!(div { class: "dx-questionnaire-progress", "data-slot": "questionnaire-progress" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        // `children:` hands the Element through as is, so an empty one still reads as "use the default text".
        primitive::QuestionnaireProgress { attributes: merged, children: props.children }
    }
}

/// One question (a `fieldset`, shown only while active).
#[component]
pub fn QuestionnaireItem(props: QuestionnaireItemProps) -> Element {
    let base =
        attributes!(fieldset { class: "dx-questionnaire-item", "data-slot": "questionnaire-item" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireItem { name: props.name, attributes: merged, {props.children} }
    }
}

/// The question (the item's `legend`).
#[component]
pub fn QuestionnaireTitle(props: QuestionnairePartProps) -> Element {
    let base =
        attributes!(legend { class: "dx-questionnaire-title", "data-slot": "questionnaire-title" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireTitle { attributes: merged, {props.children} }
    }
}

/// Supporting text under the question, announced with it.
#[component]
pub fn QuestionnaireDescription(props: QuestionnairePartProps) -> Element {
    let base = attributes!(p { class: "dx-questionnaire-description", "data-slot": "questionnaire-description" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireDescription { attributes: merged, {props.children} }
    }
}

/// The list of answers.
#[component]
pub fn QuestionnaireChoices(props: QuestionnairePartProps) -> Element {
    let base = attributes!(div { class: "dx-questionnaire-choices", "data-slot": "questionnaire-choices" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireChoices { attributes: merged, {props.children} }
    }
}

/// One fixed answer: a bordered row with a radio or checkbox indicator, its content, and its shortcut key.
/// The native input covers the row, so a click anywhere on it selects the answer.
#[component]
pub fn QuestionnaireChoice(props: QuestionnaireChoiceProps) -> Element {
    let base = attributes!(label { class: "dx-questionnaire-choice", "data-slot": "questionnaire-choice" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireChoice { value: props.value, attributes: merged,
            primitive::QuestionnaireChoiceInput {
                class: "dx-questionnaire-choice-input",
                "data-slot": "questionnaire-choice-input",
            }
            span {
                class: "dx-questionnaire-choice-indicator",
                "data-slot": "questionnaire-choice-indicator",
                aria_hidden: "true",
                span { class: "dx-questionnaire-choice-indicator-dot" }
                svg {
                    class: "dx-questionnaire-choice-indicator-check",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M20 6 9 17l-5-5" }
                }
            }
            primitive::QuestionnaireChoiceLabel {
                class: "dx-questionnaire-choice-label",
                "data-slot": "questionnaire-choice-label",
                {props.children}
            }
            primitive::QuestionnaireChoiceShortcut {
                class: "dx-questionnaire-choice-shortcut",
                "data-slot": "questionnaire-choice-shortcut",
            }
        }
    }
}

/// A muted second line inside a [`QuestionnaireChoice`].
#[component]
pub fn QuestionnaireChoiceDescription(
    /// Additional attributes for the element.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    /// The content.
    children: Element,
) -> Element {
    let base = attributes!(span {
        class: "dx-questionnaire-choice-description",
        "data-slot": "questionnaire-choice-description",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        span { ..merged, {children} }
    }
}

/// The freeform "other" answer. Give it an accessible name (`aria_label` or a visible label).
#[component]
pub fn QuestionnaireInput(props: QuestionnaireInputProps) -> Element {
    let base =
        attributes!(input { class: "dx-questionnaire-input", "data-slot": "questionnaire-input" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        div {
            class: "dx-questionnaire-input-wrapper",
            "data-slot": "questionnaire-input-wrapper",
            primitive::QuestionnaireInput { attributes: merged }
        }
    }
}

/// The item's error message, shown while the item is invalid.
#[component]
pub fn QuestionnaireError(props: QuestionnaireErrorProps) -> Element {
    let base =
        attributes!(p { class: "dx-questionnaire-error", "data-slot": "questionnaire-error" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        primitive::QuestionnaireError { attributes: merged, children: props.children }
    }
}

/// The row of navigation buttons: Previous at the start, Skip and Next/Submit at the end.
#[component]
pub fn QuestionnaireActions(
    /// Additional attributes for the element.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    /// The content.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-questionnaire-actions", "data-slot": "questionnaire-actions" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        div { ..merged, {children} }
    }
}

/// The props for the styled navigation buttons.
#[derive(Props, Clone, PartialEq)]
pub struct QuestionnaireActionButtonProps {
    /// The [`Button`] variant (Previous and Skip default to outline, Next and Submit to primary).
    #[props(default)]
    pub variant: Option<ButtonVariant>,
    /// The [`Button`] size.
    #[props(default)]
    pub size: ButtonSize,
    /// Disable the button (opt-in, e.g. until the item is answered).
    #[props(default)]
    pub disabled: bool,
    /// Additional attributes for the `button`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Replaces the default text.
    pub children: Element,
}

/// A navigation button as a [`Button`], inside a wrapper that is `hidden` while the button does not apply
/// (the Button's own `display` would otherwise override the button's `hidden`).
fn action(
    action: QuestionnaireAction,
    slot: &'static str,
    props: QuestionnaireActionButtonProps,
) -> Element {
    let state = use_questionnaire_action(action, props.disabled);
    let variant = props.variant.unwrap_or(match action {
        QuestionnaireAction::Previous | QuestionnaireAction::Skip => ButtonVariant::Outline,
        QuestionnaireAction::Next | QuestionnaireAction::Submit => ButtonVariant::Primary,
    });
    let attributes = merge_attributes(vec![
        state.attributes,
        attributes!(button { "data-slot": slot }),
        props.attributes,
    ]);
    let default_label = props.children.as_ref().ok() == VNode::empty().as_ref().ok();
    rsx! {
        span {
            class: "dx-questionnaire-action",
            "data-action": slot,
            hidden: !state.visible,
            Button { variant, size: props.size, attributes,
                if default_label {
                    {action.default_label()}
                } else {
                    {props.children}
                }
            }
        }
    }
}

/// Back one item; hidden on the first.
#[component]
pub fn QuestionnairePrevious(props: QuestionnaireActionButtonProps) -> Element {
    action(
        QuestionnaireAction::Previous,
        "questionnaire-previous",
        props,
    )
}

/// Skip the active optional item; hidden on a required item.
#[component]
pub fn QuestionnaireSkip(props: QuestionnaireActionButtonProps) -> Element {
    action(QuestionnaireAction::Skip, "questionnaire-skip", props)
}

/// Validate and go to the next item; hidden on the last.
#[component]
pub fn QuestionnaireNext(props: QuestionnaireActionButtonProps) -> Element {
    action(QuestionnaireAction::Next, "questionnaire-next", props)
}

/// Submit the answers; shown only on the last item.
#[component]
pub fn QuestionnaireSubmit(props: QuestionnaireActionButtonProps) -> Element {
    action(QuestionnaireAction::Submit, "questionnaire-submit", props)
}
