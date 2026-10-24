//! Defines the [`InputOtp`] component and its subcomponents, a one-time-passcode
//! entry control: a row of single-character boxes backed by one real, focusable,
//! form-participating `<input>`.
//!
//! ## Provenance
//!
//! This is original composition, not a port. It builds on the *shape* of
//! [`crate::checkbox`]'s `BubbleInput` pattern -- a single real, accessible,
//! form-participating `<input>` kept in sync with a purely visual, styled
//! presentation -- but not on `BubbleInput` itself, because the two controls
//! solve different halves of that shape's problem:
//!
//! - `Checkbox`'s *visible*, interactive control is a `<button role="checkbox">`.
//!   Buttons are not [submittable
//!   elements](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#category-submit)
//!   the way `input`/`select`/`textarea` are, so `Checkbox` needs a *second*,
//!   hidden native `<input type="checkbox">` (`BubbleInput`) purely to give the
//!   surrounding `<form>` something real to submit -- one-way synced
//!   Rust-state-to-DOM via [`dioxus::document::eval`].
//! - `InputOtp`'s real `<input type="text">` is simultaneously the interactive
//!   control (it receives every keystroke, paste, and the caret) *and* the
//!   submittable one -- one control does both jobs a checkbox needs two
//!   controls for, so there is no second hidden input here at all. The boxes
//!   ([`InputOtpSlot`]) are a purely visual, `aria-hidden` overlay that mirrors
//!   this one real input's value and caret position; per
//!   `dev-docs/conformance-harness.md`'s tier-2 HTML rules, the real, focusable,
//!   labelled `<input>` is the accessible source of truth.
//!
//! The caret-position bridge (`use_caret_sync`, DOM caret index -> Rust
//! signal) is new code but the *shape* -- a small `document::eval` script that
//! attaches DOM listeners and streams values back over the eval channel,
//! cleaned up on drop -- mirrors this crate's other browser-to-signal bridges
//! (`crate::use_form_reset_listener`, `crate::use_dialog_close_sync` in
//! `lib.rs`), reused here because `dioxus-html`'s own `SelectionEvent` carries
//! no cross-platform-readable selection range (`SelectionData` is an opaque
//! marker type -- see `dioxus-html`'s `events/selection.rs`), so there is no
//! way to read a caret position from the Dioxus event itself.
//!
//! Two references inform it (never vendored or copied from), per
//! `dev-docs/guides/lifting-code.md`: upstream Dioxus PR #255 (`ealmloff`,
//! draft) and `dignifiedquire`/`rust-ui`'s `input_otp.rs`, both for shape and
//! edge-case awareness only.
//!
//! ## APG / WAI-ARIA
//!
//! There is no APG pattern for "OTP input" -- it is not a distinct ARIA
//! widget, just a styled single-line text input (APG's own pattern index has
//! no entry for it; shadcn/ui's own `InputOTP` docs describe it the same way,
//! built on the `input-otp` library over a plain `<input>`). This component
//! therefore follows WCAG's generic text-input requirements rather than
//! inventing a widget role that doesn't exist: a real, focusable, labelled
//! `<input>` (`type="text"`, `inputmode="numeric"`, name/required/disabled
//! all forwarded) is the whole accessibility surface; [`InputOtpSlot`],
//! [`InputOtpGroup`], and [`InputOtpSeparator`] render inside an
//! `aria-hidden="true"` presentational layer and contribute nothing to the
//! accessibility tree.

use crate::{
    merge_attributes, use_controlled, use_form_reset_listener, use_id_or, use_previous,
    use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// The on-screen keyboard a touch device shows for an [`InputOtp`] (the
/// `<input>`'s `inputmode`). Derived from the [`OtpPattern`], never set on its
/// own, so the keypad and the accepted characters cannot disagree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpInputMode {
    /// Digits keypad (`inputmode="numeric"`).
    #[default]
    Numeric,
    /// Full keyboard (`inputmode="text"`).
    Text,
}

impl OtpInputMode {
    /// The `inputmode` attribute value.
    pub fn as_attr(self) -> &'static str {
        match self {
            OtpInputMode::Numeric => "numeric",
            OtpInputMode::Text => "text",
        }
    }
}

