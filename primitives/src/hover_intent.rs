//! Hover intent: the one place a *hover-opened, one-at-a-time* menu decides what
//! the pointer means.
//!
//! [`crate::navbar::Navbar`] and [`crate::navigation_menu::NavigationMenu`] both open a
//! panel when the pointer rests on a trigger, switch panels when it moves to a sibling
//! trigger, and must close the panel once the pointer is over neither the trigger nor
//! the panel. Wired by hand per host from `onmouseenter`/`onmouseleave`, the two would
//! diverge: one schedules a close timer on leave, while the other opens by writing its open
//! index directly and "closes" on leave by clearing a focus state that a hover-open never
//! set -- so a hover-opened dropdown would stay open for ever once the pointer left it.
//! [`HoverIntent`] owns the whole decision, so a
//! host cannot have the opening half without the closing half:
//!
//! - **the open index** and its ONE write path ([`HoverIntent::set_open`]), which also
//!   clears the "opened by hover" marker, so the marker can only ever describe a panel the
//!   pointer opened and nothing since has re-decided;
//! - **the open-intent timer** ([`HOVER_OPEN_INTENT_DELAY`]): resting on a trigger while
//!   nothing is open opens it after the delay. While *something* is open, moving to a
//!   sibling trigger switches at once (Radix: `isOpenDelayed` is only true when nothing is
//!   open);
//! - **the close-grace timer** ([`HOVER_CLOSE_GRACE_DELAY`]): the pointer leaving the
//!   region (trigger + panel) of the open item starts it, and entering the region again --
//!   including the panel across the gap between it and its trigger -- cancels it. It
//!   closes whatever opened the panel (hover, click or keyboard), as Radix does;
//! - **the `open_on_hover` switch**: every entry point is gated by it, so a host in click
//!   activation mode (`open_on_hover = false`) cannot grow an ungated hover path.
//!
//! A host calls [`HoverIntent::enter`]/[`HoverIntent::leave`] from the `mouseenter`/
//! `mouseleave` of ONE element that contains both its trigger and its panel (or from
//! both elements, when they are siblings -- the panel is then entered and left through
//! the same two calls), and [`HoverIntent::cancel`] from every deliberate non-hover
//! action (a click, a key) so a pending timer cannot override it.
//!
//! **Hover intent is a mouse's.** A tap makes the browser fire compatibility `mouseenter`/
//! `mouseover` events AFTER its touch pointer events, so a host that listens to `mouseenter`
//! alone would hover-open on a tap and then toggle the very panel it opened on the click that
//! follows. Radix gates every hover handler on `pointerType === "mouse"` (`whenMouse`); here
//! the hook watches the page's pointer events (`pointerover`/`pointerdown`, capture phase) and
//! remembers whether the LAST pointer was a mouse, and [`HoverIntent::enter`] does nothing
//! (neither opens nor switches, at the call and again when the open-intent timer fires) while
//! it was a touch or a pen. A mouse event after a touch restores hover (a laptop with a
//! touchscreen). Close-side calls are not gated: a panel a tap opened still closes on its own
//! terms (click outside, Escape).
//!
//! Reference: Radix `NavigationMenu` (`radix-ui/primitives`,
//! `packages/react/navigation-menu/src/navigation-menu.tsx`): the trigger's and the
//! content's `onPointerLeave` both call `startCloseTimer` (150 ms), and `onPointerEnter` of
//! either clears it; `onTriggerEnter` opens after `delayDuration` (200 ms) only while
//! nothing is open. This crate keeps its own, slightly longer close grace (300 ms) -- the
//! panel is separated from its trigger by a gap, and a slow diagonal is more forgiving.

use std::time::Duration;

use dioxus::prelude::*;

use crate::js_listener::{use_js_listeners, JsListeners};
use crate::menu_sub::{use_delayed_action, DelayedAction};

/// Hover-intent delay before a pointer resting on a trigger opens its panel (nothing
/// open yet). Numerically the same as this theme's `--dx-motion-duration-base` token
/// (`preview/assets/dx-components-theme.css`) -- not read from it (this is a plain Rust
/// constant, not CSS), but deliberately kept in step with it.
pub(crate) const HOVER_OPEN_INTENT_DELAY: Duration = Duration::from_millis(150);

/// Grace delay before a pointer leaving a trigger or its panel (without entering the
/// other) closes the panel. Numerically the same as this theme's
/// `--dx-motion-duration-slower` token, for the same reason as [`HOVER_OPEN_INTENT_DELAY`].
pub(crate) const HOVER_CLOSE_GRACE_DELAY: Duration = Duration::from_millis(300);

/// The hover-intent state of one hover-opened, one-at-a-time menu. `Copy`, so it travels
/// in the host's context struct like any other signal handle.
#[derive(Clone, Copy)]
pub(crate) struct HoverIntent {
    open: Signal<Option<usize>>,
    opened_by_hover: Signal<bool>,
    open_timer: DelayedAction,
    close_timer: DelayedAction,
    enabled: ReadSignal<bool>,
    set_open: Callback<Option<usize>>,
    /// The last pointer event the page saw came from a touch or a pen, not a mouse.
    non_mouse: Signal<bool>,
}

