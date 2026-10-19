use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, ChevronsUpDown, X};
use dioxus_primitives::combobox::{
    self, default_combobox_filter, ComboboxEmptyProps, ComboboxOptionProps,
};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled selected value. A signal holding `None` means no option is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The initially selected value when uncontrolled.
    #[props(default)]
    pub default_value: Option<T>,

    /// Called with the new value (`None` when the selection is cleared).
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the list.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// Whether the list starts open when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Called when the list opens or closes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The controlled text in the input.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text in the input when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Called with the new text whenever the user types.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether arrow-key navigation wraps from the last option to the first.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter, called with `(query, option_text_value)`; returns whether the option stays visible.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Placeholder text of the input.
    #[props(default)]
    pub placeholder: ReadSignal<String>,

    /// Accessible name for the input.
    #[props(default)]
    pub aria_label: Option<String>,

    /// Accessible name for the listbox of options.
    #[props(default)]
    pub list_aria_label: Option<String>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The combobox parts: trigger, list, groups and options.
    pub children: Element,
}

#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let base = attributes!(div { class: "dx-combobox" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/combobox/style.css") }
        combobox::Combobox {
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            query: props.query,
            default_query: props.default_query,
            on_query_change: props.on_query_change,
            roving_loop: props.roving_loop,
            filter: props.filter,
            attributes: merged,
            div { class: "dx-combobox-input-wrapper",
                combobox::ComboboxInput {
                    class: "dx-combobox-input",
                    placeholder: props.placeholder,
                    aria_label: props.aria_label.clone(),
                }
                ChevronsUpDown {
                    class: "dx-combobox-expand-icon",
                    size: "16px",
                }
            }
            combobox::ComboboxList {
                class: "dx-combobox-list",
                "data-dx-surface": "floating",
                aria_label: props.list_aria_label.clone(),
                {props.children}
            }
        }
    }
}

#[component]
pub fn ComboboxEmpty(props: ComboboxEmptyProps) -> Element {
    let base = attributes!(div { class: "dx-combobox-empty" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/combobox/style.css") }
        combobox::ComboboxEmpty {
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn ComboboxOption<T: Clone + PartialEq + 'static>(props: ComboboxOptionProps<T>) -> Element {
    let base = attributes!(div { class: "dx-combobox-option" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/combobox/style.css") }
        combobox::ComboboxOption::<T> {
            value: props.value,
            text_value: props.text_value,
            disabled: props.disabled,
            id: props.id,
            index: props.index,
            aria_label: props.aria_label,
            aria_roledescription: props.aria_roledescription,
            attributes: merged,
            {props.children}
            combobox::ComboboxItemIndicator {
                Check {
                    class: "dx-combobox-option-indicator",
                    size: "16px",
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxMultiProps<T: Clone + PartialEq + ToString + 'static = String> {
    /// The controlled selected values. A signal holding `None` leaves the selection to the component.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,

    /// The initially selected values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<T>,

    /// Called with the full list of selected values after every change.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,

    /// Form field name: every selected value is submitted under it, one entry each.
    #[props(default)]
    pub name: ReadSignal<Option<String>>,

    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the list.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// Whether the list starts open when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Called when the list opens or closes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The controlled text in the input.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text in the input when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Called with the new text whenever the user types.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether arrow-key navigation wraps from the last option to the first.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter, called with `(query, option_text_value)`; returns whether the option stays visible.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Placeholder text of the input.
    #[props(default)]
    pub placeholder: ReadSignal<String>,

    /// Accessible name for the input.
    #[props(default)]
    pub aria_label: Option<String>,

    /// Accessible name for the listbox of options.
    #[props(default)]
    pub list_aria_label: Option<String>,

    /// Shows a button that clears every chip while anything is selected.
    #[props(default)]
    pub show_clear: bool,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The combobox parts: the options, groups and the empty state.
    pub children: Element,
}

/// A combobox that selects several options and shows them as removable chips.
///
/// Backspace in the empty input removes the last chip; the list stays open while options are toggled.
#[component]
pub fn ComboboxMulti<T: Clone + PartialEq + ToString + 'static>(
    props: ComboboxMultiProps<T>,
) -> Element {
    let base = attributes!(div { class: "dx-combobox dx-combobox-multi" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/combobox/style.css") }
        combobox::ComboboxMulti {
            values: props.values,
            default_values: props.default_values,
            on_values_change: props.on_values_change,
            name: props.name,
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            query: props.query,
            default_query: props.default_query,
            on_query_change: props.on_query_change,
            roving_loop: props.roving_loop,
            filter: props.filter,
            attributes: merged,
            combobox::ComboboxChips {
                class: "dx-combobox-chips",
                chip_class: "dx-combobox-chip",
                remove_class: "dx-combobox-chip-remove",
                remove_icon: rsx! {
                    X { size: "12px" }
                },
                combobox::ComboboxInput {
                    class: "dx-combobox-chips-input",
                    placeholder: props.placeholder,
                    aria_label: props.aria_label.clone(),
                }
                if props.show_clear {
                    combobox::ComboboxClear {
                        class: "dx-combobox-clear",
                        X { size: "14px" }
                    }
                }
            }
            combobox::ComboboxList {
                class: "dx-combobox-list",
                "data-dx-surface": "floating",
                aria_label: props.list_aria_label.clone(),
                {props.children}
            }
        }
    }
}
