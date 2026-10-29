//! Defines the [`Switch`] component and its sub-components.

use crate::default_checked::use_default_checked;
use crate::state_attrs;
use crate::{merge_attributes, use_controlled, use_form_reset_listener, use_unique_id};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::rc::Rc;

/// The props for the [`Switch`] component.
#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    /// The controlled checked state of the switch.
    pub checked: ReadSignal<Option<bool>>,

    /// The default checked state when uncontrolled.
    #[props(default = false)]
    pub default_checked: bool,

    /// Whether the switch is disabled.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub disabled: ReadSignal<bool>,

    /// Whether the switch is required in a form.
    #[props(default)]
    pub required: ReadSignal<bool>,

    /// Whether the switch is read-only: it stays focusable and is announced (`aria-readonly`),
    /// but a click or Space does not change its state, and its value still submits with a form.
    /// Distinct from `disabled`, which removes it from the tab order and from the submission.
    #[props(default)]
    pub read_only: ReadSignal<bool>,

    /// The name attribute for form submission.
    #[props(default)]
    pub name: ReadSignal<String>,

    /// The `id` of a `<form>` the hidden input belongs to (the HTML `form` attribute), so a control
    /// rendered outside its `<form>` still submits and resets with it. `None` leaves the input's
    /// owner to the DOM ancestor.
    #[props(default)]
    pub form: ReadSignal<Option<String>>,

    /// The value attribute for form submission.
    #[props(default = ReadSignal::new(Signal::new(String::from("on"))))]
    pub value: ReadSignal<String>,

    /// Callback fired when the checked state changes.
    #[props(default)]
    pub on_checked_change: Callback<bool>,

    /// Additional attributes to apply to the switch element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the switch component.
    pub children: Element,
}

