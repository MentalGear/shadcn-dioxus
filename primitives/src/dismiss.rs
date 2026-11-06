//! The dismiss core (backlog row 487): one place that decides when an overlay closes.
//!
//! An overlay closes three ways: **Escape**, **a press outside**, and **focus leaving**. An overlay
//! states a [`Policy`] and calls [`use_dismiss`], so a rule such as "a touch press outside is a tap,
//! not a touch" (row 477) lands once and reaches every overlay; the page half is one script
//! (`DISMISS_JS` in `lib.rs`) that every layer shares, so
//!
//! - **Escape is layer-aware.** Layers that take part in Escape register on one page-side list ordered
//!   by when they opened, and only the topmost reacts. [`EscapeKey::Close`] closes it and cancels the
//!   key's default action, so an enclosing `<dialog>` does not also take the key as a `cancel`;
//!   [`EscapeKey::Native`] (a modal `<dialog>`, a `popover="auto"`) stays in the order but leaves the
//!   key to the browser, so a layer under it never closes first. A Select inside a Dialog therefore
//!   closes on the first Escape and the Dialog on the second.
//! - **A press outside follows the tap rule.** A mouse press dismisses on `pointerdown`; a touch or
//!   pen press is a tap (`pointerup` within 10 px), and a gesture the browser takes to scroll
//!   (`pointercancel`) dismisses nothing.
//! - **Focus leaving** dismisses where the surface wants it ([`Policy::focus`]).
//! - **What closed it** is the [`Cause`] handed to `on_dismiss`, so a surface returns focus to its
//!   trigger after Escape and leaves it where the user put it after an outside press
//!   (`crate::use_refocus_on_close_unless` with [`Cause::interacted_outside`]).
//!
//! ## Policy per surface
//!
//! | surface | Escape | press outside | focus leaving |
//! |---|---|---|---|
//! | Tooltip, HoverCard | close | no (hover surfaces: the pointer leaving closes them) | no (blur closes them) |
//! | NavigationMenu | close, focus back to the trigger | yes, both activation modes | no (its own blur debounce) |
//! | Navbar | close, focus back to the trigger | native `popover="auto"` light dismiss | no |
//! | Select | close | yes | yes |
//! | Combobox | close | no (a press elsewhere is allowed to leave the list open; the input's blur closes it) | no |
//! | Popover, Dialog, AlertDialog (web) | native (`cancel`), in the order | native (`popover="auto"`, the backdrop) | no |
//! | Popover, Dialog (desktop, no browser dismissal) | close | yes | yes |
//! | AlertDialog (desktop) | close | no (an alert dialog never light-dismisses) | no |
//! | DropdownMenu, ContextMenu, Menubar ([`Policy::MENU`]) | close, focus back to the trigger (the Menubar: to the menu's own trigger) | yes | yes |
//! | a submenu ([`Policy::MENU_SUB`]) | close only the submenu, focus back to its sub-trigger | no (a press outside closes the whole menu, and so the submenu with it) | yes |
//!
//! The menu hosts share one focus-out policy: focus moving to another element closes the menu and
//! focus stays where the user put it. It is Base UI's, which shadcn's `base-nova` menus wrap:
//! `Menu.Popup` renders a `FloatingFocusManager` (`@base-ui/react` 1.9.0, `menu/popup/MenuPopup.mjs`)
//! and leaves `closeOnFocusOut` at its default `true`
//! (`floating-ui-react/components/FloatingFocusManager.mjs`), for the menubar's menus as for a
//! dropdown. Escape is a `close` layer for every menu panel, a `popover="auto"` one included: the
//! browser's own close echo cannot say that Escape caused it, and the [`Cause`] is what decides
//! whether focus goes back to the trigger.
//!
//! ## The gap before a layer is on the page
//!
//! A layer registers on the page only after its panel has mounted (an effect on a later frame), and
//! a pointer open runs one task after the press. An Escape that arrives between "open requested" and
//! that registration reaches no layer, so the overlay would open and stay open. The gap is closed on
//! the Rust side by the element that has focus while it lasts, the trigger: it reports the request
//! ([`OpenGap`] for an open that waits a task, a timer's pending state for a hover delay) and takes
//! Escape through [`escape_in_gap`] while the overlay is open or its open is pending, closing or
//! cancelling it and consuming the key exactly as a registered `close` layer does. A hover surface
//! (Tooltip, HoverCard) also keeps its layer registered from the moment its open is pending, so an
//! Escape with focus elsewhere cancels a hover delay.
//!
//! The page half is still a legacy script (`dev-docs/js-interop.md`); it is the one script row 456
//! ports.