/// Which characters an [`InputOtp`] accepts. ONE typed value from which the
/// `<input>`'s `inputmode` attribute, its `pattern` attribute AND the Rust-side
/// filter applied to every typed or pasted value are all derived, so the three
/// can never disagree (shadcn's `REGEXP_ONLY_DIGITS`,
/// `REGEXP_ONLY_DIGITS_AND_CHARS` and `REGEXP_ONLY_CHARS`, plus a custom set).
///
/// There is deliberately no way to set `inputmode`/`pattern` as raw
/// attributes: the control's own attributes are applied after the caller's, so
/// the typed prop always wins.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum OtpPattern {
    /// `0-9` (the default): numeric keypad.
    #[default]
    Digits,
    /// `A-Z`, `a-z`, `0-9`: full keyboard.
    Alphanumeric,
    /// `A-Z`, `a-z`: full keyboard.
    Letters,
    /// A caller-chosen character set written as the inside of a regular
    /// expression character class (no brackets), e.g. `"A-Z0-9"` or
    /// `"0-9a-f"`. Ranges (`a-z`) and literal characters are supported; escape a
    /// literal `-`, `]`, `\` or `^` with a backslash. The same string
    /// becomes the HTML `pattern` (`[<class>]*`) and the Rust filter.
    Custom {
        /// The character-class body.
        class: String,
        /// The keyboard to show.
        inputmode: OtpInputMode,
    },
}

impl OtpPattern {
    /// The keyboard for this pattern.
    pub fn inputmode(&self) -> OtpInputMode {
        match self {
            OtpPattern::Digits => OtpInputMode::Numeric,
            OtpPattern::Alphanumeric | OtpPattern::Letters => OtpInputMode::Text,
            OtpPattern::Custom { inputmode, .. } => *inputmode,
        }
    }

    fn ranges(&self) -> Vec<(char, char)> {
        match self {
            OtpPattern::Digits => vec![('0', '9')],
            OtpPattern::Alphanumeric => vec![('A', 'Z'), ('a', 'z'), ('0', '9')],
            OtpPattern::Letters => vec![('A', 'Z'), ('a', 'z')],
            OtpPattern::Custom { class, .. } => parse_class(class),
        }
    }

    /// The HTML `pattern` attribute: `[...]*`, serialised from the same
    /// ranges [`accepts`](Self::accepts) tests, so they cannot drift.
    pub fn html_pattern(&self) -> String {
        let mut out = String::from("[");
        for (lo, hi) in self.ranges() {
            push_class_char(&mut out, lo);
            if hi != lo {
                out.push('-');
                push_class_char(&mut out, hi);
            }
        }
        out.push_str("]*");
        out
    }

    /// A JavaScript regular-expression source matching one REJECTED
    /// character (`[^...]`), built from the same ranges as the HTML pattern.
    fn js_reject_source(&self) -> String {
        let html = self.html_pattern();
        format!("[^{}]", &html[1..html.len() - 2])
    }

    /// Whether `c` is an accepted character.
    pub fn accepts(&self, c: char) -> bool {
        self.ranges().iter().any(|&(lo, hi)| lo <= c && c <= hi)
    }

    /// `value` with every rejected character removed.
    pub fn filter(&self, value: &str) -> String {
        let ranges = self.ranges();
        value
            .chars()
            .filter(|&c| ranges.iter().any(|&(lo, hi)| lo <= c && c <= hi))
            .collect()
    }
}

/// Parses a character-class body into inclusive ranges. Infallible: a
/// dangling `-` is a literal, a reversed range is dropped.
fn parse_class(class: &str) -> Vec<(char, char)> {
    let chars: Vec<char> = class.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    // Reads one (possibly escaped) literal at `i`, returning it and the next index.
    let atom = |i: usize| -> Option<(char, usize)> {
        match chars.get(i)? {
            '\\' => chars
                .get(i + 1)
                .map(|&c| (c, i + 2))
                .or(Some(('\\', i + 1))),
            &c => Some((c, i + 1)),
        }
    };
    while let Some((lo, next)) = atom(i) {
        if chars.get(next) == Some(&'-') {
            if let Some((hi, after)) = atom(next + 1) {
                if lo <= hi {
                    out.push((lo, hi));
                }
                i = after;
                continue;
            }
        }
        out.push((lo, lo));
        i = next;
    }
    out
}

