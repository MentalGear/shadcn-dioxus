//! Defines the [`RadioGroup`] component and its sub-components.

use std::collections::HashMap;

use crate::default_checked::use_default_checked;
use crate::{
    collection::{
        collection_item, use_collection_provider_with, use_item, use_item_index, CollectionOptions,
        CollectionState,
    },
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    merge_attributes, use_controlled, use_effect_with_cleanup, use_form_reset_listener,
    use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

#[derive(Clone, Copy)]
struct RadioGroupCtx {
    // State
    disabled: ReadSignal<bool>,
    required: ReadSignal<bool>,
    read_only: ReadSignal<bool>,
    name: ReadSignal<String>,
    form: ReadSignal<Option<String>>,
    /// The value the group resets to on `<form reset>`. Mirrors
    /// `Checkbox`'s `default_checked` (it is the *prop*, not a live snapshot
    /// of the value at mount, so it stays correct even if `default_value`
    /// were ever changed dynamically).
    default_value: ReadSignal<String>,
    value: Memo<String>,
    set_value: Callback<String>,

    // Keyboard nav data
    // A map of tabindex -> value in the enabled radio items
    values: Signal<HashMap<usize, String>>,
    focus: CollectionState,

    horizontal: ReadSignal<bool>,

    // Text direction, for `ArrowLeft`/`ArrowRight`'s roving-focus role --
    // see `direction::Direction::resolve_horizontal`'s doc.
    direction: ReadSignal<Direction>,
}

impl RadioGroupCtx {
    /// Set the currently focused radio item.
    ///
    /// This should be used by `focus`/`focusout` event only to start tracking focus.
    fn set_focus(&mut self, id: Option<usize>) {
        self.focus.set_focus(id);
    }

    /// Set the value of the radio group.
    fn set_value(&mut self, value: String) {
        let current_value = self.value.peek();
        if *current_value == value {
            return; // No change, do nothing
        }
        self.set_value.call(value);
    }

    fn focus_next(&mut self) {
        self.focus.focus_next();
        self.select_focused_value();
    }

    fn focus_prev(&mut self) {
        self.focus.focus_prev();
        self.select_focused_value();
    }

    fn select_focused_value(&mut self) {
        // A read-only group moves focus with the arrow keys but never changes its selection.
        if (self.read_only)() {
            return;
        }
        if let Some(current_focus) = self.focus.focused_index() {
            let value = { self.values.read().get(&current_focus).cloned() };
            if let Some(value) = value {
                self.set_value(value.clone());
            }
        }
    }

    fn focus_start(&mut self) {
        self.focus.focus_first();
    }

    fn focus_end(&mut self) {
        self.focus.focus_last();
    }
}

/// The props for the [`RadioGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    /// The controlled value of the selected radio item.
    pub value: ReadSignal<Option<String>>,

    /// The default selected value when uncontrolled.
    #[props(default)]
    pub default_value: String,

    /// Callback fired when the selected value changes.
    #[props(default)]
    pub on_value_change: Callback<String>,

    /// Whether the radio group is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the radio group is required in a form.
    #[props(default)]
    pub required: ReadSignal<bool>,

    /// Whether the radio group is read-only: its items stay focusable (arrow keys still move
    /// focus) and the group is announced (`aria-readonly`), but a click, Space or an arrow key
    /// does not change the selection, and the selected value still submits with a form.
    /// Distinct from `disabled`, which removes the items from the tab order and from the
    /// submission.
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

    /// Whether the radio group is horizontal.
    #[props(default)]
    pub horizontal: ReadSignal<bool>,

    /// The text direction for `ArrowLeft`/`ArrowRight` roving focus.
    /// Defaults to the nearest [`crate::direction::DirectionProvider`], or
    /// LTR if there is none. See [`crate::direction::use_direction`].
    #[props(default)]
    pub dir: Option<Direction>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to apply to the radio group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the radio group component.
    pub children: Element,
}

