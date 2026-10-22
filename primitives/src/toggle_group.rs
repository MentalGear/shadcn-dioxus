//! Defines the [`ToggleGroup`] component and its sub-components, which manage a group of toggle buttons with single or multiple selection.

use crate::{
    collection::{collection_item, use_collection_provider, use_item, CollectionState},
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    merge_attributes,
    toggle::Toggle,
    use_controlled,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::collections::HashSet;

/// The pressed-set key of an item: its `value`, or the decimal `index` when it has no value
/// (the deprecated index API's identity, so `HashSet<usize>` callers keep working).
fn item_key(value: &str, index: usize) -> String {
    if value.is_empty() {
        index.to_string()
    } else {
        value.to_string()
    }
}

fn indices_to_keys(indices: &HashSet<usize>) -> HashSet<String> {
    indices.iter().map(|i| i.to_string()).collect()
}

fn keys_to_indices(keys: &HashSet<String>) -> HashSet<usize> {
    keys.iter().filter_map(|k| k.parse().ok()).collect()
}

#[derive(Clone, Copy)]
struct ToggleGroupCtx {
    // State
    disabled: ReadSignal<bool>,
    pressed: Memo<HashSet<String>>,
    set_pressed: Callback<HashSet<String>>,

    allow_multiple_pressed: ReadSignal<bool>,

    // Focus state
    focus: CollectionState,

    horizontal: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,

    // Text direction, for `ArrowLeft`/`ArrowRight`'s roving-focus role --
    // see `direction::Direction::resolve_horizontal`'s doc.
    direction: ReadSignal<Direction>,
}

impl ToggleGroupCtx {
    fn orientation(&self) -> &'static str {
        match (self.horizontal)() {
            true => "horizontal",
            false => "vertical",
        }
    }

    fn is_pressed(&self, id: &str) -> bool {
        (self.pressed)().contains(id)
    }

    fn set_pressed(&self, id: String, pressed: bool) {
        let mut new_pressed = (self.pressed)();
        match pressed {
            false => new_pressed.remove(&id),
            true => {
                if !(self.allow_multiple_pressed)() {
                    new_pressed.clear();
                }
                new_pressed.insert(id)
            }
        };
        self.set_pressed.call(new_pressed);
    }

    fn is_horizontal(&self) -> bool {
        (self.horizontal)()
    }

    fn focus_next(&mut self) {
        if !(self.roving_loop)() {
            return;
        }

        self.focus.focus_next();
    }

    fn focus_prev(&mut self) {
        if !(self.roving_loop)() {
            return;
        }

        self.focus.focus_prev();
    }
}

/// The props for the [`ToggleGroup`] component
#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupProps {
    /// The values of the items pressed initially when the component is not controlled
    /// (shadcn's `defaultValue`; a single-type group holds at most one).
    #[props(default)]
    pub default_value: HashSet<String>,

    /// The values of the currently pressed items, to drive the group when controlled
    /// (shadcn's `value`).
    pub value: ReadSignal<Option<HashSet<String>>>,

    /// Called with the new set of pressed values (shadcn's `onValueChange`).
    #[props(default)]
    pub on_value_change: Callback<HashSet<String>>,

    /// Deprecated: use `default_value`. Indices of the items pressed initially; an item without
    /// a `value` is keyed by its `index`.
    #[props(default)]
    pub default_pressed: HashSet<usize>,

    /// Deprecated: use `value`. Indices of the pressed items, when controlled.
    pub pressed: ReadSignal<Option<HashSet<usize>>>,

    /// Deprecated: use `on_value_change`. Called with the indices of the pressed items.
    #[props(default)]
    pub on_pressed_change: Callback<HashSet<usize>>,

    /// Whether the toggle group is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// If multiple items can be pressed at the same time. If this is false, only one item can be pressed at a time (radio-style).
    #[props(default)]
    pub allow_multiple_pressed: ReadSignal<bool>,

    /// Whether the toggle group is horizontal or vertical.
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

    /// Additional attributes to apply to the toggle group element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toggle group, which should include multiple [`ToggleItem`] components.
    pub children: Element,
}