fn push_class_char(out: &mut String, c: char) {
    // Characters that are syntax or reserved punctuation inside a `v`-flag
    // character class (the flag browsers compile `pattern` with).
    if "()[]{}/-\\|&!#%,:;<=>@`~^".contains(c) {
        out.push('\\');
    }
    out.push(c);
}

#[derive(Clone, Copy)]
struct InputOtpCtx {
    value: Memo<String>,
    /// The slot index the caret currently sits at, or `None` when the real
    /// input isn't focused (or the whole control is disabled) -- no slot is
    /// "active" then, matching a plain text input's own behavior.
    active_index: Memo<Option<usize>>,
    disabled: ReadSignal<bool>,
}

/// The props for the [`InputOtp`] component.
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpProps {
    /// The controlled value of the passcode (each character is one slot).
    pub value: ReadSignal<Option<String>>,

    /// The default value when uncontrolled.
    #[props(default)]
    pub default_value: String,

    /// Called whenever the value changes (typing, backspace/delete, paste).
    #[props(default)]
    pub on_value_change: Callback<String>,

    /// Called once when the value's length reaches `max_length` (shadcn's
    /// `onComplete`) -- fires on the transition into "full", not on every
    /// keystroke made while already full.
    #[props(default)]
    pub on_complete: Callback<String>,

    /// The number of character slots. Must be at least 1; values below 1 are
    /// clamped up to 1.
    #[props(default = 6)]
    pub max_length: usize,

    /// Which characters are accepted. The `<input>`'s `inputmode` and
    /// `pattern` and the filter applied to typed/pasted text all derive from
    /// this one value ([`OtpPattern`]); default digits.
    #[props(default)]
    pub pattern: OtpPattern,

    /// Whether the whole control is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the passcode is required in a form.
    #[props(default)]
    pub required: ReadSignal<bool>,

    /// The name of the input, used in forms.
    #[props(default)]
    pub name: ReadSignal<String>,

    /// Optional id for the real input element -- also what a `<label for>`
    /// should point at, since the boxes are `aria-hidden` presentation.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the real `<input>` -- this is the
    /// accessible surface (`aria-label`, `class`, `data-*`, ...).
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The visual layer: [`InputOtpGroup`]/[`InputOtpSlot`]/
    /// [`InputOtpSeparator`], entirely `aria-hidden`.
    pub children: Element,
}