/// Create a [`HoverIntent`] around the host's `open` index signal. A hook (it calls
/// `use_signal` and `use_callback`): call it unconditionally, once per render, directly
/// in the host's component body. `enabled` is the host's `open_on_hover` prop.
pub(crate) fn use_hover_intent(
    mut open: Signal<Option<usize>>,
    enabled: ReadSignal<bool>,
) -> HoverIntent {
    let mut opened_by_hover = use_signal(|| false);
    let set_open = use_callback(move |index: Option<usize>| {
        opened_by_hover.set(false);
        open.set(index);
    });
    // Each `use_delayed_action()` is itself a hook, so both run here, in the body.
    let open_timer = use_delayed_action();
    let close_timer = use_delayed_action();
    let mut non_mouse = use_signal(|| false);
    use_js_listeners(move || {
        Some(
            JsListeners::install(
                "let last = null;
                const note = (e) => {
                    const v = e.pointerType !== 'mouse';
                    if (v !== last) {
                        last = v;
                        dioxus.send(v);
                    }
                };
                listen(window, 'pointerover', note, true);
                listen(window, 'pointerdown', note, true);",
            )
            .on_message(move |v: bool| non_mouse.set(v)),
        )
    });
    HoverIntent {
        open,
        opened_by_hover,
        open_timer,
        close_timer,
        enabled,
        set_open,
        non_mouse,
    }
}

impl HoverIntent {
    /// The ONE write path for the open index: it also clears the hover marker.
    pub(crate) fn set_open(&self) -> Callback<Option<usize>> {
        self.set_open
    }

    /// Whether the open panel was opened by the hover-intent logic and not yet claimed
    /// by [`Self::claim`].
    pub(crate) fn opened_by_hover(&self) -> bool {
        (self.opened_by_hover)()
    }

    /// Claim a hover-opened panel (a click on its trigger confirms it instead of
    /// toggling it shut). The next [`Self::set_open`] is then an ordinary one.
    pub(crate) fn claim(&mut self) {
        self.opened_by_hover.set(false);
    }

    /// Cancel any pending open or close. Call it from every deliberate non-hover action
    /// on the menu (click, key), so a stale timer cannot overrule it.
    pub(crate) fn cancel(&mut self) {
        self.open_timer.cancel();
        self.close_timer.cancel();
    }

    /// The pointer entered the region (trigger or panel) of item `index`.
    pub(crate) fn enter(&mut self, index: usize) {
        if !(self.enabled)() || self.touch_pointer() {
            return;
        }
        // Back inside: whatever close the pointer's last exit scheduled is void.
        self.close_timer.cancel();
        let current = *self.open.peek();
        if current == Some(index) {
            self.open_timer.cancel();
            return;
        }
        let mut hover = *self;
        if current.is_some() {
            // Something else is open: moving to this trigger switches at once.
            self.open_timer.cancel();
            hover.open_by_hover(index);
        } else {
            self.open_timer.schedule(HOVER_OPEN_INTENT_DELAY, move || {
                if !hover.touch_pointer() {
                    hover.open_by_hover(index);
                }
            });
        }
    }

    /// The pointer entered the *panel* of item `index`, when the host reports the panel
    /// separately from the trigger (Radix's `onContentEnter`): it only cancels a pending
    /// close. Unlike [`Self::enter`] it never opens anything, so the pointer touching a
    /// panel that is already fading out cannot re-open it.
    pub(crate) fn hold(&mut self) {
        if (self.enabled)() {
            self.close_timer.cancel();
        }
    }

    /// Whether pointer hover is switched on at all (`open_on_hover`).
    pub(crate) fn enabled(&self) -> bool {
        (self.enabled)()
    }

    /// The pointer left the region (trigger or panel) of item `index`. If `index` is the
    /// open item, it closes after [`HOVER_CLOSE_GRACE_DELAY`] unless the pointer re-enters.
    pub(crate) fn leave(&mut self, index: usize) {
        if !(self.enabled)() {
            return;
        }
        self.open_timer.cancel();
        if *self.open.peek() != Some(index) {
            return;
        }
        let open = self.open;
        let set_open = self.set_open;
        self.close_timer.schedule(HOVER_CLOSE_GRACE_DELAY, move || {
            if *open.peek() == Some(index) {
                set_open.call(None);
            }
        });
    }

    /// The last pointer event the page saw was a touch or a pen: no hover intent.
    fn touch_pointer(&self) -> bool {
        *self.non_mouse.peek()
    }

    /// Open `index` as the pointer's decision. After `set_open` (which clears the
    /// marker): this open was the pointer's, so a first click confirms it.
    fn open_by_hover(&mut self, index: usize) {
        self.set_open.call(Some(index));
        self.opened_by_hover.set(true);
    }
}
