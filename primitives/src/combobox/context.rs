//! Shared state for the combobox component.

use crate::selectable::{ListboxFocus, OptionState, RcPartialEqValue, SelectableContext};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

/// The mounted elements focus moves between: the input, and a multi-select's chip remove buttons
/// by chip index (`ComboboxChips` writes them, the input and the clear button read).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxHandles {
    pub input: Signal<Option<Rc<MountedData>>>,
    pub chips: Signal<HashMap<usize, Rc<MountedData>>>,
}

impl ComboboxHandles {
    /// Move DOM focus to the input (a no-op until it has mounted).
    pub fn focus_input(self) {
        if let Some(input) = self.input.peek().clone() {
            spawn(async move {
                // focus-ok: the field's own controls (a chip, the clear button, its padding) hand focus to the input
                _ = input.set_focus(true).await;
            });
        }
    }

    /// Move focus one chip toward `prev` (or from the input, when `current` is `None`): from the
    /// input `prev` lands on the last of `count` chips; stepping past the last chip lands on the
    /// input.
    pub fn focus_chip(self, current: Option<usize>, prev: bool, count: usize) {
        let at = current.unwrap_or(count);
        let next = if prev { at.saturating_sub(1) } else { at + 1 };
        let target = if next >= count {
            self.input.peek().clone()
        } else {
            self.chips.peek().get(&next).cloned()
        };
        if let Some(target) = target {
            spawn(async move {
                // focus-ok: arrow keys walk the chip remove buttons and the input (the field's roving focus)
                _ = target.set_focus(true).await;
            });
        }
    }
}

/// The default case-insensitive substring filter.
pub fn default_combobox_filter(query: &str, text: &str) -> bool {
    crate::data_view::contains_query(text, query)
}

