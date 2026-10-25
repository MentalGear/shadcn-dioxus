//! Root combobox component.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use super::super::context::{default_combobox_filter, ComboboxContext, ComboboxModel};
use crate::{
    merge_attributes,
    selectable::{
        use_selectable_root, use_single_selectable_value, RcPartialEqValue, SelectionMode,
        SingleSelectableValue,
    },
    use_controlled, use_form_reset_listener, use_unique_id, Controlled,
};

/// Props for [`Combobox`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled value. If supplied, the combobox is controlled
    /// and the signal's `None` value means no option is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The default uncontrolled value.
    #[props(default)]
    pub default_value: Option<T>,

    /// A selected value's display text, shown in the input. Required, like Base UI's `itemToStringLabel`:
    /// it is read at render time, so a preselected value's text is in the server-rendered HTML (the
    /// option registry fills in a client-only effect and can never supply it).
    pub item_to_label: Callback<T, String>,

    /// Callback fired when the value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Form field name. The selected value is submitted under it as `item_to_value`'s text (the
    /// label when that is not set); nothing is submitted while nothing is selected.
    #[props(default)]
    pub name: ReadSignal<Option<String>>,

    /// The text a selected value submits under `name`, like Base UI's `itemToStringValue`.
    /// Defaults to `item_to_label`.
    #[props(default)]
    pub item_to_value: Option<Callback<T, String>>,

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

    /// The controlled text query that filters options: the input's own text
    /// from the user's first edit after the popup opens (empty before it --
    /// opening over a selection leaves the label in the input and lists every
    /// option).
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text query when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the text query changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether typing highlights the first option that matches the query, so Enter picks it
    /// without an arrow key first (Base UI's `autoHighlight`). Opening the list highlights the
    /// selected option either way.
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

    /// Children.
    pub children: Element,
}

#[allow(clippy::too_many_arguments)] // the shared root hook for Combobox and ComboboxMulti; each argument is one prop or model value
pub(in crate::combobox) fn use_combobox_root(
    model: ComboboxModel,
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
    query: Controlled<String>,
    filter: Callback<(String, String), bool>,
    auto_highlight: ReadSignal<bool>,
) -> Memo<bool> {
    let selection_mode = match model {
        ComboboxModel::Single { .. } => SelectionMode::Single,
        ComboboxModel::Multi { .. } => SelectionMode::Multiple,
    };
    let selectable = use_selectable_root(
        values,
        set_value,
        selection_mode,
        disabled,
        roving_loop,
        open,
    );
    let (query, set_query) = use_controlled(query.value, query.default.cloned(), query.on_change);
    let open = selectable.open;
    // Placeholder value until `ComboboxInput` mounts and syncs its own id
    // in -- see `ComboboxContext::input_id`'s doc.
    let input_id = use_unique_id();
    let edited = use_signal(|| false);

    let ctx = use_context_provider(|| ComboboxContext {
        model,
        selectable,
        query,
        set_query,
        filter,
        input_id,
        edited,
    });

    // Auto-highlight: each edit of the query (the input's `oninput` clears the highlight) and each
    // change to which options pass the filter puts the highlight on the first matching option.
    // Arrow keys move it afterwards, since nothing read here changes when they do.
    use_effect(move || {
        if !auto_highlight() || !open() || !*edited.read() {
            return;
        }
        if selectable.collection.order_pending() {
            return;
        }
        let mut ctx = ctx;
        ctx.focus_first_visible();
    });

    open
}

/// A single-select autocomplete input with a filterable popup list.
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let SingleSelectableValue {
        values: selected,
        set_value,
        reset_to_default,
        clear,
    } = use_single_selectable_value(
        props.value,
        props.default_value,
        props.on_value_change,
        "combobox",
    );

    let typed = move |value: &RcPartialEqValue| -> T {
        value
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("combobox and option value types must match"))
            .clone()
    };
    let item_to_label = props.item_to_label;
    let label = use_callback(move |value: RcPartialEqValue| item_to_label.call(typed(&value)));
    let open = use_combobox_root(
        ComboboxModel::Single { clear, label },
        selected,
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

    let owned = attributes!(div {
        "data-state": if open() { "open" } else { "closed" },
        "data-disabled": (props.disabled)(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    // Native `<form reset>` cannot reach the selection: restore the default through a hidden anchor.
    let reset_id = use_unique_id();
    use_form_reset_listener(reset_id, move || reset_to_default.call(()));
    let item_to_value = props.item_to_value.unwrap_or(item_to_label);
    let submitted = (props.name)().zip(selected.read().first().map(typed));
    let disabled = (props.disabled)();

    rsx! {
        div {
            ..merged,
            input { id: reset_id, r#type: "hidden", disabled: true }
            if let Some((name, value)) = submitted {
                input {
                    r#type: "hidden",
                    name: "{name}",
                    value: item_to_value.call(value),
                    disabled,
                }
            }
            {props.children}
        }
    }
}