use std::cell::Cell;

use dioxus::prelude::*;
use serde::Serialize;

use crate::{dismiss_listeners, js_listener::use_js_listeners};

/// What Escape does to a layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EscapeKey {
    /// Escape closes the layer while it is the topmost layer taking part, and the key's default
    /// action is cancelled.
    Close,
    /// The browser closes the layer (a modal `<dialog>`'s `cancel`, a `popover="auto"`'s close
    /// request). The layer stays in the order, so no layer under it takes the key, and the key is
    /// not touched.
    #[allow(
        dead_code,
        reason = "constructed by the web arms, whose dismissal the browser owns"
    )]
    Native,
}

/// How a layer is dismissed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Policy {
    /// What Escape does.
    pub escape: EscapeKey,
    /// A press outside the root dismisses: a mouse press at once, a touch or pen press as a tap.
    pub press: bool,
    /// Focus moving outside the root dismisses.
    pub focus: bool,
}

impl Policy {
    /// Escape only (the hover surfaces).
    pub(crate) const ESCAPE: Policy = Policy {
        escape: EscapeKey::Close,
        press: false,
        focus: false,
    };

    /// Escape and a press outside, focus left alone.
    pub(crate) const ESCAPE_AND_PRESS: Policy = Policy {
        escape: EscapeKey::Close,
        press: true,
        focus: false,
    };

    /// Escape, a press outside and focus leaving.
    pub(crate) const ESCAPE_PRESS_AND_FOCUS: Policy = Policy {
        escape: EscapeKey::Close,
        press: true,
        focus: true,
    };

    /// A menu host (DropdownMenu, ContextMenu, Menubar): Escape, a press outside and focus leaving all
    /// close it (see the module doc's table).
    pub(crate) const MENU: Policy = Policy::ESCAPE_PRESS_AND_FOCUS;

    /// A submenu: Escape closes just the submenu and focus leaving it closes it. A press outside is the
    /// enclosing menu's, which closes the whole tree.
    pub(crate) const MENU_SUB: Policy = Policy {
        escape: EscapeKey::Close,
        press: false,
        focus: true,
    };

    /// The browser owns Escape and the outside press; the layer only keeps its place in the order.
    #[allow(
        dead_code,
        reason = "used by the web arms, whose dismissal the browser owns"
    )]
    pub(crate) const NATIVE: Policy = Policy {
        escape: EscapeKey::Native,
        press: false,
        focus: false,
    };
}

/// Where a layer's root is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// The element with the given id.
    Element,
    /// The nearest ancestor (or the element itself) matching a selector, found from the given id:
    /// the `nav` that holds a NavigationMenu trigger.
    Closest(&'static str),
}

/// Why a layer was asked to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cause {
    /// Escape, while focus was somewhere in the layer's root.
    EscapeWithin,
    /// Escape, with focus elsewhere (the page body, another control).
    Escape,
    /// A press outside the root.
    OutsidePress,
    /// Focus moved outside the root.
    FocusLeft,
}

impl Cause {
    /// The user acted somewhere else: focus belongs where they put it, not back on the trigger.
    pub(crate) fn interacted_outside(self) -> bool {
        matches!(self, Cause::OutsidePress | Cause::FocusLeft)
    }

    /// The key closed it.
    pub(crate) fn is_escape(self) -> bool {
        matches!(self, Cause::Escape | Cause::EscapeWithin)
    }

    /// Escape came while focus was inside the layer's root, so focus has a place to be handed back
    /// from.
    pub(crate) fn focus_was_within(self) -> bool {
        self == Cause::EscapeWithin
    }

    fn from_message(message: &str) -> Option<Self> {
        match message {
            "escape-within" => Some(Cause::EscapeWithin),
            "escape" => Some(Cause::Escape),
            "outside" => Some(Cause::OutsidePress),
            "focus" => Some(Cause::FocusLeft),
            _ => None,
        }
    }
}

