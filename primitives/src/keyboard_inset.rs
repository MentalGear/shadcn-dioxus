//! `use_keyboard_inset` / [`KeyboardInset`]: lets a modal panel with a text field in it follow the
//! on-screen keyboard.
//!
//! **The gap.** On iOS Safari (and Chromium's default `resizes-visual` mode on Android) opening the
//! keyboard shrinks the VISUAL viewport only. The layout viewport -- the box a `position: fixed`
//! panel is placed in, `100%` of it, `dvh` of it -- keeps its size, so a bottom Drawer's footer, a
//! centred Dialog's lower fields and a Sheet's last input all end up behind the keyboard, and the
//! browser answers by scrolling the page behind the (inert) modal. Anchored popovers already follow
//! the visual viewport (`top_layer.rs`'s `reposition()`); a modal panel is placed by CSS and needs
//! the numbers handed to it.
//!
//! **The construction.** While a panel is mounted, one script (listeners through
//! [`crate::js_listener`], so the removal is derived from the owning scope) watches the visual
//! viewport and publishes what a stylesheet needs, ON THE PANEL:
//!
//! * `--dx-keyboard-inset`: the distance, in `px`, from the visual viewport's bottom edge to the
//!   layout viewport's (`innerHeight - (vv.offsetTop + vv.height)`);
//! * `--dx-keyboard-top`: the visual viewport's top offset in the layout viewport (`vv.offsetTop`,
//!   `0` unless the browser has panned it);
//! * `data-dx-keyboard="open"`.
//!
//! All three exist ONLY while the keyboard is open, so a rule written against the attribute leaves
//! the panel byte-for-byte as it was with no keyboard. A sheet consumes them as
//! `.dx-x[data-dx-keyboard] { bottom: var(--dx-keyboard-inset, 0px); max-height: calc(100% - ...) }`
//! (always with a `0px` fallback: the variable is absent without a keyboard).
//!
//! The field the user is typing in is brought into view: when it takes focus, or the keyboard
//! finishes opening, and it is not already comfortably inside the visible part of its scroll
//! container, the nearest vertical scroller (else the panel) gets `padding-bottom +=
//! overlap + 48px` of scroll slack (so the last field can reach the middle) and is scrolled so the
//! field sits in the centre of the visible band. Everything is undone when the keyboard closes or the
//! panel unmounts.
//!
//! **What counts as a keyboard.** Lifted from Base UI 1.6.0 (MIT),
//! `drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.js`
//! (`KEYBOARD_RESIZE_THRESHOLD = 60`, `KEYBOARD_SCROLL_SLACK = 48`): the keyboard is open when
//! `innerHeight - visualViewport.height` exceeds 60 and `visualViewport.scale === 1`. A smaller change is browser chrome (a collapsing URL bar);
//! a pinch-zoomed page is not a keyboard, and the logic is suspended while zoomed. Adapted: the
//! inset is published as a stylesheet contract on the panel (not on a provider's viewport element),
//! a top offset is published beside it so a centred panel can be placed inside the visible band, the
//! field is scrolled only when it is not already visible, and none of vaul's imperative `bottom` /
//! `height` writes or body `position: fixed` lock is used. Base UI's touch-tap focus trick (focus
//! synchronously in `touchend` behind a `translateY(-2000px)`) is deliberately NOT ported: it is
//! only needed if a real iPhone shows the page jumping.
//!
//! This is not part of `use_dialog_open_driver` / `use_popover_sync`: those own open-time focus and
//! anchored placement. The styled Drawer, Dialog, Sheet and AlertDialog call it.

use dioxus::prelude::*;

use crate::js_listener::{use_js_listeners, JsListeners};
use crate::{use_id_or, use_unique_id};

/// The script. Receives the panel's element id, then runs until torn down.
const KEYBOARD_INSET_JS: &str = r#"
const panel = document.getElementById(await dioxus.recv());
const vv = window.visualViewport;
if (!panel || !vv) return;

// Base UI 1.6.0 DrawerVirtualKeyboardProvider: KEYBOARD_RESIZE_THRESHOLD / KEYBOARD_SCROLL_SLACK.
const THRESHOLD = __THRESHOLD__;
const SLACK = __SLACK__;
const MARGIN = __MARGIN__;
const NOT_TEXT = /^(checkbox|radio|button|submit|reset|range|file|color|image|hidden)$/;
const reduced = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;