/// # ToggleGroup
///
/// The `ToggleGroup` component manages a group of toggle buttons. It supports both single (radio-style) and multiple selection modes with keyboard navigation.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::toggle_group::{ToggleGroup, ToggleItem};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ToggleGroup { horizontal: true, allow_multiple_pressed: true,
///             ToggleItem { index: 0usize, value: "bold", em { "B" } }
///             ToggleItem { index: 1usize, value: "italic", i { "I" } }
///             ToggleItem { index: 2usize, value: "underline", u { "U" } }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The root carries `role="group"` (name it with `aria-label` or `aria-labelledby`).
///
/// The [`ToggleGroup`] component defines the following data attributes you can use to control styling:
/// - `data-orientation`: Indicates the orientation of the toggle group. Values are `horizontal` or `vertical`.
/// - `data-allow-multiple-pressed`: Indicates if multiple items can be pressed at the same time. Values are `true` or `false`.
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
#[component]
pub fn ToggleGroup(props: ToggleGroupProps) -> Element {
    // The index API is a thin adapter over the value set: an index is the key of an item that
    // has no `value`.
    let (controlled, default_set, on_pressed_change, deprecated_pressed) = (
        props.value,
        if props.default_value.is_empty() {
            indices_to_keys(&props.default_pressed)
        } else {
            props.default_value.clone()
        },
        props.on_pressed_change,
        props.pressed,
    );
    let controlled_keys = use_memo(move || {
        controlled
            .cloned()
            .or_else(|| deprecated_pressed.cloned().map(|p| indices_to_keys(&p)))
    });
    let on_value_change = props.on_value_change;
    let (pressed, set_pressed) = use_controlled(
        controlled_keys.into(),
        default_set,
        use_callback(move |keys: HashSet<String>| {
            on_value_change.call(keys.clone());
            on_pressed_change.call(keys_to_indices(&keys));
        }),
    );
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);

    let focus = use_collection_provider(props.roving_loop);
    let mut ctx = use_context_provider(|| ToggleGroupCtx {
        pressed,
        set_pressed,
        allow_multiple_pressed: props.allow_multiple_pressed,
        disabled: props.disabled,

        focus,
        horizontal: props.horizontal,
        roving_loop: props.roving_loop,
        direction: direction_signal,
    });

    let owned = attributes!(div {
        role: "group",
        "data-orientation": ctx.orientation(),
        "data-allow-multiple-pressed": ctx.allow_multiple_pressed,
        "data-direction": direction.as_str(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            dir: direction.as_str(),
            onfocusout: move |_| ctx.focus.clear_focus(),

            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`ToggleItem`] component
#[derive(Props, Clone, PartialEq)]
pub struct ToggleItemProps {
    /// The index of the item within the [`ToggleGroup`]. This orders the items for keyboard
    /// navigation; the pressed state follows `value`, so reordering items never moves it.
    pub index: ReadSignal<usize>,

    /// The item's identity in the group's pressed set (shadcn's item `value`). Unique within the
    /// group. Without one the item is keyed by its `index` (the deprecated index API).
    #[props(default)]
    pub value: ReadSignal<String>,

    /// Whether the toggle item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the toggle item element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toggle item
    pub children: Element,
}

/// # ToggleItem
///
/// An individual toggle button within a [`ToggleGroup`] component.
///
/// This must be used inside a [`ToggleGroup`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::toggle_group::{ToggleGroup, ToggleItem};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         ToggleGroup { horizontal: true, allow_multiple_pressed: true,
///             ToggleItem { index: 0usize, value: "bold", em { "B" } }
///             ToggleItem { index: 1usize, value: "italic", i { "I" } }
///             ToggleItem { index: 2usize, value: "underline", u { "U" } }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`ToggleItem`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the toggle. Values are `on` or `off`.
/// - `data-disabled`: Indicates if the toggle is disabled. Values are `true` or `false`.
/// - `data-orientation`: Indicates the orientation of the toggle group. Values are `horizontal` or `vertical`.
#[component]
pub fn ToggleItem(props: ToggleItemProps) -> Element {
    let mut ctx: ToggleGroupCtx = use_context();

    // We need a kept-alive signal to control the toggle.
    let key = move || item_key(&props.value.cloned(), props.index.cloned());
    let mut pressed = use_signal(|| ctx.is_pressed(&key()));
    use_effect(move || {
        let is_pressed = ctx.is_pressed(&key());
        pressed.set(is_pressed);
    });

    let disabled = move || (ctx.disabled)() || (props.disabled)();
    let item = use_item(
        collection_item(ctx.focus, props.index)
            .disabled(disabled)
            .key(move || Some(key())),
    );
    let tab_index = item.tabindex;
    let onmounted = item.onmounted();

    // `tabindex`/`data-orientation` used to be forwarded as their own
    // ad-hoc keyed props alongside a separately-threaded
    // `attributes: props.attributes.clone()` -- `Toggle` has no typed field
    // for either, so both landed as independent entries in the same
    // forwarded `Vec<Attribute>`, the component-forward shape of backlog
    // row 93's duplicate-attribute hazard (only `tabindex` is in the
    // analyzer's debt register; `data-orientation` is the identical
    // mechanism caught here by construction rather than left as a
    // lookalike). Both are owned (roving-focus wiring / the group's real
    // orientation state), so they win over the caller's attributes.
    let owned = attributes!(button {
        tabindex: tab_index,
        "data-orientation": ctx.orientation(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        Toggle {
            onmounted,
            onfocus: move |_| ctx.focus.set_focus(Some(props.index.cloned())),
            onkeydown: move |event: Event<KeyboardData>| {
                let key = event.key();
                let horizontal = ctx.is_horizontal();
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
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    _ => prevent_default = false,
                };

                if prevent_default {
                    event.prevent_default();
                }
            },

            disabled: disabled(),

            pressed: pressed(),
            on_pressed_change: move |pressed| {
                ctx.set_pressed(key(), pressed);
            },

            attributes: merged,

            {props.children}
        }
    }
}
