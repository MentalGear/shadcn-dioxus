//! Shared state for the combobox component.

use crate::selectable::{OptionState, RcPartialEqValue, SelectableContext};
use dioxus::prelude::*;

/// The default case-insensitive substring filter.
pub fn default_combobox_filter(query: &str, text: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || text.to_lowercase().contains(&query)
}

/// What a combobox holds, by construction.
///
/// `Single` carries nothing but the label: a single-select root has no way to remove one
/// value or clear them all, so no chip, clear button or Backspace path can be
/// reached from it, and its `on_value_change` only ever sees `Option<T>`.
/// `Multi` carries the only two operations that exist on a set of values;
/// they are built next to the root's `Vec<T>` model (`multi.rs`), so a value
/// that reaches a single-select root's callback cannot be a list.
///
/// Both carry `label`, the root's required `item_to_label`: a selected value's display text
/// (the chips and their remove buttons' accessible names; the single input's text). It is read at
/// render time, never from the option registry: the registry fills in a client-only effect, so
/// registry-derived text renders empty in the server HTML until hydration.
#[derive(Clone, Copy)]
pub(super) enum ComboboxModel {
    Single {
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
            ComboboxModel::Single { label } | ComboboxModel::Multi { label, .. } => {
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

    /// The registration index of the selected option, if any. Read from the
    /// option registry rather than the collection: a stale filter from the
    /// previous session may still have the selected option marked hidden
    /// there for a moment after a reopen.
    fn selected_index(&self) -> Option<usize> {
        let options = self.selectable.options.read();
        options
            .iter()
            .filter(|option| self.selectable.is_selected(&option.value))
            .map(|option| option.index)
            .min()
    }

    fn open_session(&mut self, initial_focus: Option<usize>) {
        self.begin_session();
        self.selectable.initial_focus.set(initial_focus);
        self.set_open(true);
    }

    /// Open on a click: the selected option (if any) becomes the active one.
    pub fn open_at_selected(&mut self) {
        let selected = self.selected_index();
        self.open_session(selected);
    }

    /// Open on ArrowDown: the selected option, else the first.
    pub fn open_at_selected_or_first(&mut self) {
        let first = self
            .selectable
            .first_matching_enabled_index(self.predicate_for(String::new()));
        let target = self.selected_index().or(first);
        self.open_session(target);
    }

    /// Open on ArrowUp: the selected option, else the last.
    pub fn open_at_selected_or_last(&mut self) {
        let last = self
            .selectable
            .last_matching_enabled_index(self.predicate_for(String::new()));
        let target = self.selected_index().or(last);
        self.open_session(target);
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

    pub fn focus_last_visible(&mut self) {
        self.selectable.focus_last_where(self.predicate());
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