/// # InputOtp
///
/// A one-time-passcode entry control: one real `<input>` drives every
/// visible box's displayed character and the active-slot/caret highlight.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::input_otp::{InputOtp, InputOtpGroup, InputOtpSlot, InputOtpSeparator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         InputOtp {
///             max_length: 6,
///             aria_label: "Verification code",
///             InputOtpGroup {
///                 for i in 0..3 {
///                     InputOtpSlot { index: i }
///                 }
///             }
///             InputOtpSeparator {}
///             InputOtpGroup {
///                 for i in 3..6 {
///                     InputOtpSlot { index: i }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// [`InputOtpSlot`] defines the following data attributes you can use to
/// control styling:
/// - `data-active`: `true` while this slot holds the caret (the real input
///   is focused and its caret sits at this slot's index).
/// - `data-disabled`: mirrors the control's `disabled` prop.
#[component]
pub fn InputOtp(props: InputOtpProps) -> Element {
    let id = use_unique_id();
    let id = use_id_or(id, props.id);

    let max_length = props.max_length.max(1);
    let disabled = props.disabled;
    let required = props.required;
    let name = props.name;
    let on_complete = props.on_complete;

    let default_value = props.default_value.clone();
    let (value, set_value) =
        use_controlled(props.value, default_value.clone(), props.on_value_change);

    // The browser's own form-reset algorithm restores this `<input>`'s
    // `.value` IDL property from its `value` content attribute -- it has no
    // way to know about, let alone update, this component's own Rust
    // signal. Same fix as `checkbox.rs`'s `BubbleInput::on_reset`, applied
    // directly to the real input here (see this module's doc for why no
    // second hidden input is needed to get there).
    use_form_reset_listener(id, move || set_value.call(default_value.clone()));

    // Fire `on_complete` exactly on the transition into "full" (value's
    // length reaches `max_length`), not on every keystroke made while
    // already full -- matches shadcn's own `onComplete` semantics.
    let previous_value = use_previous(value.into());
    use_effect(move || {
        let current = value();
        if current.chars().count() >= max_length && previous_value().chars().count() < max_length {
            on_complete.call(current);
        }
    });

    let mut selection_start = use_signal(|| 0usize);
    let mut has_focus = use_signal(|| false);
    use_caret_sync(id, selection_start);
    // Filters the DOM value synchronously, in the capture phase of the
    // input's own `input` event, so rejected characters never reach the
    // Rust handler, the slots or the next keystroke (no async write-back to
    // race with fast typing). The caret keeps its place after the accepted
    // characters that preceded it.
    let reject = props.pattern.js_reject_source();
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                const source = await dioxus.recv();
                const el = document.getElementById(id);
                if (!el) return;
                let re;
                try { re = new RegExp(source, 'gv'); } catch (_) { re = new RegExp(source, 'gu'); }
                // A typed character the pattern rejects is cancelled before it
                // lands, so it cannot replace a selected slot's character
                // (click-to-slot) and lose it.
                listen(el, 'beforeinput', (e) => {
                    if (typeof e.data === 'string' && e.data !== '' && e.data.replace(re, '') === '') {
                        e.preventDefault();
                    }
                }, true);
                listen(el, 'input', () => {
                    const raw = el.value;
                    const filtered = raw.replace(re, '');
                    if (filtered === raw) return;
                    const caret = el.selectionStart ?? raw.length;
                    const kept = raw.slice(0, caret).replace(re, '').length;
                    el.value = filtered;
                    el.setSelectionRange(kept, kept);
                }, true);",
            )
            .arg(id.cloned())
            .arg(reject.clone()),
        )
    });

    let active_index = use_memo(move || {
        if !has_focus() || disabled() {
            None
        } else {
            Some(selection_start().min(max_length.saturating_sub(1)))
        }
    });

    use_context_provider(|| InputOtpCtx {
        value,
        active_index,
        disabled,
    });

    // `spellcheck`/`autocapitalize` are overridable defaults; the rest
    // defines this control's own functional shape (numeric-only input,
    // one-time-code autofill, the length bound matching the slot count).
    let defaults = attributes!(input {
        spellcheck: "false",
        autocapitalize: "off",
    });
    let otp_pattern = props.pattern.clone();
    let owned = attributes!(input {
        r#type: "text",
        inputmode: otp_pattern.inputmode().as_attr(),
        pattern: otp_pattern.html_pattern(),
        autocomplete: "one-time-code",
        maxlength: "{max_length}",
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        div {
            position: "relative",
            display: "inline-flex",

            input {
                id: id.cloned(),
                value: value(),
                name,
                required,
                disabled,

                // Fully invisible but still real: still focusable, still
                // receives every keystroke/paste/click, still the thing a
                // screen reader announces as a labelled text field -- only
                // its own rendering (text, caret) is hidden, since the boxes
                // underneath show both instead. `opacity: 0` (not
                // `display: none`/`visibility: hidden`) is what keeps it
                // focusable and hit-testable while hiding both the glyphs
                // and the caret uniformly (caret rendering follows element
                // opacity like everything else about the element).
                position: "absolute",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
                width: "100%",
                height: "100%",
                margin: "0",
                padding: "0",
                border: "none",
                background: "transparent",
                opacity: "0",
                outline: "none",
                cursor: "text",

                onfocus: move |_| has_focus.set(true),
                onblur: move |_| has_focus.set(false),
                // The DOM value is already filtered by the capture-phase
                // listener below (synchronously, before this handler reads
                // it); the Rust filter is the same rule again as the
                // authority for the signal.
                oninput: {
                    let otp_pattern = props.pattern.clone();
                    move |e: FormEvent| set_value.call(otp_pattern.filter(&e.value()))
                },

                // Overrides the browser's own click-to-caret placement,
                // which is wrong here (see `snap_caret_to_slot` and this
                // module's doc): the real input is `opacity: 0` with no
                // `letter-spacing` tuned to the visual slot grid
                // (`preview/src/components/input_otp/style.css`'s
                // `.dx-input-otp-input` sets none), so the browser maps a
                // click's x-coordinate to a caret index using invisible
                // glyph positions that were never aligned to the 40px-wide
                // slot boxes -- e.g. on an empty field, clicking
                // slot 0, 3, or 5 all place the caret at index 0; on a
                // field filled with "123", clicking slot 0 lands at index
                // 2 while slots 1/4/5 all land at index 3. This handler
                // recomputes the intended slot from geometry instead (the
                // slots are `pointer-events: none`, so `elementFromPoint`
                // can't be used -- see the overlay's `pointer-events: none`
                // below) and explicitly moves the caret there,
                // matching upstream `input-otp`'s own approach of not
                // trusting native click-to-glyph mapping at all.
                onclick: move |e: MouseEvent| {
                    if disabled() {
                        return;
                    }
                    let input_id = id.cloned();
                    let value_len = value().chars().count();
                    let point = e.client_coordinates();
                    spawn(async move {
                        if let Some(idx) =
                            snap_caret_to_slot(input_id, value_len, point.x, point.y).await
                        {
                            selection_start.set(idx);
                        }
                    });
                },

                ..merged,
            }

            // Purely visual/presentational -- the real input above is the
            // whole accessibility surface (this module's doc, WCAG generic
            // text-input requirements). `display: contents` so this wrapper
            // adds no layout box of its own between the caller's
            // `InputOtpGroup`(s) and this root.
            //
            // `pointer-events: none` (inherited by every descendant --
            // `InputOtpGroup`/`InputOtpSlot`/`InputOtpSeparator` set none of
            // their own) is the actual hit-target fix: this overlay paints
            // *after*, and therefore on top of, the real input above in the
            // same stacking context, and its `InputOtpSlot` boxes visually
            // cover almost the entire row. Without this, every click/tap
            // lands on a decorative, non-interactive box instead of the real
            // input beneath it: `elementFromPoint` at the dead centre of a
            // visible slot would resolve to that slot's own `<div>`, not
            // the input, and only the few-pixel gaps *between* slots
            // (untouched by any box) would reach the input at all.
            // `pointer-events: none` makes this
            // layer transparent to hit-testing, same as it already is
            // visually (`opacity: 0` on the real input, this module's doc),
            // so every click/tap across the full visible row -- slots,
            // separators, and the gaps between them -- reaches the real
            // input instead of stopping at its decorative overlay.
            div {
                aria_hidden: "true",
                display: "contents",
                pointer_events: "none",
                {props.children}
            }
        }
    }
}

