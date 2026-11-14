//! The hover/focus timers shared by [`crate::tooltip`] and [`crate::hover_card`].

use crate::delayed_action::{use_delayed_action, DelayedAction};
use dioxus::prelude::*;
use std::time::Duration;

/// The one place a hover-opened, non-modal floating hint (this card, and
/// [`crate::tooltip`]) turns pointer and focus events into open/close writes.
///
/// - **Pointer enter** on the trigger opens after `delay` ms (at once when
///   `delay` is 0, or when the surface is already open -- which only cancels a
///   pending close). Entering the *content* calls [`Self::hold`], which cancels a pending
///   close, so a pointer travelling from the trigger to the content across a
///   gap does not close it on the way.
/// - **Pointer leave** (trigger or content) closes after `close_delay` ms (at
///   once when 0).
/// - **Focus / blur / Escape** are keyboard-driven and never wait: they
///   cancel both timers and write directly (Radix `HoverCard` opens on focus
///   at once; Base UI `Tooltip` has no focus delay).
///
/// Both timers are [`DelayedAction`]s: tasks spawned in the scope that
/// schedules them, so they are cancelled when that scope drops (nothing
/// outlives an unmount), and scheduling replaces whatever the handle was
/// waiting on. Every timer re-checks `disabled` when it fires.
#[derive(Clone, Copy)]
pub(crate) struct HoverTimers {
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    delay: ReadSignal<u64>,
    close_delay: ReadSignal<u64>,
    open_timer: DelayedAction,
    close_timer: DelayedAction,
    /// What this handle last asked for. The `open` memo catches up with a write only after the
    /// event that made it, so a key in the same task would read the old state from the memo.
    asked: Signal<bool>,
}

/// Create the [`HoverTimers`] of one root. A hook: call it unconditionally,
/// once per render, in the root's body.
pub(crate) fn use_hover_timers(
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    delay: ReadSignal<u64>,
    close_delay: ReadSignal<u64>,
) -> HoverTimers {
    // Follows `open` once it has caught up, so a close that did not come through this handle (a
    // scroll, the content's own) is not remembered as a request.
    let mut asked = use_signal(|| false);
    use_effect(move || asked.set(open()));
    HoverTimers {
        open,
        set_open,
        disabled,
        delay,
        close_delay,
        open_timer: use_delayed_action(),
        close_timer: use_delayed_action(),
        asked,
    }
}

impl HoverTimers {
    /// The pointer entered the trigger.
    pub(crate) fn pointer_enter(self) {
        if (self.disabled)() {
            return;
        }
        let mut close_timer = self.close_timer;
        let mut open_timer = self.open_timer;
        close_timer.cancel();
        if *self.open.peek() {
            open_timer.cancel();
            return;
        }
        let delay = *self.delay.peek();
        if delay == 0 {
            open_timer.cancel();
            self.write(true);
        } else {
            let (disabled, timers) = (self.disabled, self);
            open_timer.schedule(Duration::from_millis(delay), move || {
                if !disabled() {
                    timers.write(true);
                }
            });
        }
    }

    /// The pointer left the trigger or the content.
    pub(crate) fn pointer_leave(self) {
        if (self.disabled)() {
            return;
        }
        let mut open_timer = self.open_timer;
        let mut close_timer = self.close_timer;
        open_timer.cancel();
        let close_delay = *self.close_delay.peek();
        if close_delay == 0 {
            close_timer.cancel();
            self.write(false);
        } else if *self.open.peek() {
            let (disabled, timers) = (self.disabled, self);
            close_timer.schedule(Duration::from_millis(close_delay), move || {
                if !disabled() {
                    timers.write(false);
                }
            });
        } else {
            close_timer.cancel();
        }
    }

    /// The pointer entered the *content*: it only cancels a pending close. Unlike
    /// [`Self::pointer_enter`] it never opens anything, so the pointer touching
    /// a surface that is already fading out cannot re-open it.
    pub(crate) fn hold(self) {
        let mut close_timer = self.close_timer;
        close_timer.cancel();
    }

    /// Keyboard focus reached the trigger: open now, no delay.
    pub(crate) fn focus(self) {
        self.cancel_pending();
        if !(self.disabled)() {
            self.write(true);
        }
    }

    /// Keyboard focus left the trigger: close now, no delay.
    pub(crate) fn blur(self) {
        self.cancel_pending();
        if !(self.disabled)() {
            self.write(false);
        }
    }

    /// Whether the pointer's open delay is running: the surface is not open yet but has been
    /// asked to open. Reading it subscribes the caller.
    pub(crate) fn pending(self) -> bool {
        self.open_timer.is_pending()
    }

    /// Ask for the surface to open or close, and remember it.
    fn write(self, open: bool) {
        let mut asked = self.asked;
        asked.set(open);
        self.set_open.call(open);
    }

    /// The trigger's Escape arm: close an open surface, or cancel an open that is still waiting
    /// (the focus open of a surface whose content has not mounted yet, a running hover delay), and
    /// take the key (`dismiss::escape_in_gap`).
    pub(crate) fn escape(self, event: &KeyboardEvent) -> bool {
        let requested = *self.open.peek() || *self.asked.peek() || self.open_timer.is_pending();
        crate::dismiss::escape_in_gap(event, requested, || self.dismiss())
    }

    /// Escape: close now, whatever `disabled` says.
    pub(crate) fn dismiss(self) {
        self.cancel_pending();
        self.write(false);
    }

    fn cancel_pending(self) {
        let mut open_timer = self.open_timer;
        let mut close_timer = self.close_timer;
        open_timer.cancel();
        close_timer.cancel();
    }
}

/// Escape closes an open hover surface wherever focus is (the dismiss core, [`crate::dismiss::Policy::ESCAPE`], a layer of the root;
/// WAI-ARIA tooltip pattern: "Escape: dismisses the Tooltip", which a HoverCard shares). A
/// force-mounted card that is closed is not a layer. A hover surface has no press or focus arm: the
/// pointer leaving or the trigger blurring already closes it.
pub(crate) fn use_hover_dismiss(id: Signal<String>, timers: HoverTimers) {
    // Registered from the moment the surface is asked to open, not from the moment its content has
    // mounted, so an Escape during the pointer's open delay cancels the open.
    let requested = use_memo(move || (timers.open)() || timers.pending());
    crate::dismiss::use_dismiss(
        id,
        crate::dismiss::Scope::Element,
        crate::dismiss::Policy::ESCAPE,
        Some(requested.into()),
        move |_| timers.dismiss(),
    );
}

