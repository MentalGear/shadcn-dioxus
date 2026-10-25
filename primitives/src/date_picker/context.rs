use super::*;

/// The context provided by the [`DatePicker`] component to its children.
#[derive(Copy, Clone)]
pub(super) struct BaseDatePickerContext {
    // State
    pub(super) open: Signal<bool>,
    pub(super) read_only: ReadSignal<bool>,

    // Configuration
    pub(super) disabled: ReadSignal<bool>,
    pub(super) focus: CollectionState,
    pub(super) enabled_date_range: DateRange,
    pub(super) available_ranges: Memo<AvailableRanges>,

    // What happens to the popover once a selection is complete -- see
    // `BaseDatePickerContext::selection_complete`.
    pub(super) close_on_select: ReadSignal<bool>,
    pub(super) close_delay: ReadSignal<Duration>,
    pub(super) dwell: CloseDwell,

    // Apply/Cancel staging -- see [`DatePickerConfirm`].
    pub(super) confirm: DatePickerConfirm,
}

/// How long a picker stays open after a selection is complete, by default.
///
/// The popover closes on selection, so without a pause the only evidence that the click worked
/// is the popover vanishing. The pause lets the selected cell's own paint land first (the day's
/// fill transition is 100-200 ms, `--dx-motion-duration-fast`..`-slow`) and be registered, and
/// only then does the normal exit animation (150-200 ms) run. 300 ms is the shortest value that
/// reliably clears both: a visual change needs roughly 100-150 ms to be noticed at all, and the
/// day's own transition has to finish inside it; anything past ~400 ms starts to read as lag
/// instead of confirmation (the popover looks stuck). Pass `Duration::ZERO` to close
/// immediately.
pub const DEFAULT_CLOSE_DELAY: Duration = Duration::from_millis(300);

/// Hand keyboard focus back to the trigger that opened the popover, if focus is still inside it.
///
/// Closing the popover from state (`open = false`) unmounts its content once the exit animation is
/// done, and a focused element that is removed drops focus on `<body>` -- a keyboard user who just
/// picked a date with Enter lands at the top of the page. (The browser's own light dismiss -- Escape,
/// an outside click -- restores it natively; a close that comes from state does not.) The popover
/// content is labelled by its trigger (`aria-labelledby` is the trigger's id), so the trigger can
/// be found from wherever focus is without this crate knowing its element. A no-op when focus is
/// not inside a popover, so it never steals focus from somewhere the user moved it to.
pub(super) fn return_focus_to_trigger() {
    let _ = dioxus::document::eval(
        r#"const open = document.activeElement && document.activeElement.closest('[popover][aria-labelledby], dialog[aria-labelledby]');
        const trigger = open && document.getElementById(open.getAttribute('aria-labelledby'));
        // focus-ok: restore to the trigger; not after a touch or pen (modality.rs: a script-focused trigger rings on iOS)
        if (trigger && !/^(touch|pen)$/.test(document.documentElement.getAttribute('data-dx-modality') || '')) trigger.focus();"#,
    );
}

/// What a completed selection does to the popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectionClose {
    /// `close_on_select: false` -- the popover stays open.
    Stay,
    /// `close_delay` is zero -- close in the same turn as the selection.
    Immediately,
    /// Hold the popover open for the dwell, then close it.
    After(Duration),
}

pub(super) fn selection_close(close_on_select: bool, close_delay: Duration) -> SelectionClose {
    if !close_on_select {
        SelectionClose::Stay
    } else if close_delay.is_zero() {
        SelectionClose::Immediately
    } else {
        SelectionClose::After(close_delay)
    }
}

