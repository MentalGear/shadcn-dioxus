//! Multi-select combobox: the `Vec<T>` model, chips and the clear button.
//!
//! [`ComboboxMulti`] is the second root of the combobox. It shares every part
//! with [`Combobox`](super::Combobox) (input, list, options, empty state, clear
//! button) through one `ComboboxContext`; what differs is the *type* of its model.
//! The single root takes and reports `Option<T>`, this one `Vec<T>`, and the
//! context records which one it is (`ComboboxModel`), so the operations that
//! only exist on a set -- remove one value, Backspace on an empty input, chips --
//! are unreachable from a single-select root rather than guarded at each call
//! site.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use super::components::combobox::use_combobox_root;
use super::context::{default_combobox_filter, ComboboxContext, ComboboxModel};
use crate::{
    direction::{use_direction, HorizontalNav},
    merge_attributes,
    selectable::RcPartialEqValue,
    use_controlled, use_form_reset_listener, use_id_or, use_unique_id, Controlled,
};

/// Moves DOM focus to the element with the given id.
const FOCUS_ID_JS: &str = r#"
const id = await dioxus.recv();
const el = document.getElementById(id);
if (el) el.focus();
"#;

/// Moves focus between the chips' remove buttons and the input.
///
/// Receives `[current_button_id_or_empty, "prev" | "next", input_id]`; the chips are the
/// input's enclosing `[data-dx-combobox-chips]` field.
/// From the input (no current button) `prev` lands on the last chip; `next`
/// past the last chip lands on the input.
const CHIP_NAV_JS: &str = r#"
const [current, nav, inputId] = await dioxus.recv();
const input = document.getElementById(inputId);
const root = input && input.closest('[data-dx-combobox-chips]');
if (!root) return;
const buttons = [...root.querySelectorAll('[data-dx-combobox-chip-remove]')];
let i = current ? buttons.findIndex((b) => b.id === current) : buttons.length;
i += nav === 'prev' ? -1 : 1;
if (i < 0) i = 0;
const target = i >= buttons.length ? document.getElementById(inputId) : buttons[i];
if (target) target.focus();
"#;

fn focus_id(id: String) {
    let eval = document::eval(FOCUS_ID_JS);
    let _ = eval.send(id);
}

/// Props for [`ComboboxMulti`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxMultiProps<T: Clone + PartialEq + ToString + 'static> {
    /// The controlled selected values. A signal holding `None` leaves the
    /// selection to the component.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,

    /// The initially selected values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<T>,

    /// A selected value's display text, shown in its chip and named by its remove button.
    ///
    /// Required, like Base UI's `itemToStringLabel`: it is called at render time, so the
    /// chips of preselected values carry their text in the server-rendered HTML. The option
    /// registry cannot supply it (it fills in a client-only effect, after the chips).
    pub item_to_label: Callback<T, String>,

    /// Called with the full list of selected values after every change.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,

    /// Form field name. Every selected value is submitted under it, once each
    /// (`FormData.getAll(name)`), as the value's `ToString`.
    #[props(default)]
    pub name: ReadSignal<Option<String>>,

    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the popup.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The controlled text query that filters options.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text query when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the text query changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether typing highlights the first option that matches the query, so Enter toggles it
    /// without an arrow key first (Base UI's `autoHighlight`).
    #[props(default)]
    pub auto_highlight: ReadSignal<bool>,

    /// Whether arrow-key navigation should wrap.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter callback. Receives `(query, option_text_value)`.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children: [`ComboboxChips`] (chips, input, clear button) and [`ComboboxList`](super::ComboboxList).
    pub children: Element,
}

/// A multi-select autocomplete: a popup list that stays open while options are
/// toggled, the selection shown as removable chips.
#[component]
pub fn ComboboxMulti<T: Clone + PartialEq + ToString + 'static>(
    props: ComboboxMultiProps<T>,
) -> Element {
    let default_values = props.default_values.clone();
    let (current, set_values) = use_controlled(
        props.values,
        props.default_values.clone(),
        props.on_values_change,
    );

    let values = use_memo(move || current().into_iter().map(RcPartialEqValue::new).collect());
    let typed = move |value: &RcPartialEqValue| -> T {
        value
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("ComboboxMulti and option value types must match"))
            .clone()
    };
    // Toggle: picking a selected option removes it.
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let incoming = typed(&incoming);
        let mut next = current();
        match next.iter().position(|v| *v == incoming) {
            Some(at) => {
                next.remove(at);
            }
            None => next.push(incoming),
        }
        set_values.call(next);
    });
    let remove = use_callback(move |value: RcPartialEqValue| {
        let value = typed(&value);
        let mut next = current();
        next.retain(|v| *v != value);
        set_values.call(next);
    });
    let clear = use_callback(move |()| set_values.call(Vec::new()));
    let item_to_label = props.item_to_label;
    let label = use_callback(move |value: RcPartialEqValue| item_to_label.call(typed(&value)));

    let open = use_combobox_root(
        ComboboxModel::Multi {
            remove,
            clear,
            label,
        },
        values,
        set_value,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        Controlled {
            value: props.query,
            default: props.default_query,
            on_change: props.on_query_change,
        },
        props.filter,
        props.auto_highlight,
    );

    // Native `<form reset>` cannot reach the signals above: restore the default through a hidden anchor.
    let reset_id = use_unique_id();
    use_form_reset_listener(reset_id, move || set_values.call(default_values.clone()));

    let owned = attributes!(div {
        "data-state": if open() { "open" } else { "closed" },
        "data-disabled": (props.disabled)(),
        "data-multiple": "true",
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);
    let name = (props.name)();
    let disabled = (props.disabled)();

    rsx! {
        div {
            ..merged,
            input { id: reset_id, r#type: "hidden", disabled: true }
            if let Some(name) = name {
                for value in current() {
                    input {
                        key: "{value.to_string()}",
                        r#type: "hidden",
                        name: "{name}",
                        value: "{value.to_string()}",
                        disabled,
                    }
                }
            }
            {props.children}
        }
    }
}

/// Props for [`ComboboxChips`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxChipsProps {
    /// Optional id for the chips container.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Class of each chip.
    #[props(default)]
    pub chip_class: String,

    /// Class of each chip's remove button.
    #[props(default)]
    pub remove_class: String,

    /// Accessible-name prefix of a chip's remove button: "Remove" gives "Remove Next.js".
    #[props(default = "Remove".to_string())]
    pub remove_label: String,

    /// Icon inside each remove button.
    #[props(default = rsx! { "\u{d7}" })]
    pub remove_icon: Element,

    /// Additional attributes for the container.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Rendered after the chips: the [`ComboboxInput`](super::ComboboxInput) and a [`ComboboxClear`].
    pub children: Element,
}