/// The argument of the page script: one layer's policy, root and place in the Escape order.
#[derive(Serialize)]
pub(crate) struct Spec {
    id: String,
    /// A second element that is inside too (a submenu's trigger beside its content).
    also: Option<String>,
    closest: Option<&'static str>,
    seq: u64,
    escape: EscapeKey,
    press: bool,
    focus: bool,
}

thread_local! {
    static NEXT_SEQ: Cell<u64> = const { Cell::new(1) };
}

/// The next place in the Escape order: a layer that opens later is above one that opened earlier.
fn next_seq() -> u64 {
    NEXT_SEQ.with(|n| {
        let seq = n.get();
        n.set(seq + 1);
        seq
    })
}

impl Spec {
    fn new(id: String, also: Option<String>, scope: Scope, policy: Policy) -> Self {
        Spec {
            id,
            also,
            closest: match scope {
                Scope::Element => None,
                Scope::Closest(selector) => Some(selector),
            },
            seq: next_seq(),
            escape: policy.escape,
            press: policy.press,
            focus: policy.focus,
        }
    }
}

/// The trigger's Escape arm for the gap before the layer is on the page (see the module doc).
///
/// `requested` is "the overlay is open, or its open is pending". When the key is Escape and
/// `requested` holds, `close` runs and the key is consumed (default action cancelled, propagation
/// stopped), as a registered `close` layer would, so an enclosing `<dialog>` keeps its own Escape.
/// Returns whether it took the key.
pub(crate) fn escape_in_gap(event: &KeyboardEvent, requested: bool, close: impl FnOnce()) -> bool {
    if requested && event.key() == Key::Escape {
        close();
        event.prevent_default();
        event.stop_propagation();
        true
    } else {
        false
    }
}

/// An open that runs one task after the gesture that requested it, and the state of that wait.
///
/// A mouse press on a trigger opens one task later, after the press's `mousedown` has moved focus
/// onto it. Until that task has run, the overlay is neither open nor a layer, yet the user has asked
/// for it: [`OpenGap::escape`] cancels the wait.
#[derive(Clone, Copy)]
pub(crate) struct OpenGap {
    task: Signal<Option<dioxus::core::Task>>,
}

/// Create an [`OpenGap`]. A hook: call it unconditionally, once per render.
pub(crate) fn use_open_gap() -> OpenGap {
    OpenGap {
        task: use_signal(|| None),
    }
}

impl OpenGap {
    /// Run `open` one task from now, replacing a wait already pending.
    pub(crate) fn defer(mut self, open: impl FnOnce() + 'static) {
        self.cancel();
        let mut slot = self.task;
        let task = spawn(async move {
            crate::sleep::sleep(std::time::Duration::ZERO).await;
            slot.set(None);
            open();
        });
        self.task.set(Some(task));
    }

    /// Whether an open is waiting for its task.
    pub(crate) fn pending(self) -> bool {
        self.task.peek().is_some()
    }

    /// Drop the wait, if any.
    pub(crate) fn cancel(mut self) {
        if let Some(task) = self.task.write().take() {
            task.cancel();
        }
    }

    /// The trigger's Escape arm: [`escape_in_gap`] for an overlay that is `open` or waiting on this
    /// handle. The wait is cancelled before `close` runs.
    pub(crate) fn escape(self, event: &KeyboardEvent, open: bool, close: impl FnOnce()) -> bool {
        escape_in_gap(event, open || self.pending(), || {
            self.cancel();
            close();
        })
    }
}

/// Make the calling component a dismiss layer for as long as it is mounted (and `active`, when
/// given).
///
/// `root` is the id of the element that holds everything "inside": a press or focus on it, or
/// within it, is not outside. `active` is the layer's open state for a surface that stays mounted
/// while closed (a force-mounted HoverCard); a layer takes its place in the Escape order each time
/// it becomes active. `on_dismiss` is called with the [`Cause`]; the surface decides what closing
/// means (a signal write, a timer cancelled, focus handed back).
pub(crate) fn use_dismiss(
    root: impl Readable<Target = String> + Copy + 'static,
    scope: Scope,
    policy: Policy,
    active: Option<ReadSignal<bool>>,
    on_dismiss: impl FnMut(Cause) + Clone + 'static,
) {
    use_dismiss_within(
        root,
        None::<Signal<String>>,
        scope,
        policy,
        active,
        on_dismiss,
    );
}

