//! How the user is interacting right now -- keyboard, mouse, touch or pen -- written ONCE to
//! `<html data-dx-modality="...">`, and the one open-focus policy that reads it (backlog rows 206
//! and 226).
//!
//! ## The class this removes
//!
//! Every overlay has to decide two things when it opens, and each used to decide them on its own:
//!
//! 1. **Where does focus go?** `showModal()`/`showPopover()` focus the first focusable descendant,
//!    which for a text field raises the on-screen keyboard while the panel is still being placed. A
//!    keyboard user wants exactly that; a finger does not. Row 206 read the opener's
//!    `:focus-visible` to tell them apart, which is wrong on iOS: Safari does not focus a tapped
//!    button, and keeps focus on a previously tapped text input, which always matches
//!    `:focus-visible`, so a tap after typing was read as a keyboard open.
//! 2. **Does the focused thing paint a ring?** A script-focused `<dialog>`/popover panel gets the
//!    UA `outline: auto`, and a script-focused trigger gets the library's box-shadow ring, because
//!    script focus with no pointer-focus before it counts as keyboard-ish.
//!
//! Both are questions about the *modality* of the last interaction, so the modality is recorded
//! once, by a page-lifetime capture listener pair, and never inferred from `:focus-visible` or
//! `document.activeElement`:
//!
//! * [`MODALITY_JS`] -- the single writer. `pointerdown` writes `touch`, `pen` or `mouse` from
//!   `pointerType`; a non-modifier `keydown` writes `keyboard`. Before any event has been seen
//!   (a `default_open`, a programmatic open) a `(any-pointer: coarse) and (hover: none)` match
//!   reads as `touch`, anything else as `mouse`. The attribute is installed from the app's
//!   pre-paint script (`preview/src/theme.rs` `PREPAINT_JS`, byte-identical in `index.html`) so
//!   it is right from the first frame, and again as a prefix of every overlay script that needs
//!   it (idempotent), so an app without the pre-paint script still has it from the first overlay
//!   mount.
//! * [`OPEN_FOCUS_JS`] -- `dxOpenFocus(panel, show)`, the one open-focus policy, used by
//!   `use_dialog_open_driver` (every native-`<dialog>` modal), `use_popover_shown_while_mounted`
//!   for a `<dialog popover>` (the non-modal Popover, hence DatePicker and ColorPicker) and, through
//!   `data-dx-autofocus`, by the Command palette. Keyboard and mouse (Base UI's default open-focus
//!   policy, owner decision 2026-10-18): the browser default -- an `[autofocus]` target, else the
//!   first focusable control, a text field included -- and an element marked `data-dx-autofocus`
//!   (a palette's search box) explicitly. Touch and pen: the panel, never an editable element, so
//!   the on-screen keyboard does not rise while the panel is placed; a plain `[autofocus]` wins
//!   only when it is not editable. Base UI's `createDefaultInitialFocus` likewise returns the
//!   popup only for touch. Two deliberate deviations: pen is grouped with touch here (Base UI groups
//!   pen with mouse), and the Drawer follows the same rule as every other modal (Base UI's
//!   DrawerPopup always focuses the popup; our touch rule and `preventScroll` already cover its
//!   mobile-keyboard and swipe-gesture reasons).
//! * The theme (`dx-components-theme.css`, "Focus modality") reads the same attribute: the
//!   non-field ring tokens are transparent unless the modality is `keyboard`, and a programmatic
//!   focus container (`dialog`, `[popover]`, listbox/menu, `tabindex="-1"`) never draws the UA
//!   outline. `--dx-ring-field` is untouched: a focused text field still rings after a mouse open
//!   (shadcn does the same); buttons and containers show no ring after a pointer open.
//!
//! `scripts/check-open-focus.sh` keeps a second hand-written `autofocus`, `.focus()` or
//! `set_focus(true)` on an open path from coming back.

use dioxus::prelude::*;

/// The modality writer: one idempotent script, safe to run any number of times.
///
/// Page-lifetime capture listeners (`pointerdown`, `keydown`, and a `click` that no `pointerdown`
/// preceded, which iOS produces for a tap in a control's hit slop) write `data-dx-modality` on
/// `<html>` to `keyboard`, `mouse`, `touch` or `pen`. Kept on ONE line with no double quotes so the
/// pre-paint script can embed the identical text; `preview/src/theme.rs` has a test that it does,
/// and `scripts/check-open-focus.sh` compares all three copies.
pub const MODALITY_JS: &str = "(function(){var W=window,R=document.documentElement;if(W.__dxModality)return;W.__dxModality=1;function S(m){if(R.getAttribute('data-dx-modality')!==m)R.setAttribute('data-dx-modality',m)}var Q=W.matchMedia&&W.matchMedia('(any-pointer: coarse) and (hover: none)');if(!R.hasAttribute('data-dx-modality'))S(Q&&Q.matches?'touch':'mouse');var K={Shift:1,Control:1,Alt:1,Meta:1,AltGraph:1,CapsLock:1,Fn:1,FnLock:1,Hyper:1,Super:1,OS:1};function P(t){return t==='touch'||t==='pen'?t:'mouse'}try{/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('pointerdown',function(e){S(P(e.pointerType))},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('keydown',function(e){if(!K[e.key])S('keyboard')},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('click',function(e){if(e.detail>0&&R.getAttribute('data-dx-modality')==='keyboard')S(P(e.pointerType||(Q&&Q.matches?'touch':'mouse')))},true)}catch(x){}})()";