const isField = (el) => !!el && el.nodeType === 1 && panel.contains(el) &&
    (el.isContentEditable || el.tagName === 'TEXTAREA' ||
     (el.tagName === 'INPUT' && !NOT_TEXT.test(el.type)));

// The nearest vertical scroller between the field and the panel, the panel itself otherwise.
const scrollerOf = (field) => {
    for (let el = field.parentElement; el && el !== panel; el = el.parentElement) {
        const oy = getComputedStyle(el).overflowY;
        if (oy === 'auto' || oy === 'scroll' || oy === 'overlay') return el;
    }
    return panel;
};

let isOpen = false;
let lastInset = -1;
let slack = null; // { el, padding, anchor }: what to restore
let frame = 0;

const restoreSlack = () => {
    if (!slack) return;
    slack.el.style.paddingBottom = slack.padding;
    slack.el.style.overflowAnchor = slack.anchor;
    slack = null;
};

const addSlack = (el, px) => {
    if (!slack || slack.el !== el) {
        restoreSlack();
        slack = { el, padding: el.style.paddingBottom, anchor: el.style.overflowAnchor, base: parseFloat(getComputedStyle(el).paddingBottom) || 0 };
    }
    el.style.overflowAnchor = 'none';
    el.style.paddingBottom = (slack.base + px) + 'px';
};

// Bring the focused field to the centre of the visible band of its scroller, if it is not
// already comfortably inside it.
const reveal = () => {
    frame = 0;
    const field = document.activeElement;
    if (!isOpen || !isField(field)) return;
    const sc = scrollerOf(field);
    const visTop = vv.offsetTop;
    const visBottom = vv.offsetTop + vv.height;
    let s = sc.getBoundingClientRect();
    let r = field.getBoundingClientRect();
    const inside = () => r.top >= Math.max(s.top, visTop) + MARGIN && r.bottom <= Math.min(s.bottom, visBottom) - MARGIN;
    if (inside()) return;
    addSlack(sc, Math.max(0, s.bottom - visBottom) + SLACK);
    s = sc.getBoundingClientRect();
    r = field.getBoundingClientRect();
    const bandTop = Math.max(s.top, visTop) + MARGIN;
    const bandBottom = Math.min(s.bottom, visBottom) - MARGIN;
    const delta = (r.top + r.bottom) / 2 - (bandTop + bandBottom) / 2;
    sc.scrollBy({ top: delta, behavior: reduced() ? 'auto' : 'smooth' });
};
const scheduleReveal = () => { if (!frame) frame = requestAnimationFrame(reveal); };

const clear = () => {
    panel.style.removeProperty('--dx-keyboard-inset');
    panel.style.removeProperty('--dx-keyboard-top');
    panel.removeAttribute('data-dx-keyboard');
    restoreSlack();
};

const update = () => {
    const keyboard = window.innerHeight - vv.height > THRESHOLD && Math.abs(vv.scale - 1) < 0.001;
    if (!keyboard) {
        if (isOpen) clear();
        isOpen = false;
        lastInset = -1;
        return;
    }
    const inset = Math.max(0, Math.round(window.innerHeight - vv.offsetTop - vv.height));
    panel.style.setProperty('--dx-keyboard-inset', inset + 'px');
    panel.style.setProperty('--dx-keyboard-top', Math.max(0, Math.round(vv.offsetTop)) + 'px');
    panel.setAttribute('data-dx-keyboard', 'open');
    // Only the keyboard opening or changing height re-reveals; a pan (offsetTop) alone does not.
    if (!isOpen || inset !== lastInset) scheduleReveal();
    isOpen = true;
    lastInset = inset;
};

listen(vv, 'resize', update, { passive: true });
listen(vv, 'scroll', update, { passive: true });
listen(document, 'focusin', (e) => { if (isField(e.target)) { update(); scheduleReveal(); } }, { capture: true, passive: true });
cleanup(() => { if (frame) cancelAnimationFrame(frame); clear(); });
update();
"#;

/// Base UI's `KEYBOARD_RESIZE_THRESHOLD`: the least `innerHeight - visualViewport.height`, in px,
/// that is the keyboard and not browser chrome.
pub const KEYBOARD_RESIZE_THRESHOLD_PX: u32 = 60;
/// Base UI's `KEYBOARD_SCROLL_SLACK`: the extra scroll room, in px, added under the focused field's
/// scroller so the last field can reach the middle of the visible area.
pub const KEYBOARD_SCROLL_SLACK_PX: u32 = 48;
/// The gap, in px, kept between the focused field and the edges of the visible band.
const FIELD_MARGIN_PX: u32 = 16;