/// Computes which [`InputOtpSlot`] a click at `(client_x, client_y)` landed
/// in by geometry, then explicitly moves the real `<input>`'s native caret
/// there -- overriding the browser's own (wrong, see [`InputOtp`]'s
/// `onclick`) click-to-glyph caret placement.
///
/// The slot boxes are `pointer-events: none` (the overlay's hit-target rule
/// in [`InputOtp`]), so `elementFromPoint` would just return
/// the real input itself, never the slot underneath. Instead this reads
/// every slot's own `getBoundingClientRect()` directly and does a plain
/// point-in-rect test -- a fixed, non-scrolling layout, so this is exact.
/// Slots are queried from `id`'s own `aria-hidden` overlay sibling (its
/// `next_element_sibling` in the DOM, matching this module's own `input {
/// ... } div { aria_hidden: ... }` structure) rather than `document`-wide,
/// so this stays correct with more than one [`InputOtp`] on the same page.
/// Grouping ([`InputOtpGroup`]/[`InputOtpSeparator`]) needs no special
/// handling: every rendered slot carries its own `data-slot-index` and its
/// own rect regardless of which group it's in, so iterating all of them
/// flattens any grouping automatically.
///
/// Clamping rule: the target slot's index is clamped to `min(slot_index,
/// value_len)` -- clicking on or before an existing character goes to that
/// exact position (ordinary text-input UX), while clicking past the last
/// character (an empty, untyped slot) goes to `value_len`, i.e. right after
/// the last real character, the same place any plain text input puts the
/// caret when you click past its end. A click that lands in a gap no slot's
/// rect covers (e.g. a separator) is left alone -- returns `None`, so the
/// caller makes no change and the browser's own (harmless there, since
/// there's no slot grid to misalign with) placement stands.
///
/// Selection, not just a collapsed caret (typing a digit into the clicked
/// field must correct it): when the clamped index falls on an *existing* character
/// (`clamped < value_len`), this selects that one character --
/// `setSelectionRange(clamped, clamped + 1, ...)` -- instead of collapsing
/// to `(clamped, clamped)`. A collapsed caret makes native typing *insert*
/// at that position rather than overwrite, which has two symptoms: on a full value (already at `maxlength`), the browser silently
/// refuses the keystroke outright (inserting would exceed `maxlength`), and
/// on a partial value, typing a digit shifts every character after the
/// caret one slot right instead of correcting the clicked one. Typing over
/// an actual selection *replaces* it -- the standard `<input>` semantics for
/// "correct this character" -- and, because a replacement never grows the
/// string, it also can't hit the `maxlength` insert-block. When `clamped ==
/// value_len` (clicked past the last real character, nothing there to
/// select), the collapsed-caret behavior stays: there is no character at
/// that position for a range to cover.
///
/// Direction: `"forward"` (anchor at `clamped`, focus/caret at `clamped +
/// 1`), not the default `"none"` or `"backward"`. This matches the
/// left-to-right reading order every other caret movement in this component
/// already assumes (`ArrowRight` moving to a higher index, `Delete` acting
/// on the character *at* the caret going forward) and makes
/// Shift+`ArrowRight`/Shift+`ArrowLeft` extend/shrink from the natural end
/// if a caller ever drives this with a real keyboard selection. It has no
/// effect on the unshifted nav covered by this module's tests:
/// unshifted `ArrowLeft`/`ArrowRight` on any non-collapsed selection collapse
/// to that selection's left/right edge regardless of direction (the
/// keyboard-nav case of `playwright/input_otp.spec.ts` holds it), so a
/// single-character selection here behaves exactly like a collapsed caret
/// for every keyboard interaction
/// this component exercises -- the only visible difference is that *typing*
/// over it replaces instead of inserting.
async fn snap_caret_to_slot(
    id: String,
    value_len: usize,
    client_x: f64,
    client_y: f64,
) -> Option<usize> {
    let mut eval = document::eval(
        r#"
        const id = await dioxus.recv();
        const valueLen = await dioxus.recv();
        const x = await dioxus.recv();
        const y = await dioxus.recv();
        const input = document.getElementById(id);
        const overlay = input ? input.nextElementSibling : null;
        const slots = overlay ? Array.from(overlay.querySelectorAll('[data-slot-index]')) : [];
        let target = null;
        for (const slot of slots) {
            const r = slot.getBoundingClientRect();
            if (x >= r.left && x <= r.right && y >= r.top && y <= r.bottom) {
                target = Number(slot.getAttribute('data-slot-index'));
                break;
            }
        }
        if (target !== null && input) {
            const clamped = Math.min(target, valueLen);
            if (clamped < valueLen) {
                // An existing character sits here -- select it so typing
                // replaces it instead of inserting before it.
                input.setSelectionRange(clamped, clamped + 1, 'forward');
            } else {
                // Past the last real character -- nothing to select: the
                // "click to continue typing" collapsed caret.
                input.setSelectionRange(clamped, clamped, 'forward');
            }
            dioxus.send(clamped);
        } else {
            dioxus.send(null);
        }
        "#,
    );
    let _ = eval.send(id);
    let _ = eval.send(value_len);
    let _ = eval.send(client_x);
    let _ = eval.send(client_y);
    eval.recv::<Option<usize>>().await.ok().flatten()
}