/// The open-focus policy as JS function declarations (`dxModality`, `dxEditable`, `dxOpenFocus`),
/// prefixed to the overlay scripts that open a panel. `dxOpenFocus(panel, show)` calls `show()`
/// (`showModal()` or `showPopover()`) and decides where focus lands -- see the module doc for the
/// table. This is the only place a panel's initial focus is chosen.
pub const OPEN_FOCUS_JS: &str = r#"
function dxModality() {
    const m = document.documentElement.getAttribute('data-dx-modality');
    if (m) return m;
    return matchMedia('(any-pointer: coarse) and (hover: none)').matches ? 'touch' : 'mouse';
}
function dxEditable(el) {
    if (!el) return false;
    if (el.isContentEditable) return true;
    if (el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') return true;
    return el.tagName === 'INPUT' && !/^(button|checkbox|radio|submit|reset|image|file|range|color|hidden)$/i.test(el.type);
}
function dxOpenFocus(panel, show) {
    const m = dxModality();
    const marked = panel.querySelector('[data-dx-autofocus], [autofocus]');
    if (m === 'keyboard' || m === 'mouse') {
        // Base UI's default: only touch (and pen) is kept off the controls. The browser's own
        // step focuses an `[autofocus]` target or else the first focusable descendant, a text
        // field included; `data-dx-autofocus` (a palette's search box) is focused explicitly.
        show();
        if (marked && marked.hasAttribute('data-dx-autofocus')) marked.focus({ preventScroll: true });
        return;
    }
    // Touch and pen: an autofocus target wins only when it is not a field.
    const wins = marked && !dxEditable(marked);
    if (wins) {
        show();
        marked.focus({ preventScroll: true });
        return;
    }
    // The panel itself takes focus, with its children inert for the one call so the browser's own
    // focusing step finds no delegate: a field is never focused, not even for an instant.
    panel.setAttribute('tabindex', '-1');
    const held = [...panel.children].filter((c) => !c.inert);
    held.forEach((c) => { c.inert = true; });
    try {
        show();
        panel.focus({ preventScroll: true });
    } finally {
        held.forEach((c) => { c.inert = false; });
    }
}
"#;

/// The pointer type of the press that precedes a click, kept per component instance.
///
/// A click handler cannot tell a tap from a mouse click (`MouseData` has no pointer type), and the
/// Safari remedy "focus the button on click" (Safari does not focus a clicked button) is wrong for a
/// touch: it makes a script-focused control match `:focus-visible` and ring, and it dismisses the
/// software keyboard of a field the user was typing in. [`PressKind`] records the type from
/// `onpointerdown` so the click can focus only after a mouse press.
#[derive(Clone, Copy)]
pub(crate) struct PressKind(Signal<bool>);

impl PressKind {
    /// Record the press: wire this to the element's `onpointerdown`.
    pub(crate) fn pressed(&self, event: &PointerEvent) {
        // Written from an event handler and only ever read from one, so nothing re-renders.
        let mut touch = self.0;
        touch.set(event.pointer_type() != "mouse");
    }

    /// Whether the last press was a touch or a pen (a click without a press counts as a mouse or
    /// keyboard one: the element is already focused by then).
    pub(crate) fn was_touch(&self) -> bool {
        *self.0.peek()
    }
}

/// A [`PressKind`] that lives as long as the calling component.
pub(crate) fn use_press_kind() -> PressKind {
    PressKind(use_signal(|| false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_is_one_line_without_double_quotes() {
        // The pre-paint script embeds it inside a Rust string literal and an HTML `<script>`.
        assert!(!MODALITY_JS.contains('\n'));
        assert!(!MODALITY_JS.contains('"'));
        assert!(!MODALITY_JS.contains("</script"));
    }

    #[test]
    fn writer_is_idempotent_and_reads_no_focus_state() {
        assert!(MODALITY_JS.contains("__dxModality"));
        // The decision is never taken from :focus-visible or activeElement.
        assert!(!MODALITY_JS.contains("focus-visible"));
        assert!(!MODALITY_JS.contains("activeElement"));
        assert!(!OPEN_FOCUS_JS.contains("focus-visible"));
        assert!(!OPEN_FOCUS_JS.contains("activeElement"));
    }
}
