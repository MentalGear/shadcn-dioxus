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
//! * Author inputs (backlog row 240): `initial_focus` / `final_focus` on `DialogContent`,
//!   `AlertDialogContent` and `PopoverContent` are functions of [`OpenModality`] returning a
//!   [`FocusTarget`]. They feed the policy above through `data-dx-initial-focus` /
//!   `data-dx-final-focus` (read by `dxFocusSpec`); the browser default, the touch rule and the
//!   fallback for a selector that matches nothing stay the policy's. `dxOpenFocus` also remembers
//!   the opener (`panel.__dxOpener`) for every overlay kind, and `dxCloseFocus` hands focus back.
//!   Script focus never passes `focusVisible`: a mouse-opened field must still ring (`:focus-visible`).
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
pub const MODALITY_JS: &str = "(function(){var W=window,R=document.documentElement;if(W.__dxModality)return;W.__dxModality=1;function S(m){if(R.getAttribute('data-dx-modality')!==m)R.setAttribute('data-dx-modality',m)}var Q=W.matchMedia&&W.matchMedia('(any-pointer: coarse) and (hover: none)');if(!R.hasAttribute('data-dx-modality'))S(Q&&Q.matches?'touch':'mouse');var K={Shift:1,Control:1,Alt:1,Meta:1,AltGraph:1,CapsLock:1,Fn:1,FnLock:1,Hyper:1,Super:1,OS:1};function P(t){return t==='touch'||t==='pen'?t:'mouse'}try{/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('pointerdown',function(e){S(P(e.pointerType))},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('keydown',function(e){if(!K[e.key])S('keyboard')},true);/*listener-ok: the modality writer lives as long as the page*/W.addEventListener('click',function(e){if(e.detail===0){S('keyboard')}else if(R.getAttribute('data-dx-modality')==='keyboard')S(P(e.pointerType||(Q&&Q.matches?'touch':'mouse')))},true)}catch(x){}})()";

