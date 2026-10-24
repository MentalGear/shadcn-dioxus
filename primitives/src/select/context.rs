//! Context types and implementations for the select component.

use crate::sleep::sleep;
use dioxus::prelude::*;
use dioxus_core::Task;

use std::time::Duration;

use super::text_search::AdaptiveKeyboard;
use crate::direction::Direction;
use crate::selectable::{RcPartialEqValue, SelectableContext};

/// Where a [`Select`](super::components::select::Select)'s listbox is placed (Radix's `position`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectPosition {
    /// Anchored below the trigger (flipping above when it does not fit): the popper placement,
    /// shadcn's current default.
    #[default]
    Popper,
    /// The selected option sits over the trigger and the listbox stays inside the viewport:
    /// Radix's `item-aligned` mode.
    ItemAligned,
}

impl SelectPosition {
    /// The `data-position` value the listbox carries.
    pub fn as_str(self) -> &'static str {
        match self {
            SelectPosition::Popper => "popper",
            SelectPosition::ItemAligned => "item-aligned",
        }
    }
}

/// Main context for the select component containing all shared state
#[derive(Clone, Copy)]
pub(super) struct SelectContext {
    /// Shared selectable listbox state.
    pub selectable: SelectableContext,
    /// A selected value's display text: the root's required `item_to_label`.
    ///
    /// Read at render time, never from the option registry (which fills in a client-only
    /// effect and is empty in the server HTML): see [`Self::selected_text`].
    pub label: Callback<RcPartialEqValue, String>,
    /// Adaptive keyboard system for multi-language support
    pub adaptive_keyboard: Signal<AdaptiveKeyboard>,
    /// The typeahead buffer for searching options
    pub typeahead_buffer: Signal<String>,
    /// The ID of the list for ARIA attributes
    pub typeahead_clear_task: Signal<Option<Task>>,
    /// Timeout before clearing typeahead buffer
    pub typeahead_timeout: ReadSignal<Duration>,

    /// Set for the duration of an Alt+ArrowDown-opened popup (APG select-
    /// only combobox, Optional: "Alt + Down Arrow: ... displays the popup
    /// without moving focus"), so `SelectListRendered`'s own "nothing is
    /// focused yet -- focus the listbox container" fallback (`list.rs`)
    /// does not steal DOM focus off the trigger for this one open path.
    /// Reset to `false` whenever the popup closes (`select.rs`), so it can
    /// never leak into the next, differently-triggered open.
    pub keep_trigger_focus: Signal<bool>,

    /// Mirrors [`super::components::select::SelectProps::required`] into the
    /// context so `SelectTrigger` (`trigger.rs`) can expose it as
    /// `aria-required` on the combobox-role trigger (dev-docs/backlog.md row 8).
    /// `SelectMulti` has no `required` prop of its own (see that prop's doc
    /// in `select.rs`) -- `use_select_root` wires a constant `false` signal
    /// in for it, so this field is always populated regardless of which of
    /// the two root components provided this context.
    pub required: ReadSignal<bool>,

    /// Mirrors [`super::components::select::SelectProps::scroll_lock`] (and
    /// `SelectMultiProps::scroll_lock`) into the context so
    /// `SelectListRendered` (`list.rs`) can gate its `ScrollLockGuard` on it
    /// (dev-docs/backlog.md row 9). Defaults to `true` (Radix parity) via both
    /// props' own `#[props(default = ...)]`.
    pub scroll_lock: ReadSignal<bool>,

    /// Text direction, emitted as `dir`/`data-direction` on the trigger and
    /// the listbox content. Neither this crate's trigger nor its listbox
    /// has an `ArrowLeft`/`ArrowRight`-driven navigation to flip (both are
    /// vertical-only, matching the APG select-only combobox pattern) --
    /// Radix's own RTL-specific positioning math is for its "item-aligned"
    /// mode and does not apply to this crate's anchor-positioned popper.
    pub direction: ReadSignal<Direction>,

    /// Where the listbox is placed: see [`SelectPosition`].
    pub position: ReadSignal<SelectPosition>,
}

impl SelectContext {
    pub fn set_open(&mut self, open: bool) {
        self.selectable.set_open(open);
    }