/// What a combobox holds, by construction.
///
/// `Single` holds at most one value, so the only operation besides picking is clearing it (the
/// clear button); there are no chips and no Backspace removal, and its `on_value_change` only ever
/// sees `Option<T>`. `Multi` carries the two operations that exist on a set of values; they are
/// built next to the root's `Vec<T>` model (`multi.rs`), so a value that reaches a single-select
/// root's callback cannot be a list.
///
/// Both carry `label`, the root's required `item_to_label`: a selected value's display text
/// (the chips and their remove buttons' accessible names; the single input's text). It is read at
/// render time, never from the option registry: the registry fills in a client-only effect, so
/// registry-derived text renders empty in the server HTML until hydration.
#[derive(Clone, Copy)]
pub(super) enum ComboboxModel {
    Single {
        /// Select nothing (the clear button).
        clear: Callback<()>,
        label: Callback<RcPartialEqValue, String>,
    },
    Multi {
        /// Remove one selected value (a chip's remove button, Backspace).
        remove: Callback<RcPartialEqValue>,
        /// Remove every selected value (the clear button).
        clear: Callback<()>,
        label: Callback<RcPartialEqValue, String>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct ComboboxContext {
    pub model: ComboboxModel,
    pub selectable: SelectableContext,
    pub query: Memo<String>,
    pub set_query: Callback<String>,
    pub filter: Callback<(String, String), bool>,

    /// The current `ComboboxInput`'s own element id, kept in sync by that
    /// component -- mirrors
    /// `DropdownMenuContext::content_id` (`dropdown_menu.rs`), except this
    /// one holds the *trigger* side's id rather than the content side's:
    /// `ComboboxList`'s listbox anchors to the input (there is no separate
    /// trigger button here), and the input's id -- unlike `SelectTrigger`'s
    /// already-stable `trigger_id` -- is generated locally by
    /// `ComboboxInput` itself (`use_id_or`/`use_unique_id`), so it needs
    /// this same sync-back to be visible to the listbox's anchor-name
    /// wiring. See `PopoverCtx::content_id`'s doc in `popover.rs` for the
    /// exact bug this guards against if the two ever named different ids.
    pub input_id: Signal<String>,

    /// The mounted input and chip buttons, for moving focus between them.
    pub handles: ComboboxHandles,

    /// Whether the user has edited the input's text since the popup opened.
    ///
    /// This is what separates *engaging* the input from *querying* with it.
    /// Opening the popup (a click, ArrowDown/ArrowUp, or a controlled `open`)
    /// must leave the input's text alone -- it is the selected option's label
    /// -- and list every option; only the first edit turns that text into a
    /// query (`input_text`, `filter_query`). Reset by [`Self::open_session`]
    /// for every user-initiated open and by `ComboboxList` once the popup has
    /// finished closing (not at close: the closing list keeps its filter
    /// through its exit animation).
    pub edited: Signal<bool>,
}

impl ComboboxContext {
    pub fn is_multi(&self) -> bool {
        matches!(self.model, ComboboxModel::Multi { .. })
    }

    /// Pick an option: the selectable's own select (which closes a
    /// single-select popup) plus, for a multi-select, ending the text edit --
    /// the picked option's text is spent, so the next pick starts from the
    /// full list. The popup stays open.
    pub fn select_value(&mut self, value: RcPartialEqValue) {
        self.selectable.select_value(value);
        if self.is_multi() {
            self.reset_query();
        }
    }

    pub fn reset_query(&mut self) {
        if *self.edited.peek() {
            self.edited.set(false);
        }
        if !self.query.peek().is_empty() {
            self.set_query.call(String::new());
        }
    }

    /// Clear the selection (the clear button). A single-select's input empties with it: the edit is
    /// over, so it shows the (now absent) selection's label.
    pub fn clear(&mut self) {
        let (ComboboxModel::Single { clear, .. } | ComboboxModel::Multi { clear, .. }) = self.model;
        clear.call(());
        self.reset_query();
    }

    /// Remove the last selected value (Backspace on an empty input). No-op for `Single`.
    pub fn remove_last(&self) {
        let last = self.selectable.values.peek().last().cloned();
        if let (ComboboxModel::Multi { remove, .. }, Some(last)) = (self.model, last) {
            remove.call(last);
        }
    }

    /// A selected value's text: the root's `item_to_label`, available at render time.
    pub fn label_of(&self, value: &RcPartialEqValue) -> String {
        match self.model {
            ComboboxModel::Single { label, .. } | ComboboxModel::Multi { label, .. } => {
                label.call(value.clone())
            }
        }
    }

    pub fn set_open(&mut self, open: bool) {
        if open {
            self.selectable.collection.clear_focus();
        }
        self.selectable.set_open(open);
    }

    fn predicate_for(&self, query: String) -> impl Fn(&OptionState) -> bool {
        let filter = self.filter;
        move |option| filter.call((query.clone(), option.text_value.clone()))
    }

    /// The selected value's label, from the root's `item_to_label` (never the option registry, so it
    /// is present in the server HTML and cannot flash empty while the list's options re-register).
    fn selected_label(&self) -> Option<String> {
        // A multi-select shows its values as chips; the input is only ever the query.
        if self.is_multi() {
            return None;
        }
        let value = self.selectable.values.read().first().cloned()?;
        Some(self.label_of(&value))
    }

    /// The text the options are filtered by: the query once the user has
    /// edited the input (or when there is no selected label to show), and
    /// otherwise nothing, so a popup opened over a selection lists every option
    /// instead of only the one matching its own label.
    fn filter_query(&self) -> String {
        if *self.edited.read() || self.selected_label().is_none() {
            self.query.cloned()
        } else {
            String::new()
        }
    }

    fn predicate(&self) -> impl Fn(&OptionState) -> bool {
        self.predicate_for(self.filter_query())
    }

    /// What the input shows. The user's query while they are editing it; the
    /// selected option's label otherwise -- including while the popup is open
    /// and untouched (clicking or focusing must not change the text), and the
    /// instant it closes (Escape, blur or a pick restore the label).
    pub fn input_text(&self) -> String {
        let open = (self.selectable.open)();
        match self.selected_label() {
            Some(label) if !(open && *self.edited.read()) => label,
            _ if open => self.query.cloned(),
            _ => String::new(),
        }
    }

    pub fn is_visible(&self, tab_index: usize) -> bool {
        let predicate = self.predicate();
        self.selectable
            .options
            .read()
            .iter()
            .find(|option| option.index == tab_index)
            .is_some_and(predicate)
    }

    pub fn has_visible_options(&self) -> bool {
        self.selectable.options.read().iter().any(self.predicate())
    }

    /// Start a fresh editing session: no edit yet, no query.
    fn begin_session(&mut self) {
        self.reset_query();
    }

    fn open_session(&mut self, initial_focus: ListboxFocus) {
        self.begin_session();
        self.selectable.initial_focus.set(Some(initial_focus));
        self.set_open(true);
    }

    /// Open on a click: the selected option (if any) becomes the active one.
    pub fn open_at_selected(&mut self) {
        self.open_session(ListboxFocus::SELECTED);
    }

    /// Open on ArrowDown: the selected option, else the first.
    pub fn open_at_selected_or_first(&mut self) {
        self.open_session(ListboxFocus::SELECTED_OR_FIRST);
    }

    /// Open on ArrowUp: the selected option, else the last.
    pub fn open_at_selected_or_last(&mut self) {
        self.open_session(ListboxFocus::SELECTED_OR_LAST);
    }

    pub fn focused_option_id(&self) -> Option<String> {
        self.selectable.focused_option_id()
    }

    pub fn focus_next_visible(&mut self) {
        self.selectable.focus_next_where(self.predicate());
    }

    pub fn focus_prev_visible(&mut self) {
        self.selectable.focus_prev_where(self.predicate());
    }

    pub fn focus_first_visible(&mut self) {
        self.selectable.focus_first_where(self.predicate());
    }

    pub fn select_focused(&mut self) {
        // The values memo has not recomputed yet at this point, so "did the pick change them" cannot
        // be read back; a highlighted option is what makes Enter a pick.
        let picks = self.selectable.collection.focused_index().is_some();
        self.selectable.select_focused();
        if self.is_multi() && picks {
            self.reset_query();
        }
    }
}