/// # Switch
///
/// The `Switch` component is a toggle control that allows users to switch a state on or off.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::switch::{Switch, SwitchThumb};
/// #[component]
/// fn Demo() -> Element {
///     let mut checked = use_signal(|| false);
///     rsx! {
///         Switch {
///             checked: checked(),
///             aria_label: "Switch Demo",
///             SwitchThumb {}
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Switch`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the switch. Values are `checked` or `unchecked`.
/// - `data-checked` / `data-unchecked`: Present (empty) while checked / unchecked, absent otherwise.
/// - `data-disabled`: Indicates if the switch is disabled. Values are `true` or `false`.
/// - `data-readonly`: Present (empty) while the switch is read-only, absent otherwise. The
///   element also carries `aria-readonly="true"` then.
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let (checked, set_checked) = use_controlled(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );

    // The hidden mirror <input> needs a stable id so the form-reset listener
    // below can find it (and, through it, its owning form) in the DOM.
    let hidden_input_id = use_unique_id();
    let default_checked = props.default_checked;
    use_form_reset_listener(hidden_input_id, move || set_checked.call(default_checked));
    // `<form reset>` restores the hidden input from `.defaultChecked`; see `default_checked.rs`.
    use_default_checked(move || hidden_input_id.cloned(), move || default_checked);

    // Safari and Firefox on macOS do not focus <button> on click by default
    // (matches the macOS "Press Tab to highlight items" preference). Capture
    // a mounted ref so we can explicitly focus on click, matching what
    // Radix UI / Headless UI / Material UI do for the same reason.
    let mut button_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let press = crate::modality::use_press_kind();

    // `type` is an overridable default; the rest is this switch's own
    // functional state (role/aria-checked/aria-required define the widget,
    // the `data-*` pair mirrors its real state).
    let defaults = attributes!(button { r#type: "button" });
    let owned = attributes!(button {
        role: "switch",
        aria_checked: checked,
        aria_required: props.required,
        ..state_attrs::checked(checked()),
        // Only add data-disabled when actually disabled
        ..state_attrs::disabled(props.disabled),
        aria_readonly: (props.read_only)().then_some("true"),
        ..state_attrs::readonly(props.read_only),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        button {
            value: props.value,
            disabled: props.disabled,

            onmounted: move |evt| button_ref.set(Some(evt.data())),
            onpointerdown: move |evt| press.pressed(&evt),
            onclick: move |_| {
                if (props.read_only)() {
                    return;
                }
                let new_checked = !checked();
                set_checked.call(new_checked);
                // Safari click-focus remedy for a mouse press; never after a touch or pen press
                // (see `crate::modality::PressKind`).
                if !press.was_touch() {
                    if let Some(node) = button_ref() {
                        spawn(async move {
                            // focus-ok: Safari click-focus remedy, mouse presses only (modality.rs)
                            let _ = node.set_focus(true).await;
                        });
                    }
                }
            },

            // Switches should only toggle on Space, not Enter
            onkeydown: move |e| {
                if e.key() == Key::Enter {
                    e.prevent_default();
                }
            },

            ..merged,
            {props.children}
        }

        // Hidden input for form submission
        input {
            id: hidden_input_id,
            type: "checkbox",
            aria_hidden: true,
            tabindex: -1,
            name: props.name,
            form: props.form,
            value: props.value,
            required: props.required,
            // `checked` is the server-rendered content attribute and the live sync (->
            // `.checked` IDL property) that keeps this mirror in step with the visible switch;
            // the default the HTML reset algorithm reads is `use_default_checked` above, never
            // `initial_checked` (row 253, `default_checked.rs`; same as Checkbox's BubbleInput).
            checked,
            disabled: props.disabled,
            style: "transform: translateX(-100%); position: absolute; pointer-events: none; opacity: 0; margin: 0; width: 0; height: 0;",
        }
    }
}

/// The props for the [`SwitchThumb`] component.
#[derive(Props, Clone, PartialEq)]
pub struct SwitchThumbProps {
    /// Additional attributes to apply to the thumb element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the thumb component.
    pub children: Element,
}

/// # SwitchThumb
///
/// The `SwitchThumb` component represents the visual thumb indicator that moves when the switch is toggled.
///
/// This must be used inside a [`Switch`] component.
///
/// ## Example
///
/// ```rust
///
/// use dioxus::prelude::*;
/// use dioxus_primitives::switch::{Switch, SwitchThumb};
/// #[component]
/// fn Demo() -> Element {
///     let mut checked = use_signal(|| false);
///     rsx! {
///         Switch {
///             checked: checked(),
///             aria_label: "Switch Demo",
///             SwitchThumb {}
///         }
///     }
/// }
/// ```
#[component]
pub fn SwitchThumb(props: SwitchThumbProps) -> Element {
    rsx! {
        span { ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_checked::test_support::hidden_inputs;

    #[component]
    fn Defaults() -> Element {
        rsx! {
            Switch { default_checked: true, name: "a", SwitchThumb {} }
            Switch { name: "b", SwitchThumb {} }
        }
    }

    /// A read-only Switch is announced and marked; a writable one carries neither attribute.
    #[test]
    fn read_only_sets_aria_readonly_and_data_readonly() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Switch { read_only: true, name: "a", SwitchThumb {} }
                Switch { name: "b", SwitchThumb {} }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let switches = crate::test_support::find_elements(&html, |a| {
            a.get("role").map(String::as_str) == Some("switch")
        });
        assert_eq!(switches.len(), 2, "{html}");
        assert_eq!(
            switches[0].attrs.get("aria-readonly").map(String::as_str),
            Some("true")
        );
        assert!(switches[0].attrs.contains_key("data-readonly"), "{html}");
        assert!(!switches[1].attrs.contains_key("aria-readonly"), "{html}");
        assert!(!switches[1].attrs.contains_key("data-readonly"), "{html}");
    }

    /// A default-checked Switch's hidden input is server-rendered with the `checked` content
    /// attribute and never with a literal `initial_checked` (backlog row 253).
    #[test]
    fn default_checked_is_a_checked_attribute_not_initial_checked() {
        let mut dom = VirtualDom::new(Defaults);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let flags: Vec<bool> = hidden_inputs(&html, "checkbox")
            .iter()
            .map(|(checked, _)| *checked)
            .collect();
        assert_eq!(flags, [true, false], "{html}");
    }
}