/// The open-focus policy as JS function declarations (`dxModality`, `dxEditable`, `dxFocusSpec`,
/// `dxOpenFocus`, `dxCloseFocus`), prefixed to the overlay scripts that open or close a panel.
/// `dxOpenFocus(panel, show)` calls `show()` (`showModal()` or `showPopover()`) and decides where
/// focus lands -- see the module doc for the table. This is the only place a panel's initial focus
/// is chosen, and the only place the opener is remembered (`panel.__dxOpener`, read by the dialog's
/// close handoff and by `dxCloseFocus` for every overlay kind).
///
/// An author's `initial_focus` / `final_focus` ([`FocusTarget`]) are INPUTS to this policy: the
/// props are evaluated for the four modalities at render and stored as `data-dx-initial-focus` /
/// `data-dx-final-focus` on the panel, and `dxFocusSpec` reads the entry for the current modality.
/// There is no second path; with no prop the table is absent and the built-in policy decides.
///
/// Script focus here never passes `focusVisible` (Chrome 144+): it would override the text-field
/// rule (a focused field matches `:focus-visible` after any modality, `--dx-ring-field`), and the
/// non-field rings are already decided by the modality attribute. See the backlog row 240 update.
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
// The author's choice for `kind` ('initial' | 'final') under modality `m`: 'default', 'panel',
// 'none' or 'selector:<css>'. Absent or malformed means 'default'.
function dxFocusSpec(panel, kind, m) {
    try {
        const raw = panel.getAttribute('data-dx-' + kind + '-focus');
        const v = raw && JSON.parse(raw)[m];
        return typeof v === 'string' ? v : 'default';
    } catch (e) { return 'default'; }
}
function dxOpenFocus(panel, show) {
    const m = dxModality();
    // The element holding focus now is the opener: the one place it is remembered, for every
    // overlay kind. The close hands focus back to it (`dxCloseFocus`, the dialog's close handoff).
    const prev = document.activeElement;
    panel.__dxOpener = prev && prev !== document.body && prev !== document.documentElement ? prev : null;
    const marked = panel.querySelector('[data-dx-autofocus], [autofocus]');
    // 1. The author's `initial_focus` for this modality, when it names something that exists.
    const spec = dxFocusSpec(panel, 'initial', m);
    let plan = null; // 'panel' | Element | 'browser'
    if (spec === 'panel') plan = 'panel';
    else if (spec.startsWith('selector:')) {
        let hit = null;
        try { hit = panel.querySelector(spec.slice(9)); } catch (e) { hit = null; }
        if (hit) plan = hit;
    }
    // 2. The built-in policy (Base UI's default): only touch and pen are kept off the controls.
    if (!plan) {
        if (m === 'keyboard' || m === 'mouse') {
            // The browser's own step focuses an `[autofocus]` target or else the first focusable
            // descendant, a text field included; `data-dx-autofocus` (a palette's search box) is
            // focused explicitly.
            plan = marked && marked.hasAttribute('data-dx-autofocus') ? marked : 'browser';
        } else {
            // An autofocus target wins only when it is not a field.
            plan = marked && !dxEditable(marked) ? marked : 'panel';
        }
    }
    if (plan === 'browser') { show(); return; }
    if (plan !== 'panel') {
        show();
        plan.focus({ preventScroll: true });
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
// Where focus goes when the panel has closed (`panel` may already be out of the top layer). The
// author's `final_focus` for the current modality: 'none' leaves no element focused, a selector
// names the target, 'default' returns focus to the element focused before the open
// (`panel.__dxOpener`, which is the trigger for a click open and whatever held focus for a
// programmatic one). Only when focus is still inside the panel or lost (or, for an explicit
// target, on the opener the browser just restored): focus the user already moved elsewhere stays.
// `insideOnly` (a panel closed by state, not by the browser's own hide): only when focus is still
// inside the panel, so a click outside that took focus (or none) is never undone.
function dxCloseFocus(panel, insideOnly) {
    const m = dxModality();
    const spec = dxFocusSpec(panel, 'final', m);
    const a = document.activeElement;
    const opener = panel.__dxOpener && panel.__dxOpener.isConnected ? panel.__dxOpener : null;
    const inside = !!a && panel.contains(a);
    const nowhere = !a || a === document.body || a === document.documentElement;
    // Focus that may be moved: still in the panel, or (unless `insideOnly`) lost.
    const lost = inside || (!insideOnly && nowhere);
    // An author's explicit target also replaces the opener the browser just restored.
    const movable = lost || (!!opener && a === opener);
    if (spec === 'none') {
        if (movable) { if (a && a !== document.body) a.blur(); }
        return;
    }
    if (spec.startsWith('selector:')) {
        let hit = null;
        try { hit = document.querySelector(spec.slice(9)); } catch (e) { hit = null; }
        if (hit && movable) { hit.focus({ preventScroll: true }); return; }
    }
    if (lost && opener) opener.focus({ preventScroll: true });
}
"#;

/// Close handoff for a panel the browser does not hide itself (the non-modal Popover leaves the DOM
/// by state): when `open` goes `false`, `dxCloseFocus` runs on the still-mounted panel, which
/// returns focus to the opener (or the author's `final_focus`) if focus is still inside it. The
/// modal `<dialog>` does the same from `use_dialog_open_driver` right after its native `close()`.
#[cfg(feature = "web")]
pub(crate) fn use_close_focus(id: String, open: impl Readable<Target = bool> + Copy + 'static) {
    let mut was_open = use_signal(|| false);
    use_effect(move || {
        let now = open.cloned();
        let before = *was_open.peek();
        was_open.set(now);
        if before && !now {
            let id = id.clone();
            document::eval(&format!(
                "{OPEN_FOCUS_JS}
                const panel = document.getElementById('{id}');
                if (panel) dxCloseFocus(panel, true);"
            ));
        }
    });
}

/// What an overlay focuses: the value of an `initial_focus` or `final_focus` prop.
///
/// A prop is a function of the [`OpenModality`] (Base UI's `initialFocus(openType)`): `|m|
/// if m == OpenModality::Touch { FocusTarget::Panel } else { FocusTarget::selector("#name") }`.
/// It is an input to the single open-focus policy (`dxOpenFocus` / `dxCloseFocus`), never a
/// second code path: the function is called once per modality at render, the answers ride on the
/// panel as a data attribute, and the policy reads the one for the modality of the open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FocusTarget {
    /// The built-in policy: `initial_focus` -- the browser's first focusable control for keyboard
    /// and mouse, the panel for touch and pen; `final_focus` -- the element focused before the
    /// open (the trigger for a click open).
    #[default]
    Default,
    /// `initial_focus` only: the panel itself (no control, so no software keyboard).
    Panel,
    /// A CSS selector: `initial_focus` resolves it inside the panel, `final_focus` in the
    /// document. A selector that matches nothing falls back to [`FocusTarget::Default`].
    Selector(String),
    /// `final_focus` only: leave nothing focused after the close.
    None,
}

impl FocusTarget {
    /// [`FocusTarget::Selector`] from anything string-like.
    pub fn selector(css: impl Into<String>) -> Self {
        FocusTarget::Selector(css.into())
    }

    fn encode(&self) -> String {
        match self {
            FocusTarget::Default => "default".to_string(),
            FocusTarget::Panel => "panel".to_string(),
            FocusTarget::None => "none".to_string(),
            FocusTarget::Selector(css) => format!("selector:{css}"),
        }
    }
}

/// How the open (or close) was made: the value of `<html data-dx-modality>` when it happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OpenModality {
    Keyboard,
    Mouse,
    Touch,
    Pen,
}

