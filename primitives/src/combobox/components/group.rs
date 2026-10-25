//! Combobox option groups.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::{listbox::ListboxContext, merge_attributes, use_id_or, use_unique_id};

/// The group an option belongs to: how many of its options pass the filter, and its label's id.
#[derive(Clone, Copy)]
pub(in crate::combobox) struct ComboboxGroupContext {
    pub(in crate::combobox) visible: Signal<usize>,
    labeled_by: Signal<Option<String>>,
}

/// Props for [`ComboboxGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupProps {
    /// Optional id for the group element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// A [`ComboboxGroupLabel`] and the group's [`ComboboxOption`](super::ComboboxOption)s.
    pub children: Element,
}

/// A labelled section of a [`ComboboxList`](super::ComboboxList): a `role="group"` named by its
/// [`ComboboxGroupLabel`]. Its options stay in the list's one keyboard order. While the query
/// leaves none of its options visible, the whole group (its label included) is `hidden`.
#[component]
pub fn ComboboxGroup(props: ComboboxGroupProps) -> Element {
    let visible = use_signal(|| 0usize);
    let labeled_by = use_signal(|| None);
    use_context_provider(|| ComboboxGroupContext {
        visible,
        labeled_by,
    });
    let render = use_context::<ListboxContext>().render;
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);

    let owned = attributes!(div {
        role: "group",
        aria_labelledby: labeled_by,
        hidden: visible() == 0,
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        if render() {
            div { id, ..merged, {props.children} }
        } else {
            // Closed: the options stay mounted (they register with the list) but render nothing.
            {props.children}
        }
    }
}

/// Props for [`ComboboxGroupLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupLabelProps {
    /// Optional id for the label element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The label's content.
    pub children: Element,
}

/// The heading of a [`ComboboxGroup`], which names the group. It is not an option: the keyboard
/// skips it and it cannot be picked.
#[component]
pub fn ComboboxGroupLabel(props: ComboboxGroupLabelProps) -> Element {
    let mut group = use_context::<ComboboxGroupContext>();
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);
    use_effect(move || group.labeled_by.set(Some(id())));

    rsx! {
        div { id, ..props.attributes, {props.children} }
    }
}