/// The field of a multi-select: one removable chip per selected value, then its
/// children. It is the popup's anchor, so the list matches its width.
///
/// A click on the empty part of the field focuses the input.
#[component]
pub fn ComboboxChips(props: ComboboxChipsProps) -> Element {
    let ctx = use_context::<ComboboxContext>();
    let direction = use_direction(None);
    let generated = use_unique_id();
    let id = use_id_or(generated, props.id);

    let multi = ctx.is_multi();
    let selected: Vec<RcPartialEqValue> = ctx.selectable.values.read().clone();
    let disabled = (ctx.selectable.disabled)();
    let input_id = ctx.input_id;

    let owned = attributes!(div {
        // The popup anchors to this field, not to the input inside it (the input drops its own anchor in a multi-select).
        style: crate::top_layer::anchor_name_style(&ctx.input_id.cloned()),
        "data-dx-combobox-chips": "true",
        "data-disabled": disabled,
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            id,
            onpointerdown: move |event| {
                // Pressing the field's own padding must not blur the input.
                event.prevent_default();
                focus_id(input_id.cloned());
            },
            ..merged,
            if multi {
                for (n , value) in selected.into_iter().enumerate() {
                    {
                        let label = ctx.label_of(&value);
                        let button_id = format!("{}-chip-{n}", id());
                        let remove_name = format!("{} {label}", props.remove_label);
                        let key = format!("{n}-{label}");
                        rsx! {
                            span {
                                key: "{key}",
                                class: props.chip_class.clone(),
                                "data-dx-combobox-chip": "true",
                                span { "{label}" }
                                button {
                                    id: button_id.clone(),
                                    r#type: "button",
                                    class: props.remove_class.clone(),
                                    aria_label: remove_name,
                                    disabled,
                                    "data-dx-combobox-chip-remove": "true",
                                    onpointerdown: move |event| {
                                        event.prevent_default();
                                        event.stop_propagation();
                                    },
                                    onclick: {
                                        let value = value.clone();
                                        move |_| {
                                            if let ComboboxModel::Multi { remove, .. } = ctx.model {
                                                remove.call(value.clone());
                                            }
                                            focus_id(input_id.cloned());
                                        }
                                    },
                                    onkeydown: {
                                        let value = value.clone();
                                        let button_id = button_id.clone();
                                        move |event: KeyboardEvent| {
                                            let key = event.key();
                                            if matches!(key, Key::Backspace | Key::Delete) {
                                                if let ComboboxModel::Multi { remove, .. } = ctx.model {
                                                    remove.call(value.clone());
                                                }
                                                focus_id(input_id.cloned());
                                                event.prevent_default();
                                            } else if let Some(nav) = direction.resolve_horizontal(&key) {
                                                chip_nav(button_id.clone(), nav, input_id.cloned());
                                                event.prevent_default();
                                            }
                                        }
                                    },
                                    {props.remove_icon.clone()}
                                }
                            }
                        }
                    }
                }
            }
            {props.children}
        }
    }
}

/// Move focus one chip toward `nav` (or from the input when `current` is empty).
pub(super) fn chip_nav(current: String, nav: HorizontalNav, input_id: String) {
    let eval = document::eval(CHIP_NAV_JS);
    let nav = match nav {
        HorizontalNav::Prev => "prev",
        HorizontalNav::Next => "next",
    };
    let _ = eval.send((current, nav.to_string(), input_id));
}

/// Props for [`ComboboxClear`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxClearProps {
    /// Additional attributes for the button.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The button's content, usually an icon.
    pub children: Element,
}

/// Clears the selection -- every value of a [`ComboboxMulti`], the one value of a
/// [`Combobox`](super::Combobox) -- and returns focus to the input. Renders nothing while nothing is
/// selected.
#[component]
pub fn ComboboxClear(props: ComboboxClearProps) -> Element {
    let mut ctx = use_context::<ComboboxContext>();
    let input_id = ctx.input_id;
    if ctx.selectable.values.read().is_empty() {
        return rsx! {};
    }
    let defaults = attributes!(button {
        aria_label: "Clear"
    });
    let owned = attributes!(button {
        r#type: "button",
        disabled: (ctx.selectable.disabled)(),
        "data-dx-combobox-clear": "true",
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        button {
            onpointerdown: move |event| {
                event.prevent_default();
                event.stop_propagation();
            },
            onclick: move |_| {
                ctx.clear();
                focus_id(input_id.cloned());
            },
            ..merged,
            {props.children}
        }
    }
}