impl OpenModality {
    /// All four, in the order the table is written.
    pub const ALL: [OpenModality; 4] = [
        OpenModality::Keyboard,
        OpenModality::Mouse,
        OpenModality::Touch,
        OpenModality::Pen,
    ];

    /// The attribute value (`"keyboard"`, `"mouse"`, `"touch"`, `"pen"`).
    pub fn as_str(self) -> &'static str {
        match self {
            OpenModality::Keyboard => "keyboard",
            OpenModality::Mouse => "mouse",
            OpenModality::Touch => "touch",
            OpenModality::Pen => "pen",
        }
    }
}

/// An `initial_focus` / `final_focus` prop: a function of the modality.
pub type FocusTargetFn = Callback<OpenModality, FocusTarget>;

/// The JSON table the policy reads: `{"keyboard": "...", "mouse": "...", ...}`, or `None` when the
/// prop is absent or answers [`FocusTarget::Default`] for every modality (nothing to store).
pub(crate) fn focus_table(f: Option<FocusTargetFn>) -> Option<String> {
    let f = f?;
    let entries: Vec<(OpenModality, FocusTarget)> =
        OpenModality::ALL.iter().map(|m| (*m, f.call(*m))).collect();
    if entries.iter().all(|(_, t)| *t == FocusTarget::Default) {
        return None;
    }
    let body: Vec<String> = entries
        .iter()
        .map(|(m, t)| format!("\"{}\":{}", m.as_str(), json_string(&t.encode())))
        .collect();
    Some(format!("{{{}}}", body.join(",")))
}

/// A JSON string literal (quotes, backslashes and control characters escaped).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The `data-dx-initial-focus` / `data-dx-final-focus` attributes for a panel's props, appended to
/// the panel's own attribute list.
pub(crate) fn focus_attributes(
    initial: Option<FocusTargetFn>,
    final_: Option<FocusTargetFn>,
) -> Vec<Attribute> {
    let mut out = Vec::new();
    if let Some(t) = focus_table(initial) {
        out.push(Attribute::new("data-dx-initial-focus", t, None, false));
    }
    if let Some(t) = focus_table(final_) {
        out.push(Attribute::new("data-dx-final-focus", t, None, false));
    }
    out
}

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
    fn focus_targets_encode_and_escape() {
        assert_eq!(FocusTarget::Default.encode(), "default");
        assert_eq!(FocusTarget::Panel.encode(), "panel");
        assert_eq!(FocusTarget::None.encode(), "none");
        assert_eq!(FocusTarget::selector("#a").encode(), "selector:#a");
        assert_eq!(
            json_string("selector:[x=\"y\"]"),
            "\"selector:[x=\\\"y\\\"]\""
        );
    }

    #[test]
    fn open_focus_policy_is_the_only_reader_of_the_focus_tables() {
        assert!(OPEN_FOCUS_JS.contains("dxFocusSpec(panel, 'initial'"));
        assert!(OPEN_FOCUS_JS.contains("dxFocusSpec(panel, 'final'"));
        assert!(!OPEN_FOCUS_JS.contains("focusVisible"));
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