/// [`use_dismiss`] for a layer whose inside is two elements: `root` and `also`. A submenu is its
/// content and its sub-trigger, which no common ancestor holds without holding the enclosing menu's
/// other items too. `also` may be absent (a trigger that has not rendered yet).
pub(crate) fn use_dismiss_within(
    root: impl Readable<Target = String> + Copy + 'static,
    also: Option<impl Readable<Target = String> + Copy + 'static>,
    scope: Scope,
    policy: Policy,
    active: Option<ReadSignal<bool>>,
    on_dismiss: impl FnMut(Cause) + Clone + 'static,
) {
    use_js_listeners(move || {
        if let Some(active) = active {
            if !active() {
                return None;
            }
        }
        let mut on_dismiss = on_dismiss.clone();
        let spec = Spec::new(root.cloned(), also.map(|also| also.cloned()), scope, policy);
        Some(dismiss_listeners(&spec).on_message(move |message: String| {
            if let Some(cause) = Cause::from_message(&message) {
                on_dismiss(cause);
            }
        }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIB: &str = include_str!("lib.rs");

    #[test]
    fn a_spec_serialises_the_policy_for_the_page() {
        let spec = Spec::new(
            "panel".into(),
            Some("trigger".into()),
            Scope::Closest("nav"),
            Policy::ESCAPE_AND_PRESS,
        );
        let json = serde_json::to_value(&spec).expect("serialises");
        assert_eq!(json["id"], "panel");
        assert_eq!(json["also"], "trigger");
        assert_eq!(json["closest"], "nav");
        assert_eq!(json["escape"], "close");
        assert_eq!(json["press"], true);
        assert_eq!(json["focus"], false);
        let native = Spec::new("d".into(), None, Scope::Element, Policy::NATIVE);
        let json = serde_json::to_value(&native).expect("serialises");
        assert_eq!(json["escape"], "native");
        assert!(json["closest"].is_null());
        assert!(json["also"].is_null());
    }

    #[test]
    fn a_layer_that_opens_later_is_above_one_that_opened_earlier() {
        let first = Spec::new("a".into(), None, Scope::Element, Policy::ESCAPE);
        let second = Spec::new("b".into(), None, Scope::Element, Policy::ESCAPE);
        assert!(second.seq > first.seq);
    }

    #[test]
    fn the_menu_hosts_share_one_policy_and_a_submenu_leaves_the_press_to_its_menu() {
        let menu = Policy::MENU;
        assert_eq!(menu.escape, EscapeKey::Close);
        assert!(menu.press && menu.focus);
        let sub = Policy::MENU_SUB;
        assert_eq!(sub.escape, EscapeKey::Close);
        assert!(!sub.press && sub.focus);
    }

    #[test]
    fn every_page_message_maps_to_a_cause() {
        assert_eq!(Cause::from_message("escape"), Some(Cause::Escape));
        assert_eq!(
            Cause::from_message("escape-within"),
            Some(Cause::EscapeWithin)
        );
        assert_eq!(Cause::from_message("outside"), Some(Cause::OutsidePress));
        assert_eq!(Cause::from_message("focus"), Some(Cause::FocusLeft));
        assert_eq!(Cause::from_message("other"), None);
        assert!(Cause::OutsidePress.interacted_outside());
        assert!(Cause::FocusLeft.interacted_outside());
        assert!(!Cause::Escape.interacted_outside());
        assert!(Cause::EscapeWithin.focus_was_within());
    }

    /// The script sends exactly the messages [`Cause::from_message`] reads, and only a `close`
    /// layer cancels the key.
    #[test]
    fn the_page_script_speaks_the_messages_rust_reads() {
        let start = LIB.find("const DISMISS_JS").expect("script");
        let script = &LIB[start..start + LIB[start..].find(";\n\n").expect("end")];
        for message in ["'escape-within'", "'escape'", "'outside'", "'focus'"] {
            assert!(script.contains(message), "{message}");
        }
        assert!(script.contains("o.escape === 'native'"));
        assert!(script.contains("e.preventDefault()"));
        assert!(script.contains("o.also"));
    }
}
