//! Shared state and behavior for select-like listbox components.

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::{
    collection::{use_collection_provider, CollectionPlacement, CollectionState},
    listbox::{use_listbox_option, ListboxOptionContext},
    use_controlled, use_unique_id, Controlled,
};

pub(crate) use crate::selection::{OptionState, RcPartialEqValue};

/// Whether selecting an option should replace the current value or toggle it.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SelectionMode {
    /// A single value is selected and the popup closes after selection.
    Single,
    /// Multiple values can be selected and the popup stays open after selection.
    Multiple,
}

impl SelectionMode {
    pub(crate) fn is_multiple(self) -> bool {
        matches!(self, Self::Multiple)
    }

    fn closes_on_select(self) -> bool {
        matches!(self, Self::Single)
    }
}

/// Where an opening listbox puts focus: the (first) selected option, else the
/// `fallback` placement, else nowhere.
///
/// An intent, not an index: the list mounts its options afresh on every open,
/// so they only get their indices, and their document order (row 396's
/// `crate::collection::use_item_index`), once the list is open.
/// [`crate::listbox::use_listbox_container`] resolves it then.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ListboxFocus {
    pub(crate) selected: bool,
    pub(crate) fallback: Option<CollectionPlacement>,
}

impl ListboxFocus {
    pub(crate) const SELECTED: Self = Self {
        selected: true,
        fallback: None,
    };
    pub(crate) const FIRST: Self = Self {
        selected: false,
        fallback: Some(CollectionPlacement::First),
    };
    pub(crate) const LAST: Self = Self {
        selected: false,
        fallback: Some(CollectionPlacement::Last),
    };
    pub(crate) const SELECTED_OR_FIRST: Self = Self {
        selected: true,
        fallback: Some(CollectionPlacement::First),
    };
    pub(crate) const SELECTED_OR_LAST: Self = Self {
        selected: true,
        fallback: Some(CollectionPlacement::Last),
    };
}

/// Shared context for components built around a selectable listbox.
#[derive(Clone, Copy)]
pub(crate) struct SelectableContext {
    pub(crate) open: Memo<bool>,
    pub(crate) set_open: Callback<bool>,
    pub(crate) values: Memo<Vec<RcPartialEqValue>>,
    pub(crate) set_value: Callback<RcPartialEqValue>,
    pub(crate) selection_mode: SelectionMode,
    pub(crate) options: Signal<Vec<OptionState>>,
    pub(crate) list_id: Signal<Option<String>>,
    pub(crate) collection: CollectionState,
    pub(crate) initial_focus: Signal<Option<ListboxFocus>>,
    pub(crate) disabled: ReadSignal<bool>,

    /// Id of the trigger button, for `use_refocus_on_close_unless` (lib.rs)
    /// to return focus to on close. See dev-docs/plan.md Phase 3.1.
    pub(crate) trigger_id: Signal<String>,

    /// Set just before a close caused by focus leaving the listbox (see the
    /// `onblur` handler in `SelectList`), so the refocus hook knows to leave
    /// focus where the user put it instead of yanking it back to the
    /// trigger. Reset to `false` whenever the popup opens.
    pub(crate) interacted_outside: Signal<bool>,
}

#[derive(Clone, Copy)]
pub(crate) struct SelectableOption<T: Clone + PartialEq + 'static> {
    pub(crate) id: Memo<String>,
    pub(crate) disabled: Memo<bool>,
    pub(crate) selected: Memo<bool>,
    pub(crate) focused: Memo<bool>,
    pub(crate) down_pos: Signal<Option<ItemPress>>,
    pub(crate) index: ReadSignal<usize>,
    pub(crate) value: ReadSignal<T>,
}

pub(crate) struct SelectableOptionConfig<T: Clone + PartialEq + 'static> {
    pub(crate) id: ReadSignal<Option<String>>,
    pub(crate) index: ReadSignal<usize>,
    pub(crate) value: ReadSignal<T>,
    pub(crate) text_value: ReadSignal<Option<String>>,
    pub(crate) option_disabled: ReadSignal<bool>,
    pub(crate) component_name: &'static str,
}

impl SelectableContext {
    pub(crate) fn set_open(&mut self, open: bool) {
        self.set_open.call(open);
    }

    pub(crate) fn is_selected(&self, value: &RcPartialEqValue) -> bool {
        self.values.read().iter().any(|selected| selected == value)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.read().is_empty()
    }