    /// The path Enter and Space on `SelectTrigger` both route through: open
    /// the popup and request focus on the currently-selected option, or the
    /// first available one if nothing is selected yet. APG select-only
    /// combobox: "focus the listbox with the current option active" --
    /// unlike a plain click open (which leaves DOM focus on the
    /// listbox container itself, per `select.spec.ts`), keyboard activation must
    /// land real DOM focus on an option, same as ArrowDown/ArrowUp already
    /// do just below this in `trigger.rs`.
    pub fn open_with_selected_or_first_focus(&mut self) {
        self.set_open(true);
        let target = self
            .selectable
            .collection
            .selected_available_index()
            .or_else(|| self.selectable.collection.first_available_index());
        self.selectable.initial_focus.set(target);
    }

    /// The pointer (click or tap) open: the currently selected option is highlighted and focused,
    /// and scrolled into view, exactly as a keyboard open does (Base UI: a Select opens on its
    /// `selectedIndex` for every modality; a Menu, by contrast, highlights its first item for a
    /// keyboard open only). With nothing selected, or in a multi-select, a pointer open stays as it was: no highlight,
    /// DOM focus on the listbox container.
    pub fn open_for_pointer(&mut self) {
        // Single-select only: a multi-select has no one selected option to open on, and its
        // pointer open keeps its focus on the listbox (ArrowDown then reaches the first option).
        let target = (!self.multi())
            .then(|| self.selectable.collection.selected_available_index())
            .flatten();
        if target.is_some() {
            self.selectable.initial_focus.set(target);
        }
        self.set_open(true);
    }

    /// The selected values' text in selection order, joined by `", "`; `None` while nothing is selected.
    ///
    /// Built from the root's `item_to_label`, so it is the same on the server and the client and no
    /// registry state can blank it.
    pub fn selected_text(&self) -> Option<String> {
        let values = self.selectable.values.read();
        let parts: Vec<String> = values
            .iter()
            .map(|value| self.label.call(value.clone()))
            .collect();
        (!parts.is_empty()).then(|| parts.join(", "))
    }

    pub fn multi(&self) -> bool {
        self.selectable.selection_mode.is_multiple()
    }

    /// Select the currently focused item
    pub fn select_current_item(&mut self) {
        self.selectable.select_focused();
    }

    /// Learn from a keyboard event mapping physical key to logical character
    pub fn learn_from_keyboard_event(&mut self, physical_code: &str, logical_char: char) {
        let mut adaptive = self.adaptive_keyboard.write();
        let logical_char = logical_char.to_lowercase().next().unwrap_or(logical_char);
        adaptive.learn_from_event(physical_code, logical_char);
    }

    /// Add text to the typeahead buffer for searching
    pub fn add_to_typeahead_buffer(&mut self, text: &str) {
        // Cancel any existing clear task to prevent race conditions
        if let Some(existing_task) = self.typeahead_clear_task.write().take() {
            existing_task.cancel();
        }

        // Update the buffer and get the current content
        let typeahead = {
            let mut typeahead_buffer = self.typeahead_buffer.write();
            typeahead_buffer.push_str(text);
            typeahead_buffer.clone()
        };

        // Create references for the async closure
        let mut typeahead_buffer_signal = self.typeahead_buffer;
        let mut typeahead_clear_task_signal = self.typeahead_clear_task;

        // Spawn a new task to clear the buffer after the configured timeout
        let timeout = self.typeahead_timeout.cloned();
        let new_task = spawn(async move {
            sleep(timeout).await;

            // Clear the buffer
            typeahead_buffer_signal.write().clear();

            // Remove our own task handle to indicate no task is active
            typeahead_clear_task_signal.write().take();
        });

        // Store the new task handle
        self.typeahead_clear_task.write().replace(new_task);

        // Focus the best match using adaptive keyboard
        let options = self.selectable.options.read();
        let keyboard = self.adaptive_keyboard.read();

        if let Some(best_match_index) =
            super::text_search::best_match(&keyboard, &typeahead, &options, |index| {
                self.selectable.collection.is_available(index)
            })
        {
            self.selectable.collection.set_focus(Some(best_match_index));
        }
    }
}

/// Context for select group components
#[derive(Clone, Copy)]
pub(super) struct SelectGroupContext {
    /// ID of the element that labels this group
    pub labeled_by: Signal<Option<String>>,
}