/// Keeps a Rust signal in sync with the real `<input>`'s live caret
/// position, so [`InputOtpSlot`] can show which slot currently holds the
/// caret. See this module's doc for why this can't be done from the Dioxus
/// event objects alone (`SelectionEvent`'s `SelectionData` carries no
/// readable range) and reads the DOM directly instead, the same
/// eval-listener-then-stream-back shape as `crate::use_form_reset_listener`/
/// `crate::use_dialog_close_sync`.
fn use_caret_sync(
    id: impl Readable<Target = String> + Copy + 'static,
    mut selection_start: Signal<usize>,
) {
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                const input = document.getElementById(id);
                if (!input) return;
                const send = () => dioxus.send(input.selectionStart ?? input.value.length);
                for (const ev of ['input', 'keyup', 'click', 'focus', 'select']) listen(input, ev, send);",
            )
            .arg(id.cloned())
            .on_message(move |pos: usize| selection_start.set(pos)),
        )
    });
}

/// The props for the [`InputOtpGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpGroupProps {
    /// Additional attributes to apply to the group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The [`InputOtpSlot`]s in this group.
    pub children: Element,
}

/// # InputOtpGroup
///
/// Lays out a cluster of [`InputOtpSlot`]s (e.g. the first half of a
/// 3-3-split six-digit code) with consistent spacing between them. Purely
/// visual -- must be used inside an [`InputOtp`], whose `aria-hidden`
/// wrapper already covers it.
#[component]
pub fn InputOtpGroup(props: InputOtpGroupProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`InputOtpSlot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpSlotProps {
    /// This slot's character index into the passcode value (`0`-based).
    pub index: usize,

    /// Additional attributes to apply to the slot element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # InputOtpSlot
///
/// A single visual box showing one character of the passcode's value at
/// `index`, and a caret indicator (via `data-active`, see [`InputOtp`]'s
/// styling section) while the real input's caret sits at this slot. Must be
/// used inside an [`InputOtp`].
#[component]
pub fn InputOtpSlot(props: InputOtpSlotProps) -> Element {
    let ctx: InputOtpCtx = use_context();
    let index = props.index;

    let value = ctx.value;
    let ch = use_memo(move || {
        value()
            .chars()
            .nth(index)
            .map(|c| c.to_string())
            .unwrap_or_default()
    });
    let active_index = ctx.active_index;
    let active = use_memo(move || active_index() == Some(index));
    let disabled = ctx.disabled;

    let owned = attributes!(div {
        "data-active": active(),
        "data-disabled": disabled(),
        "data-slot-index": "{index}",
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div { ..merged, {ch()} }
    }
}

/// The props for the [`InputOtpSeparator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct InputOtpSeparatorProps {
    /// Additional attributes to apply to the separator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Custom separator content; defaults to a middle dot ("•") when omitted.
    #[props(default)]
    pub children: Option<Element>,
}

/// # InputOtpSeparator
///
/// A purely visual divider between [`InputOtpGroup`]s (e.g. the dash/dot
/// between a 3-3-split six-digit code). Contributes nothing to the
/// accessibility tree -- it sits inside [`InputOtp`]'s `aria-hidden`
/// wrapper alongside the groups/slots it separates.
#[component]
pub fn InputOtpSeparator(props: InputOtpSeparatorProps) -> Element {
    let children = props.children.unwrap_or_else(|| rsx! { "\u{2022}" });
    rsx! {
        div { ..props.attributes, {children} }
    }
}

#[cfg(test)]
mod pattern_tests {
    use super::*;

    #[test]
    fn presets_derive_attribute_and_filter_from_one_source() {
        assert_eq!(OtpPattern::Digits.html_pattern(), "[0-9]*");
        assert_eq!(OtpPattern::Alphanumeric.html_pattern(), "[A-Za-z0-9]*");
        assert_eq!(OtpPattern::Letters.html_pattern(), "[A-Za-z]*");
        assert_eq!(OtpPattern::Digits.filter("1a2-3"), "123");
        assert_eq!(OtpPattern::Alphanumeric.filter("A1 b-2"), "A1b2");
        assert_eq!(OtpPattern::Letters.filter("A1b2"), "Ab");
        assert_eq!(OtpPattern::Digits.inputmode(), OtpInputMode::Numeric);
        assert_eq!(OtpPattern::Letters.inputmode(), OtpInputMode::Text);
    }

    #[test]
    fn custom_class_ranges_literals_and_escapes() {
        let p = OtpPattern::Custom {
            class: "0-9a-f\\-_".into(),
            inputmode: OtpInputMode::Text,
        };
        assert_eq!(p.filter("0fg-_Z9"), "0f-_9");
        assert_eq!(p.html_pattern(), "[0-9a-f\\-_]*");
        let dangling = OtpPattern::Custom {
            class: "ab-".into(),
            inputmode: OtpInputMode::Text,
        };
        assert_eq!(dangling.filter("a-bc"), "a-b");
    }
}