    /// The option `focus` resolves to among the registered options: the
    /// first available selected one, else the fallback placement. Tracked.
    pub(crate) fn initial_focus_target(&self, focus: ListboxFocus) -> Option<usize> {
        let selected = focus
            .selected
            .then(|| {
                self.options
                    .read()
                    .iter()
                    .filter(|option| {
                        self.is_selected(&option.value)
                            && self.collection.is_available(option.index)
                    })
                    .map(|option| option.index)
                    .min()
            })
            .flatten();
        selected.or_else(|| match focus.fallback? {
            CollectionPlacement::First => self.collection.first_available_index(),
            CollectionPlacement::Last => self.collection.last_available_index(),
        })
    }

    pub(crate) fn focused_option_id(&self) -> Option<String> {
        self.collection.focused_key()
    }

    pub(crate) fn select_focused(&mut self) {
        if !self.open.cloned() {
            return;
        }
        let Some(index) = self.collection.focused_index() else {
            return;
        };
        if !self.collection.is_available(index) {
            return;
        }
        let value = self
            .options
            .read()
            .iter()
            .find(|option| option.index == index)
            .map(|option| option.value.clone());
        if let Some(value) = value {
            self.select_value(value);
        }
    }

    fn matching_enabled_indices(&self, predicate: impl Fn(&OptionState) -> bool) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .options
            .read()
            .iter()
            .filter(|option| self.collection.is_available(option.index) && predicate(option))
            .map(|option| option.index)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    pub(crate) fn first_matching_enabled_index(
        &self,
        predicate: impl Fn(&OptionState) -> bool,
    ) -> Option<usize> {
        self.matching_enabled_indices(predicate).first().copied()
    }

    pub(crate) fn last_matching_enabled_index(
        &self,
        predicate: impl Fn(&OptionState) -> bool,
    ) -> Option<usize> {
        self.matching_enabled_indices(predicate).last().copied()
    }

    pub(crate) fn focus_next_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let options = self.options;
        self.collection.focus_next_matching(move |index| {
            options
                .read()
                .iter()
                .any(|option| option.index == index && predicate(option))
        });
    }

    pub(crate) fn focus_prev_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let options = self.options;
        self.collection.focus_prev_matching(move |index| {
            options
                .read()
                .iter()
                .any(|option| option.index == index && predicate(option))
        });
    }

    pub(crate) fn focus_first_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let index = self.first_matching_enabled_index(predicate);
        self.collection.set_focus(index);
    }

    pub(crate) fn focus_last_where(&mut self, predicate: impl Fn(&OptionState) -> bool) {
        let index = self.last_matching_enabled_index(predicate);
        self.collection.set_focus(index);
    }

    pub(crate) fn select_value(&mut self, value: RcPartialEqValue) {
        self.set_value.call(value);
        if self.selection_mode.closes_on_select() {
            self.set_open(false);
        }
    }
}

/// The model of a single-select root, from [`use_single_selectable_value`].
pub(crate) struct SingleSelectableValue {
    /// The selection as the selectable's list: empty or one value.
    pub(crate) values: Memo<Vec<RcPartialEqValue>>,
    pub(crate) set_value: Callback<RcPartialEqValue>,
    /// Restores `default_value` (a `<form reset>`).
    pub(crate) reset_to_default: Callback<()>,
    /// Selects nothing.
    pub(crate) clear: Callback<()>,
}

pub(crate) fn use_single_selectable_value<T: Clone + PartialEq + 'static>(
    controlled_value: Option<ReadSignal<Option<T>>>,
    default_value: Option<T>,
    on_change: Callback<Option<T>>,
    component_name: &'static str,
) -> SingleSelectableValue {
    let mut internal_value: Signal<Option<T>> = use_signal(|| default_value.clone());
    let value = use_memo(move || match controlled_value {
        Some(value) => value.cloned(),
        None => internal_value.cloned(),
    });
    let values = use_memo(move || value().map(RcPartialEqValue::new).into_iter().collect());
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let value = incoming
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("{component_name} and option value types must match"))
            .clone();
        internal_value.set(Some(value.clone()));
        on_change.call(Some(value));
    });

    // Resets the (Dioxus-side) selection back to `default_value`. Native
    // `<form reset>` only restores the hidden mirror `<select>` this crate
    // renders (see select.rs) -- it has no way to reach this signal, so
    // callers wire this into a `use_form_reset_listener` themselves.
    let default_for_reset = default_value.clone();
    let reset_to_default = use_callback(move |()| {
        internal_value.set(default_for_reset.clone());
        on_change.call(default_for_reset.clone());
    });
    let clear = use_callback(move |()| {
        internal_value.set(None);
        on_change.call(None);
    });

    SingleSelectableValue {
        values,
        set_value,
        reset_to_default,
        clear,
    }
}