/// The one pending "close after the dwell" timer of a picker.
///
/// Scope-owned, following `crate::interval::use_interval`: the timer is a Dioxus task spawned
/// in the PICKER's own scope (not whichever scope happened to call [`Self::schedule`] -- the
/// calendar inside the popover unmounts when the popover does), so it is dropped with the picker
/// and there is no JS `setTimeout` for anything to forget to clear (`crate::sleep::sleep`:
/// `tokio` on native, `gloo_timers` on wasm, whose timeout is cleared when its future is
/// dropped). At most one is pending at a time: scheduling replaces, and [`Self::cancel`] drops
/// it, so every interaction, reopen or new selection during the dwell restarts or cancels it
/// instead of racing it.
///
/// The wait is counted from the selection, not from when the selected day is painted: no effect
/// can be tied to the render of one particular day cell (the cell renders in a later pass than
/// the picker's own), and on ordinary hardware the difference is a frame. `close_delay` is the
/// time between the click and the start of the exit animation.
#[derive(Clone, Copy)]
pub(super) struct CloseDwell {
    pub(super) scope: ScopeId,
    pub(super) task: Signal<Option<Task>>,
}

/// Create the picker's [`CloseDwell`] and wire it to the popover's `open` signal: the pending
/// close is cancelled whenever the popover opens or closes, so a close scheduled for one open
/// session can never land on the next one (Escape, an outside click or the trigger during the
/// dwell, then a reopen before it elapses). A hook: call it unconditionally, once per render,
/// from the picker root.
pub(super) fn use_close_dwell(open: Signal<bool>) -> CloseDwell {
    let dwell = CloseDwell {
        scope: current_scope_id(),
        task: use_signal(|| None),
    };
    use_effect(move || {
        let _ = open();
        dwell.cancel();
    });
    dwell
}

impl CloseDwell {
    /// Drop the pending close, if any.
    pub(super) fn cancel(mut self) {
        let task = self.task.write().take();
        if let Some(task) = task {
            task.cancel();
        }
    }

    /// Whether a close is currently waiting out its dwell.
    pub(super) fn is_pending(self) -> bool {
        self.task.peek().is_some()
    }

    /// Run `close` after `delay`, replacing whatever was already pending.
    pub(super) fn schedule(mut self, delay: Duration, close: impl FnOnce() + 'static) {
        self.cancel();
        let mut slot = self.task;
        let task = Runtime::current().spawn(self.scope, async move {
            sleep(delay).await;
            slot.set(None);
            close();
        });
        self.task.set(Some(task));
    }
}

impl BaseDatePickerContext {
    /// A selection has just been completed (a date picked, or a range's end date picked): close
    /// the popover as `close_on_select`/`close_delay` say. The selected state is already painted
    /// by the time a dwell elapses, the popover's own exit animation runs after it, and focus
    /// returns to the trigger exactly as it does for any other close.
    pub(super) fn selection_complete(self) {
        let mut open = self.open;
        match selection_close(*self.close_on_select.peek(), *self.close_delay.peek()) {
            SelectionClose::Stay => self.dwell.cancel(),
            SelectionClose::Immediately => {
                self.dwell.cancel();
                return_focus_to_trigger();
                open.set(false);
            }
            SelectionClose::After(delay) => self.dwell.schedule(delay, move || {
                return_focus_to_trigger();
                open.set(false);
            }),
        }
    }
}

/// The Apply/Cancel handle of a picker, from [`use_date_picker_confirm`].
///
/// With `confirm: true` a picker does not commit what the calendar selects: the selection is
/// STAGED inside the panel (the calendar shows it, the input and `on_value_change` keep the
/// committed value) until [`Self::apply`] commits it and closes the popover. [`Self::cancel`], and
/// every other way of closing -- Escape, an outside click, the trigger -- drop the staged selection
/// and leave the committed value as it was. A picker built from the primitives renders its own
/// footer from this handle; the styled `DatePicker` renders `Cancel` / `Apply` buttons.
///
/// What counts as "something to apply" ([`Self::can_apply`]): the staged value exists and differs
/// from the committed one. For a range that means BOTH ends are picked -- `RangeCalendar` reports
/// a range only once its end date is chosen (the first click just anchors it), so a half-picked
/// range is never staged and Apply stays disabled until it is complete. Picking the selected day
/// again clears the selection, which is a change like any other and can be applied.
#[derive(Copy, Clone)]
pub struct DatePickerConfirm {
    enabled: ReadSignal<bool>,
    pub(super) can_apply: Memo<bool>,
    apply: Callback<()>,
    cancel: Callback<()>,
}