fn script() -> String {
    KEYBOARD_INSET_JS
        .replace("__THRESHOLD__", &KEYBOARD_RESIZE_THRESHOLD_PX.to_string())
        .replace("__SLACK__", &KEYBOARD_SCROLL_SLACK_PX.to_string())
        .replace("__MARGIN__", &FIELD_MARGIN_PX.to_string())
}

/// The element id of a modal panel, for [`KeyboardInset`]: the caller's own `id` when it set one,
/// else a generated one. Build it with [`use_keyboard_panel`].
#[derive(Clone, Copy, PartialEq)]
pub struct KeyboardPanel {
    id: Memo<String>,
    id_prop: Memo<Option<String>>,
}

impl KeyboardPanel {
    /// The id to give the panel element (`DialogContent { id: panel.id(), .. }`).
    pub fn id(&self) -> ReadSignal<Option<String>> {
        ReadSignal::from(self.id_prop)
    }
}

/// Settles the id of a modal panel so [`KeyboardInset`] can find it: `user_id` when given and set,
/// else a generated, hydration-safe one. Call it in the component that renders the panel's content
/// component and pass [`KeyboardPanel::id`] as that component's `id`.
pub fn use_keyboard_panel(user_id: Option<ReadSignal<Option<String>>>) -> KeyboardPanel {
    let generated = use_unique_id();
    let none = use_hook(|| ReadSignal::new(Signal::new(None)));
    let user_id = user_id.unwrap_or(none);
    let id = use_id_or(generated, user_id);
    let id_prop = use_memo(move || Some(id()));
    KeyboardPanel { id, id_prop }
}

/// Publishes `--dx-keyboard-inset`, `--dx-keyboard-top` and `data-dx-keyboard` on the panel with
/// element id `panel_id` while the on-screen keyboard is open, and scrolls the focused field into
/// view. See the module docs. Active for as long as the calling component is mounted, so call it
/// from a component that lives exactly as long as the panel (see [`KeyboardInset`]).
pub fn use_keyboard_inset(panel_id: Memo<String>) {
    use_js_listeners(move || Some(JsListeners::install(&script()).arg(panel_id.cloned())));
}

/// Renders nothing and runs [`use_keyboard_inset`] for `panel`. Put it inside the panel's content
/// (`DialogContent { id: panel.id(), {children} KeyboardInset { panel } }`): the content is mounted
/// only while the modal is open or animating out, so the listeners exist exactly then.
#[component]
pub fn KeyboardInset(
    /// The panel, from [`use_keyboard_panel`].
    panel: KeyboardPanel,
) -> Element {
    use_keyboard_inset(panel.id);
    rsx! {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every placeholder is substituted; none can reach the browser as an identifier.
    #[test]
    fn the_script_has_no_unsubstituted_placeholder() {
        let s = script();
        assert!(
            !s.contains("__THRESHOLD__") && !s.contains("__SLACK__") && !s.contains("__MARGIN__")
        );
        assert!(s.contains("const THRESHOLD = 60;") && s.contains("const SLACK = 48;"));
    }

    /// The contract a stylesheet reads: the names are the public API of this module.
    #[test]
    fn the_script_publishes_the_documented_contract() {
        let s = script();
        for name in [
            "--dx-keyboard-inset",
            "--dx-keyboard-top",
            "data-dx-keyboard",
        ] {
            assert!(s.contains(name), "{name}");
        }
        // Both the publish and the clear of each custom property.
        assert!(
            s.contains("setProperty('--dx-keyboard-inset'")
                && s.contains("removeProperty('--dx-keyboard-inset'")
        );
        assert!(
            s.contains("setProperty('--dx-keyboard-top'")
                && s.contains("removeProperty('--dx-keyboard-top'")
        );
    }

    /// Pinch zoom suspends the logic, as Base UI's `scale === 1` check does.
    #[test]
    fn the_script_ignores_a_zoomed_viewport() {
        assert!(script().contains("Math.abs(vv.scale - 1) < 0.001"));
    }

    /// Registered through `listen` only: the js-listeners gate, and the derived removal.
    #[test]
    fn the_script_registers_listeners_only_through_listen() {
        let s = script();
        assert!(!s.contains("addEventListener"));
        assert!(s.contains("listen(vv, 'resize'") && s.contains("listen(vv, 'scroll'"));
        assert!(s.contains("listen(document, 'focusin'"));
    }
}