pub(crate) fn use_selectable_root(
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    selection_mode: SelectionMode,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
) -> SelectableContext {
    let (open, set_open) = use_controlled(open.value, open.default.cloned(), open.on_change);
    let options: Signal<Vec<OptionState>> = use_signal(Vec::default);
    let list_id = use_signal(|| None);
    let collection = use_collection_provider(roving_loop);
    let initial_focus = use_signal(|| None);
    let trigger_id = use_unique_id();
    let mut interacted_outside = use_signal(|| false);

    // A fresh open shouldn't inherit an `interacted_outside` flag left over
    // from a previous close (see dev-docs/plan.md Phase 3.1).
    use_effect(move || {
        if open() {
            interacted_outside.set(false);
        }
    });

    SelectableContext {
        open,
        set_open,
        values,
        set_value,
        selection_mode,
        options,
        list_id,
        collection,
        initial_focus,
        disabled,
        trigger_id,
        interacted_outside,
    }
}

pub(crate) fn use_selectable_option<T: Clone + PartialEq + 'static>(
    selectable: SelectableContext,
    option: SelectableOptionConfig<T>,
) -> SelectableOption<T> {
    let SelectableOptionConfig {
        id,
        index,
        value,
        text_value,
        option_disabled,
        component_name,
    } = option;
    let disabled = {
        let root_disabled = selectable.disabled;
        use_memo(move || root_disabled.cloned() || option_disabled.cloned())
    };
    let id = use_listbox_option(
        id,
        index,
        value,
        text_value,
        selectable.options,
        component_name,
    );
    let selected = use_memo(move || selectable.is_selected(&RcPartialEqValue::new(value.cloned())));
    let focused = use_memo(move || selectable.collection.is_focused(index()));
    let down_pos: Signal<Option<ItemPress>> = use_signal(|| None);

    use_context_provider(|| ListboxOptionContext {
        selected: selected.into(),
    });

    SelectableOption {
        id,
        disabled,
        selected,
        focused,
        down_pos,
        index,
        value,
    }
}

/// A press on a selectable item, recorded on `pointerdown` and committed on release.
///
/// A mouse press commits on `pointerup`. A touch or pen press commits on the `click` that follows
/// the release instead: committing closes the popup, and on a touch screen the browser fires that
/// `click` after the release, hit-tested at the tap point, so a popup already closed on `pointerup`
/// would hand the tap to whatever lies under it (on iOS, a native `<select>` below the list opens).
/// Committing on the `click` consumes it on the item itself. A touch that turns into a scroll gets
/// a `pointercancel` and no `click`, so it never selects.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ItemPress {
    x: f64,
    y: f64,
    mouse: bool,
}

/// The `pointerdown` half of an item press: records it, so only a press that started on this
/// item can select it (a long-press that opened the menu and lifts over an item does not).
pub(crate) fn pointer_select_start(
    event: &Event<PointerData>,
    disabled: bool,
    mut press: Signal<Option<ItemPress>>,
) {
    if disabled || event.trigger_button() != Some(MouseButton::Primary) {
        return;
    }
    event.prevent_default();
    let p = event.client_coordinates();
    press.set(Some(ItemPress {
        x: p.x,
        y: p.y,
        mouse: event.pointer_type() == "mouse",
    }));
}

/// The `pointerup` half: `true` when a mouse press on this item ends here and selects it. A touch
/// or pen press stays recorded for [`pointer_select_click`].
pub(crate) fn pointer_select_commit(
    event: &Event<PointerData>,
    disabled: bool,
    mut press: Signal<Option<ItemPress>>,
) -> bool {
    if disabled || event.trigger_button() != Some(MouseButton::Primary) {
        return false;
    }
    match press() {
        Some(p) if p.mouse => {
            press.set(None);
            true
        }
        Some(p) => {
            let up = event.client_coordinates();
            let (dx, dy) = (up.x - p.x, up.y - p.y);
            if dx * dx + dy * dy > 25.0 {
                press.set(None);
            }
            false
        }
        None => false,
    }
}

/// The `click` half: `true` when a touch or pen press on this item ends here and selects it.
pub(crate) fn pointer_select_click(disabled: bool, mut press: Signal<Option<ItemPress>>) -> bool {
    match press.take() {
        Some(p) => !p.mouse && !disabled,
        None => false,
    }
}

pub(crate) fn pointer_select_cancel(mut press: Signal<Option<ItemPress>>) {
    press.set(None);
}