/// # RadioGroup
///
/// The `RadioGroup` component is a container for a group of [`RadioItem`] components that allows users to select a single option from a list of choices.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::radio_group::{RadioGroup, RadioItem};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         RadioGroup {
///             RadioItem {
///                 value: "option1".to_string(),
///                 "Blue"
///             }
///             RadioItem {
///                 value: "option2".to_string(),
///                 "Red"
///             }
///             RadioItem {
///                 value: "option3".to_string(),
///                 disabled: true,
///                 "Green"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`RadioGroup`] component defines the following data attributes you can use to control styling:
/// - `data-orientation`: Indicates the orientation of the radio group. Values are `horizontal` or `vertical`.
/// - `data-disabled`: Indicates if the radio group is disabled. Values are `true` or `false`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
/// - `data-readonly`: Present (empty) while the group is read-only, absent otherwise. The group
///   also carries `aria-readonly="true"` then.
#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);
    // Snapshot the default before `use_controlled` consumes it -- each
    // RadioItem's hidden radio needs it (for `use_default_checked`) and the
    // group's own form-reset listener needs it (to restore the Rust-side
    // selection). `RadioGroupCtx` derives `Copy`, so this has to be a
    // `Signal`, not the plain `String` the prop is.
    let default_value = use_signal(|| props.default_value.clone());
    let (value, set_value) =
        use_controlled(props.value, props.default_value, props.on_value_change);

    // Native radio-group semantics: when nothing is selected, every radio stays
    // in the tab order until one is focused or checked.
    let focus = use_collection_provider_with(
        props.roving_loop,
        CollectionOptions {
            tabbable_when_empty: true,
        },
    );
    let mut ctx = use_context_provider(|| RadioGroupCtx {
        value,
        set_value,
        disabled: props.disabled,
        required: props.required,
        read_only: props.read_only,
        name: props.name,
        form: props.form,
        default_value: default_value.into(),

        values: Signal::new(Default::default()),
        focus,
        horizontal: props.horizontal,
        direction: direction_signal,
    });

    let owned = attributes!(div {
        role: "radiogroup",
        "data-orientation": if (props.horizontal)() { "horizontal" } else { "vertical" },
        "data-disabled": (props.disabled)(),
        "data-direction": direction.as_str(),
        aria_required: props.required,
        aria_readonly: (props.read_only)().then_some("true"),
        "data-readonly": (props.read_only)().then_some(""),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            dir: direction.as_str(),

            onfocusout: move |_| ctx.set_focus(None),
            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`RadioItem`] component
#[derive(Props, Clone, PartialEq)]
pub struct RadioItemProps {
    /// The value of the radio item. This will be passed to [`RadioGroupProps::on_value_change`] when selected.
    pub value: ReadSignal<String>,
    /// The item's place in keyboard order within the [`RadioGroup`]. Leave it unset: items are
    /// ordered as they are created, which is tree order. Set it on every item only when an item is
    /// inserted between items that are already mounted, or a keyed list reorders them.
    #[props(default)]
    pub index: ReadSignal<Option<usize>>,

    /// Whether the radio item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional ID for the radio item element.
    pub id: Option<String>,
    /// Optional class for the radio item element.
    pub class: Option<String>,

    /// Additional attributes to apply to the radio item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the radio item component.
    pub children: Element,
}

/// # RadioItem
///
/// The `RadioItem` component represents a single radio button within a [`RadioGroup`]. Only one radio item can be selected at a time within a group.
///
/// This must be used inside a [`RadioGroup`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::radio_group::{RadioGroup, RadioItem};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         RadioGroup {
///             RadioItem {
///                 value: "option1".to_string(),
///                 "Blue"
///             }
///             RadioItem {
///                 value: "option2".to_string(),
///                 "Red"
///             }
///             RadioItem {
///                 value: "option3".to_string(),
///                 disabled: true,
///                 "Green"
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`RadioItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the radio item. Values are `checked` or `unchecked`.
/// - `data-disabled`: Indicates if the radio item is disabled. Values are `true` or `false`.
/// - `data-readonly`: Present (empty) while the group is read-only, absent otherwise.
#[component]
pub fn RadioItem(props: RadioItemProps) -> Element {
    let mut ctx: RadioGroupCtx = use_context();
    let disabled = move || (ctx.disabled)() || (props.disabled)();
    let order = use_item_index(ctx.focus, props.index);
    let index = order.index;

    use_effect_with_cleanup(move || {
        let index = index();
        if disabled() {
            ctx.values.write().remove(&index);
        } else {
            ctx.values.write().insert(index, (props.value)());
        }
        move || {
            ctx.values.write().remove(&index);
        }
    });

    let value = (props.value)().clone();
    let checked = use_memo(move || (ctx.value)() == value);

    let item = use_item(
        collection_item(ctx.focus, index)
            .disabled(disabled)
            .selected(move || checked.cloned()),
    );
    let tab_index = item.tabindex;
    let onmounted = item.onmounted();

    // Hidden native radio mirroring this item, so the group participates in
    // HTML form submission -- see dev-docs/plan.md Phase 1.2. Rendered
    // unconditionally, matching Checkbox's BubbleInput: this repo's decision
    // table (dev-docs/reference/platform-rationale.md §1) follows upstream over
    // Radix's `closest('form')` gate, which Dioxus has no cheap equivalent
    // for. Shape ported from dignifiedquire/dx-components (MIT OR
    // Apache-2.0), primitives/src/radio_group.rs (RadioGroupItem's hidden
    // `input`, ~L260-273 as read from that fork's tree) -- adapted to this
    // repo's per-item `checked` memo, the `checked`/`.defaultChecked` split
    // (`use_default_checked`) needed for correct `<form reset>` behavior (that
    // fork's own hidden input lacks the split, as a bare `checked` binding
    // does; see `Checkbox::BubbleInput` in checkbox.rs), and a `reset` listener
    // that resyncs the Dioxus-side selection, which neither fork implements.
    let hidden_input_id = use_unique_id();
    use_form_reset_listener(hidden_input_id, move || {
        ctx.set_value(ctx.default_value.cloned());
    });
    let is_default = use_memo(move || (ctx.default_value)() == (props.value)());
    // `<form reset>` restores the hidden radio from `.defaultChecked`; see `default_checked.rs`.
    use_default_checked(move || hidden_input_id.cloned(), move || *is_default.read());

    // `type` is an overridable default; the rest is this item's own
    // functional state (role/tabindex define the widget and its
    // roving-focus wiring, aria-checked/data-state/data-disabled mirror
    // its real state).
    let defaults = attributes!(button { r#type: "button" });
    let owned = attributes!(button {
        role: "radio",
        tabindex: tab_index,
        aria_checked: checked,
        "data-state": if checked() { "checked" } else { "unchecked" },
        "data-disabled": disabled(),
        "data-readonly": (ctx.read_only)().then_some(""),
    });
    let merged = merge_attributes(vec![
        order.marker_attribute(),
        defaults,
        props.attributes.clone(),
        owned,
    ]);

    rsx! {
        button {
            id: props.id,
            class: props.class,
            disabled: disabled(),

            onclick: move |_| {
                if (ctx.read_only)() {
                    return;
                }
                let value = (props.value)().clone();
                ctx.set_value(value);
            },

            onmounted,
            onfocus: move |_| ctx.set_focus(Some(index())),

            onkeydown: move |event: Event<KeyboardData>| {
                let key = event.key();
                let horizontal = (ctx.horizontal)();
                let mut prevent_default = true;
                match key {
                    Key::ArrowUp if !horizontal => ctx.focus_prev(),
                    Key::ArrowDown if !horizontal => ctx.focus_next(),
                    Key::ArrowLeft | Key::ArrowRight if horizontal => {
                        match (ctx.direction)().resolve_horizontal(&key) {
                            Some(HorizontalNav::Prev) => ctx.focus_prev(),
                            Some(HorizontalNav::Next) => ctx.focus_next(),
                            None => {}
                        }
                    }
                    Key::Home => ctx.focus_start(),
                    Key::End => ctx.focus_end(),
                    _ => prevent_default = false,
                };
                if prevent_default {
                    event.prevent_default();
                }
            },
            ..merged,

            {props.children}
        }

        input {
            id: hidden_input_id,
            type: "radio",
            aria_hidden: "true",
            tabindex: "-1",
            position: "absolute",
            pointer_events: "none",
            opacity: "0",
            margin: "0",
            transform: "translateX(-100%)",

            name: ctx.name,
            form: ctx.form,
            value: (props.value)(),
            required: ctx.required,
            disabled: disabled(),
            // `checked` is the server-rendered content attribute and the live sync with the
            // visible radio; the default `<form reset>` restores from is `use_default_checked`
            // above, never `initial_checked` (row 253, `default_checked.rs`).
            checked: checked(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_checked::test_support::hidden_inputs;

    #[component]
    fn Defaults() -> Element {
        rsx! {
            RadioGroup { default_value: "pro", name: "plan",
                RadioItem { value: "starter", "Starter" }
                RadioItem { value: "pro", "Pro" }
                RadioItem { value: "scale", "Scale" }
            }
        }
    }

    /// A read-only group is announced and marked on the group and its items; a writable one
    /// carries neither attribute.
    #[test]
    fn read_only_marks_the_group_and_its_items() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                RadioGroup { read_only: true, default_value: "a",
                    RadioItem { value: "a", "A" }
                }
                RadioGroup { default_value: "a",
                    RadioItem { value: "a", "A" }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let groups = crate::test_support::find_elements(&html, |a| {
            a.get("role").map(String::as_str) == Some("radiogroup")
        });
        assert_eq!(groups.len(), 2, "{html}");
        assert_eq!(
            groups[0].attrs.get("aria-readonly").map(String::as_str),
            Some("true")
        );
        assert!(groups[0].attrs.contains_key("data-readonly"), "{html}");
        assert!(!groups[1].attrs.contains_key("aria-readonly"), "{html}");
        assert!(!groups[1].attrs.contains_key("data-readonly"), "{html}");
        let items = crate::test_support::find_elements(&html, |a| {
            a.get("role").map(String::as_str) == Some("radio")
        });
        assert_eq!(items.len(), 2, "{html}");
        assert!(items[0].attrs.contains_key("data-readonly"), "{html}");
        assert!(!items[1].attrs.contains_key("data-readonly"), "{html}");
    }

    /// Only the default RadioItem's hidden radio is server-rendered with the `checked` content
    /// attribute, and none with a literal `initial_checked` (backlog row 253).
    #[test]
    fn default_checked_is_a_checked_attribute_not_initial_checked() {
        let mut dom = VirtualDom::new(Defaults);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let flags: Vec<bool> = hidden_inputs(&html, "radio")
            .iter()
            .map(|(checked, _)| *checked)
            .collect();
        assert_eq!(flags, [false, true, false], "{html}");
    }
}