impl DatePickerConfirm {
    /// Whether the picker stages its selections (`confirm: true`). When `false`, [`Self::apply`]
    /// and [`Self::cancel`] still work (they just close) but there is nothing to apply.
    pub fn enabled(&self) -> bool {
        (self.enabled)()
    }

    /// Whether a staged selection differs from the committed value, i.e. Apply has an effect.
    pub fn can_apply(&self) -> bool {
        (self.can_apply)()
    }

    /// Commit the staged selection and close the popover (focus returns to the trigger).
    pub fn apply(&self) {
        self.apply.call(());
    }

    /// Discard the staged selection and close the popover (focus returns to the trigger).
    pub fn cancel(&self) {
        self.cancel.call(());
    }
}

/// The Apply/Cancel handle of the enclosing [`DatePicker`] / [`DateRangePicker`].
pub fn use_date_picker_confirm() -> DatePickerConfirm {
    use_context::<BaseDatePickerContext>().confirm
}

/// A picker's staged selection: `None` when nothing is staged, `Some(value)` (the value itself
/// possibly `None`, a cleared selection) once the calendar has selected something.
pub(super) struct Staging<T: 'static> {
    pub(super) staged: Signal<Option<Option<T>>>,
    /// What the calendar shows: the staged value when there is one, the committed value otherwise.
    pub(super) displayed: Memo<Option<T>>,
}

impl<T> Clone for Staging<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Staging<T> {}

impl<T: Clone + PartialEq + 'static> Staging<T> {
    /// A calendar selected `value`. Only a `confirm` picker stages; returns whether it did, so the
    /// caller knows to skip the commit-and-close path.
    pub(super) fn stage(self, confirm: DatePickerConfirm, value: Option<T>) -> bool {
        if !confirm.enabled() {
            return false;
        }
        let mut staged = self.staged;
        staged.set(Some(value));
        true
    }
}

/// Create a picker's staging and its [`DatePickerConfirm`]. A hook: call it unconditionally, once
/// per render, from the picker root, before the context is provided.
///
/// The staged value is dropped whenever the popover opens or closes, so no way of closing it
/// (Escape, an outside click, the trigger, a typed commit in the input) can leave a selection
/// behind to resurface on the next open -- that is the construction behind "Cancel / Escape /
/// outside click restore the previous value": none of them has to know about staging.
pub(super) fn use_staging<T: Clone + PartialEq + 'static>(
    enabled: ReadSignal<bool>,
    open: Signal<bool>,
    dwell: CloseDwell,
    committed: ReadSignal<Option<T>>,
    commit: Callback<Option<T>>,
) -> (Staging<T>, DatePickerConfirm) {
    let mut staged = use_signal(|| None::<Option<T>>);
    let displayed = use_memo(move || match staged() {
        Some(value) => value,
        None => committed(),
    });
    let can_apply =
        use_memo(move || enabled() && staged().is_some_and(|value| value != committed()));

    use_effect(move || {
        let _ = open();
        if staged.peek().is_some() {
            staged.set(None);
        }
    });

    let close = move || {
        let mut open = open;
        dwell.cancel();
        return_focus_to_trigger();
        open.set(false);
    };
    let apply = use_callback(move |_: ()| {
        let pending = staged.peek().clone();
        if let Some(value) = pending {
            if *committed.peek() != value {
                commit.call(value);
            }
        }
        staged.set(None);
        close();
    });
    let cancel = use_callback(move |_: ()| {
        staged.set(None);
        close();
    });

    (
        Staging { staged, displayed },
        DatePickerConfirm {
            enabled,
            can_apply,
            apply,
            cancel,
        },
    )
}
