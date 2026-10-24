//! Top-layer wiring for non-modal overlays — the `popover` attribute.
//!
//! Concept ported from `dignifiedquire/dx-components` (MIT OR Apache-2.0),
//! `primitives/src/top_layer.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99
//! (`use_top_layer`, `TopLayerKind`). Adapted: rewritten against this
//! crate's `document::eval` house pattern (see `use_form_reset_listener`,
//! `lib.rs`) instead of `wasm_bindgen`/`web_sys` — this crate has neither as
//! a dependency, and `dev-docs/archive/fork-research/lifting-from-forks.md` §4 says not to take dependency
//! edits when a lift can avoid them. Narrowed to the Popover API only; the
//! fork's `TopLayerKind::DialogModal` arm is dropped, since native
//! `<dialog>` + `showModal()` is the modal engine's job (`dialog.rs`), not
//! this module's.
//!
//! Two WHATWG HTML spec sections define everything below:
//! - The `popover` attribute and its `auto`/`manual` states:
//!   <https://html.spec.whatwg.org/multipage/popover.html#the-popover-attribute>
//! - Light dismiss (Escape, outside pointerdown, closing sibling `auto`
//!   popovers) for `popover="auto"`:
//!   <https://html.spec.whatwg.org/multipage/popover.html#popover-light-dismiss>
//! - `showPopover()`/`hidePopover()` and the `toggle` event fired on every
//!   state change (browser- or script-driven alike):
//!   <https://html.spec.whatwg.org/multipage/popover.html#dom-showpopover>,
//!   <https://html.spec.whatwg.org/multipage/popover.html#dom-hidepopover>

#[cfg(feature = "web")]
use dioxus::document;
// Unconditional, unlike the rest of this block: `anchored_trigger_attributes`
// (below) is called by every anchored trigger on BOTH arms -- mirroring
// `anchor_name_style`, see its own doc for why -- and needs `Attribute` in
// scope, plus whatever `dioxus_attributes::attributes!`'s expansion resolves
// against (`dioxus_core`/`dioxus_elements`, brought in by this same glob),
// regardless of `web`. A plain glob import is never itself an error when
// some of what it would bring in is `#[cfg]`'d away, so this costs the
// `web` arm nothing.
use dioxus::prelude::*;

#[cfg(feature = "web")]
use std::cell::Cell;

#[cfg(feature = "web")]
thread_local! {
    /// Whether [`ensure_anchor_positioning_styles`]'s `document::eval` has
    /// already been scheduled once in this WASM instance -- same idempotency
    /// guard shape as `scroll_lock.rs`'s
    /// `SCROLLBAR_GUTTER_BASELINE_INSTALLED`.
    static ANCHOR_POSITIONING_STYLES_INSTALLED: Cell<bool> = const { Cell::new(false) };

    /// Whether [`ensure_top_layer_ink_styles`]'s `document::eval` has
    /// already been scheduled once in this WASM instance -- same
    /// idempotency guard shape as `ANCHOR_POSITIONING_STYLES_INSTALLED`
    /// above (itself modeled on `scroll_lock.rs`'s
    /// `SCROLLBAR_GUTTER_BASELINE_INSTALLED`).
    static TOP_LAYER_INK_STYLES_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

/// Installs the shared, engine-level anchor-positioning stylesheet described
/// below, at most once per WASM instance -- mirrors
/// `scroll_lock::ensure_scrollbar_gutter_baseline`'s idempotent
/// style-tag-injection shape exactly (a Rust-side `Cell` guard, plus a
/// JS-side `getElementById` check as a second guard that survives a
/// hot-reload resetting Rust statics but leaving the already-injected tag in
/// the live DOM).
///
/// ## The bug class this closes
///
/// The `@supports (anchor-name: --a)` block
/// (per-`[data-side]`/`[data-align]` `anchor()` rules, `position-try-
/// fallbacks: flip-block, flip-inline`) and the `[popover]` UA-stylesheet
/// reset (`margin: 0; inset: auto;` -- undoing `[popover] { inset: 0; margin:
/// auto; }`, which otherwise invokes CSS's auto-margin centering algorithm
/// and lands content far from its trigger; see `use_anchor_position_fallback`'s
/// doc above) are not hand-copied into *every* consuming page's own
/// `#[css_module]` stylesheet, because `manganis-core`'s `css_module_parser`
/// does not scope classes referenced only inside `@supports`
/// (`dev-docs/issues/css-module-supports-scoping.md`) -- so each copy would
/// have to be a plain, unhashed `dx-anchor-*` marker class rather than a
/// `Styles::`-routed one. A consuming page that lacked its copy
/// silently fell back to un-anchored positioning -- a bug class, not a one-off: every `dx-anchor-*`-marked
/// overlay would have to remember to duplicate this same block into its own
/// page's stylesheet, or fail the identical way. Injecting this once, engine-side, from the same
/// Rust module that defines the marker-class contract
/// (`anchor_name_style`/`position_anchor_style` below) makes anchor
/// positioning "just work" for `dx-anchor-tooltip`/`dx-anchor-hover-card`/
/// `dx-anchor-popover`/`dx-anchor-dropdown-menu`/`dx-anchor-menubar`/
/// `dx-anchor-navbar`/`dx-anchor-navigation-menu`/`dx-anchor-select`/
/// `dx-anchor-combobox` (`select/components/list.rs`'s `SelectListRendered` and
/// `combobox/components/list.rs`'s `ComboboxListRendered`; `dx-anchor-navbar`:
/// `navbar.rs`'s `NavbarContentRendered`)
/// content everywhere in the app, with no per-page CSS to remember.
///
/// Call this once per overlay content mount -- [`use_anchor_position_fallback`]
/// does, since every current `dx-anchor-*` consumer (`tooltip.rs`,
/// `hover_card.rs`, `popover.rs`, `dropdown_menu.rs`, `menubar.rs`,
/// `navbar.rs`, `select/components/list.rs`, `combobox/components/list.rs`,
/// and anything built on `crate::popover` like `ColorPicker`/`DatePicker`)
/// already calls that hook. A future
/// `dx-anchor-*` consumer that doesn't call that hook for some reason should
/// call this directly instead.
///
/// `:where(...)` (zero specificity) is used for the `[popover]` reset so it
/// never has to out-specificity-fight a component's own rules -- author
/// origin already beats the UA stylesheet regardless of specificity (the
/// CSS cascade orders origin before specificity), so zero specificity
/// here is exactly enough to win against the *UA* default while leaving any
/// component-authored override free to still take precedence if one is ever
/// needed.
///
/// `border`/`overflow` resets are deliberately **not** included here, unlike
/// the `margin`/`inset` ones: those two are genuine per-component style
/// choices (`Tooltip` wants no border and never scrolls; `HoverCard`/
/// `Popover` already declare a real border of their own, which -- being
/// author-origin -- already beats the UA default with no reset needed), not
/// a single correct answer every anchored overlay shares the way the
/// centering-trap fix is. Each component's own stylesheet keeps whatever
/// `border`/`overflow` reset it individually needs.
#[cfg(feature = "web")]
pub(crate) fn ensure_anchor_positioning_styles() {
    if ANCHOR_POSITIONING_STYLES_INSTALLED.with(|installed| installed.replace(true)) {
        return;
    }
    let eval = document::eval(&anchor_positioning_inject_js());
    let _ = eval;
}

/// The idempotent (JS-side `getElementById` guarded, so safe to run more
/// than once) style-tag-injection statement [`ensure_anchor_positioning_styles`]
/// dispatches on its own, factored out into a plain JS-snippet-returning
/// function rather than only living inside that eval call, so
/// [`use_anchor_position_fallback`] below can prepend the *exact same*
/// statement to the front of its own measurement script.
///
/// That prepending matters, and is not merely a convenience: this function's
/// statement and a measurement's `getBoundingClientRect()` call dispatched
/// from two *separate* `document::eval()` calls have no guaranteed relative
/// ordering against each other from Rust's point of view (calling
/// `ensure_anchor_positioning_styles()` from a `use_effect` declared just
/// before `use_anchor_position_fallback`'s own measurement effect in the
/// same function, relying on Dioxus's normal same-component effect-ordering
/// guarantee, fails: on a fresh page load, before this stylesheet has ever
/// been injected, the measurement's `reposition()` can still run before the
/// browser has applied the freshly-appended `<style>` tag's rules, so its
/// `matches()` check sees the *pre-anchor* position and wrongly concludes
/// this engine's CSS Anchor Positioning integration has failed, taking over
/// with an inline-style override it never needed to make --
/// `playwright/oracle/tier2-html/top-layer.spec.ts`'s Rule 8 ColorPicker
/// case covers the very first cold load of that page). Two
/// statements inside *one* `document::eval()` call, by contrast, are one
/// synchronous JS script as far as the browser's engine is concerned:
/// `appendChild`ing the `<style>` tag synchronously invalidates style for
/// matching elements, and the very next statement's `getBoundingClientRect()`
/// forces a synchronous style/layout recalc that is guaranteed to already
/// see it -- no ordering assumption about *separate* `document::eval()`
/// dispatches required at all.
#[cfg(feature = "web")]
fn anchor_positioning_inject_js() -> String {
    format!(
        r#"
        if (!document.getElementById('dx-anchor-positioning-styles')) {{
            const style = document.createElement('style');
            style.id = 'dx-anchor-positioning-styles';
            style.textContent = {css};
            document.head.appendChild(style);
        }}
        "#,
        css = ANCHOR_POSITIONING_CSS_JS_LITERAL
    )
}

/// The shared anchor-positioning stylesheet's CSS text, as a JS backtick
/// template-literal source (i.e. this string itself already includes the
/// wrapping backticks) -- see [`ensure_anchor_positioning_styles`]'s doc for
/// the bug class this closes and why `border`/`overflow` resets are
/// deliberately not part of it.
///
/// Every `.dx-anchor-popover[popover]` selector below has a
/// `.dx-anchor-popover:modal` sibling: a modal `Popover`'s web arm
/// is a real `<dialog>` opened with `showModal()`, so it carries the
/// `dx-anchor-popover` marker class the same way the non-modal arm's
/// `<dialog popover="auto">` does, but never the `popover` attribute itself
/// (`showModal()` and the Popover API are two different top-layer
/// mechanisms -- see `popover.rs`'s module doc). `:modal` is the CSS
/// pseudo-class a `<dialog>` matches for as long as it is shown via
/// `showModal()` -- the natural selector to key the exact same reset/
/// `anchor()` treatment off for that arm, mirroring `[popover]`'s role for
/// the Popover-API arms without needing a second marker class.
#[cfg(feature = "web")]
const ANCHOR_POSITIONING_CSS_JS_LITERAL: &str = r#"`
:where(.dx-anchor-tooltip[popover], .dx-anchor-hover-card[popover], .dx-anchor-popover[popover], .dx-anchor-popover:modal, .dx-anchor-dropdown-menu[popover],
  .dx-anchor-menubar[popover], .dx-anchor-navbar[popover], .dx-anchor-navigation-menu[popover], .dx-anchor-select[popover], .dx-anchor-combobox[popover]) {
  margin: 0;
  inset: auto;
}

@supports (anchor-name: --a) {
  .dx-anchor-tooltip[popover],
  .dx-anchor-popover[popover],
  .dx-anchor-popover:modal,
  .dx-anchor-dropdown-menu[popover],
  .dx-anchor-menubar[popover],
  .dx-anchor-navbar[popover],
  .dx-anchor-navigation-menu[popover],
  .dx-anchor-select[popover],
  .dx-anchor-combobox[popover] {
    position: fixed;
    margin: 0;
    inset: auto;
    transform: none;
    position-try-fallbacks: flip-block, flip-inline;
    /* Anchor width contract (docs/backlog.md row 47): publish
       the trigger's width as the same --dx-anchor-width custom property
       use_anchor_position_fallback's JS measurement writes on every
       engine (see that function's doc, "Anchor width contract") -- a CSS-
       native belt on a conforming engine, alongside that JS suspenders
       everywhere else. In Chromium, anchor-size() substitutes correctly
       when only referenced from inside a custom-property declaration and then read
       back through var() at a sizing property's use site -- so this is
       safe to declare generically here, on the marker classes every
       anchored overlay already carries, without knowing anything about a
       specific component's styling intent (that stays in each component's
       own stylesheet, e.g. dx-select-list's min-width: var(--dx-anchor-
       width, 100%)). The JS write is authoritative regardless: it fires
       unconditionally too, right after this rule takes effect, and is
       what actually covers Firefox/WebKit and any engine that reaches
       this file's use_anchor_position_fallback at all -- this declaration
       is only for the sliver of time between paint and that effect's
       first reposition() call, and for engines that resolve anchor-size()
       in a custom property, this simply keeps agreeing with it. Note:
       this comment (and the one on the hover-card rule below) must never
       contain a backtick character -- this whole stylesheet is embedded
       as one JS template-literal string in
       ANCHOR_POSITIONING_CSS_JS_LITERAL below, and a stray backtick here
       would terminate that literal early, producing broken JS (a runtime
       "Invalid left-hand side expression in postfix operation", from
       "--dx-anchor-width" being parsed as JS code instead of string
       content once the literal closed early). */
    --dx-anchor-width: anchor-size(width);
  }

  .dx-anchor-tooltip[popover][data-side="top"],
  .dx-anchor-popover[popover][data-side="top"],
  .dx-anchor-popover:modal[data-side="top"],
  .dx-anchor-dropdown-menu[popover][data-side="top"],
  .dx-anchor-menubar[popover][data-side="top"],
  .dx-anchor-navbar[popover][data-side="top"],
  .dx-anchor-navigation-menu[popover][data-side="top"],
  .dx-anchor-select[popover][data-side="top"],
  .dx-anchor-combobox[popover][data-side="top"] {
    bottom: anchor(top);
    left: anchor(center);
    margin-bottom: 8px;
    transform: translateX(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="right"],
  .dx-anchor-popover[popover][data-side="right"],
  .dx-anchor-popover:modal[data-side="right"],
  .dx-anchor-dropdown-menu[popover][data-side="right"],
  .dx-anchor-menubar[popover][data-side="right"],
  .dx-anchor-navbar[popover][data-side="right"],
  .dx-anchor-navigation-menu[popover][data-side="right"],
  .dx-anchor-select[popover][data-side="right"],
  .dx-anchor-combobox[popover][data-side="right"] {
    top: anchor(center);
    left: anchor(right);
    margin-left: 8px;
    transform: translateY(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="bottom"],
  .dx-anchor-popover[popover][data-side="bottom"],
  .dx-anchor-popover:modal[data-side="bottom"],
  .dx-anchor-dropdown-menu[popover][data-side="bottom"],
  .dx-anchor-menubar[popover][data-side="bottom"],
  .dx-anchor-navbar[popover][data-side="bottom"],
  .dx-anchor-navigation-menu[popover][data-side="bottom"],
  .dx-anchor-select[popover][data-side="bottom"],
  .dx-anchor-combobox[popover][data-side="bottom"] {
    top: anchor(bottom);
    left: anchor(center);
    margin-top: 8px;
    transform: translateX(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="left"],
  .dx-anchor-popover[popover][data-side="left"],
  .dx-anchor-popover:modal[data-side="left"],
  .dx-anchor-dropdown-menu[popover][data-side="left"],
  .dx-anchor-menubar[popover][data-side="left"],
  .dx-anchor-navbar[popover][data-side="left"],
  .dx-anchor-navigation-menu[popover][data-side="left"],
  .dx-anchor-select[popover][data-side="left"],
  .dx-anchor-combobox[popover][data-side="left"] {
    top: anchor(center);
    right: anchor(left);
    margin-right: 8px;
    transform: translateY(-50%);
  }

  .dx-anchor-hover-card[popover] {
    position: fixed;
    margin: 0;
    inset: auto;
    position-try-fallbacks: flip-block, flip-inline;
    /* See the anchor width contract comment on the shared marker-class
       rule above (no backticks in this comment either -- same reason) --
       same reasoning, this selector just lives in its own rule because
       HoverCard uses a wider default gap. */
    --dx-anchor-width: anchor-size(width);
  }

  .dx-anchor-hover-card[popover][data-side="top"] {
    bottom: anchor(top);
    left: anchor(center);
    margin-bottom: 10px;
    transform: translateX(-50%);
  }

  .dx-anchor-hover-card[popover][data-side="right"] {
    top: anchor(center);
    left: anchor(right);
    margin-left: 10px;
    transform: translateY(-50%);
  }

  .dx-anchor-hover-card[popover][data-side="bottom"] {
    top: anchor(bottom);
    left: anchor(center);
    margin-top: 10px;
    transform: translateX(-50%);
  }

  .dx-anchor-hover-card[popover][data-side="left"] {
    top: anchor(center);
    right: anchor(left);
    margin-right: 10px;
    transform: translateY(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="top"][data-align="start"],
  .dx-anchor-hover-card[popover][data-side="top"][data-align="start"],
  .dx-anchor-popover[popover][data-side="top"][data-align="start"],
  .dx-anchor-popover:modal[data-side="top"][data-align="start"],
  .dx-anchor-dropdown-menu[popover][data-side="top"][data-align="start"],
  .dx-anchor-menubar[popover][data-side="top"][data-align="start"],
  .dx-anchor-navbar[popover][data-side="top"][data-align="start"],
  .dx-anchor-navigation-menu[popover][data-side="top"][data-align="start"],
  .dx-anchor-select[popover][data-side="top"][data-align="start"],
  .dx-anchor-combobox[popover][data-side="top"][data-align="start"],
  .dx-anchor-tooltip[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-hover-card[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-popover[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-popover:modal[data-side="bottom"][data-align="start"],
  .dx-anchor-dropdown-menu[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-menubar[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-navbar[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-navigation-menu[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-select[popover][data-side="bottom"][data-align="start"],
  .dx-anchor-combobox[popover][data-side="bottom"][data-align="start"] {
    left: anchor(left);
    transform: none;
  }

  .dx-anchor-tooltip[popover][data-side="top"][data-align="center"],
  .dx-anchor-hover-card[popover][data-side="top"][data-align="center"],
  .dx-anchor-popover[popover][data-side="top"][data-align="center"],
  .dx-anchor-popover:modal[data-side="top"][data-align="center"],
  .dx-anchor-dropdown-menu[popover][data-side="top"][data-align="center"],
  .dx-anchor-menubar[popover][data-side="top"][data-align="center"],
  .dx-anchor-navbar[popover][data-side="top"][data-align="center"],
  .dx-anchor-navigation-menu[popover][data-side="top"][data-align="center"],
  .dx-anchor-select[popover][data-side="top"][data-align="center"],
  .dx-anchor-combobox[popover][data-side="top"][data-align="center"],
  .dx-anchor-tooltip[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-hover-card[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-popover[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-popover:modal[data-side="bottom"][data-align="center"],
  .dx-anchor-dropdown-menu[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-menubar[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-navbar[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-navigation-menu[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-select[popover][data-side="bottom"][data-align="center"],
  .dx-anchor-combobox[popover][data-side="bottom"][data-align="center"] {
    left: anchor(center);
    transform: translateX(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="top"][data-align="end"],
  .dx-anchor-hover-card[popover][data-side="top"][data-align="end"],
  .dx-anchor-popover[popover][data-side="top"][data-align="end"],
  .dx-anchor-popover:modal[data-side="top"][data-align="end"],
  .dx-anchor-dropdown-menu[popover][data-side="top"][data-align="end"],
  .dx-anchor-menubar[popover][data-side="top"][data-align="end"],
  .dx-anchor-navbar[popover][data-side="top"][data-align="end"],
  .dx-anchor-navigation-menu[popover][data-side="top"][data-align="end"],
  .dx-anchor-select[popover][data-side="top"][data-align="end"],
  .dx-anchor-combobox[popover][data-side="top"][data-align="end"],
  .dx-anchor-tooltip[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-hover-card[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-popover[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-popover:modal[data-side="bottom"][data-align="end"],
  .dx-anchor-dropdown-menu[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-menubar[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-navbar[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-navigation-menu[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-select[popover][data-side="bottom"][data-align="end"],
  .dx-anchor-combobox[popover][data-side="bottom"][data-align="end"] {
    left: anchor(right);
    transform: translateX(-100%);
  }

  /* RTL (backlog row 137): align is logical -- start is the INLINE start
     edge, so under dir=rtl a top/bottom panel aligned to start hangs from the
     trigger's right edge and one aligned to end from its left. The panel is a
     DOM descendant of its trigger, so :dir() sees the page direction even in
     the top layer. (left/right sides align on the block axis: unchanged.) */
  .dx-anchor-tooltip[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-hover-card[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-popover[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-popover:modal[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-dropdown-menu[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-menubar[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-navbar[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-navigation-menu[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-select[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-combobox[popover][data-side="top"][data-align="start"]:dir(rtl),
  .dx-anchor-tooltip[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-hover-card[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-popover[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-popover:modal[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-dropdown-menu[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-menubar[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-navbar[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-navigation-menu[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-select[popover][data-side="bottom"][data-align="start"]:dir(rtl),
  .dx-anchor-combobox[popover][data-side="bottom"][data-align="start"]:dir(rtl) {
    left: anchor(right);
    transform: translateX(-100%);
  }

  .dx-anchor-tooltip[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-hover-card[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-popover[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-popover:modal[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-dropdown-menu[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-menubar[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-navbar[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-navigation-menu[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-select[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-combobox[popover][data-side="top"][data-align="end"]:dir(rtl),
  .dx-anchor-tooltip[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-hover-card[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-popover[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-popover:modal[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-dropdown-menu[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-menubar[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-navbar[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-navigation-menu[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-select[popover][data-side="bottom"][data-align="end"]:dir(rtl),
  .dx-anchor-combobox[popover][data-side="bottom"][data-align="end"]:dir(rtl) {
    left: anchor(left);
    transform: none;
  }

  .dx-anchor-tooltip[popover][data-side="left"][data-align="start"],
  .dx-anchor-hover-card[popover][data-side="left"][data-align="start"],
  .dx-anchor-popover[popover][data-side="left"][data-align="start"],
  .dx-anchor-popover:modal[data-side="left"][data-align="start"],
  .dx-anchor-dropdown-menu[popover][data-side="left"][data-align="start"],
  .dx-anchor-menubar[popover][data-side="left"][data-align="start"],
  .dx-anchor-navbar[popover][data-side="left"][data-align="start"],
  .dx-anchor-navigation-menu[popover][data-side="left"][data-align="start"],
  .dx-anchor-select[popover][data-side="left"][data-align="start"],
  .dx-anchor-combobox[popover][data-side="left"][data-align="start"],
  .dx-anchor-tooltip[popover][data-side="right"][data-align="start"],
  .dx-anchor-hover-card[popover][data-side="right"][data-align="start"],
  .dx-anchor-popover[popover][data-side="right"][data-align="start"],
  .dx-anchor-popover:modal[data-side="right"][data-align="start"],
  .dx-anchor-dropdown-menu[popover][data-side="right"][data-align="start"],
  .dx-anchor-menubar[popover][data-side="right"][data-align="start"],
  .dx-anchor-navbar[popover][data-side="right"][data-align="start"],
  .dx-anchor-navigation-menu[popover][data-side="right"][data-align="start"],
  .dx-anchor-select[popover][data-side="right"][data-align="start"],
  .dx-anchor-combobox[popover][data-side="right"][data-align="start"] {
    top: anchor(top);
    transform: none;
  }

  .dx-anchor-tooltip[popover][data-side="left"][data-align="center"],
  .dx-anchor-hover-card[popover][data-side="left"][data-align="center"],
  .dx-anchor-popover[popover][data-side="left"][data-align="center"],
  .dx-anchor-popover:modal[data-side="left"][data-align="center"],
  .dx-anchor-dropdown-menu[popover][data-side="left"][data-align="center"],
  .dx-anchor-menubar[popover][data-side="left"][data-align="center"],
  .dx-anchor-navbar[popover][data-side="left"][data-align="center"],
  .dx-anchor-navigation-menu[popover][data-side="left"][data-align="center"],
  .dx-anchor-select[popover][data-side="left"][data-align="center"],
  .dx-anchor-combobox[popover][data-side="left"][data-align="center"],
  .dx-anchor-tooltip[popover][data-side="right"][data-align="center"],
  .dx-anchor-hover-card[popover][data-side="right"][data-align="center"],
  .dx-anchor-popover[popover][data-side="right"][data-align="center"],
  .dx-anchor-popover:modal[data-side="right"][data-align="center"],
  .dx-anchor-dropdown-menu[popover][data-side="right"][data-align="center"],
  .dx-anchor-menubar[popover][data-side="right"][data-align="center"],
  .dx-anchor-navbar[popover][data-side="right"][data-align="center"],
  .dx-anchor-navigation-menu[popover][data-side="right"][data-align="center"],
  .dx-anchor-select[popover][data-side="right"][data-align="center"],
  .dx-anchor-combobox[popover][data-side="right"][data-align="center"] {
    top: anchor(center);
    transform: translateY(-50%);
  }

  .dx-anchor-tooltip[popover][data-side="left"][data-align="end"],
  .dx-anchor-hover-card[popover][data-side="left"][data-align="end"],
  .dx-anchor-popover[popover][data-side="left"][data-align="end"],
  .dx-anchor-popover:modal[data-side="left"][data-align="end"],
  .dx-anchor-dropdown-menu[popover][data-side="left"][data-align="end"],
  .dx-anchor-menubar[popover][data-side="left"][data-align="end"],
  .dx-anchor-navbar[popover][data-side="left"][data-align="end"],
  .dx-anchor-navigation-menu[popover][data-side="left"][data-align="end"],
  .dx-anchor-select[popover][data-side="left"][data-align="end"],
  .dx-anchor-combobox[popover][data-side="left"][data-align="end"],
  .dx-anchor-tooltip[popover][data-side="right"][data-align="end"],
  .dx-anchor-hover-card[popover][data-side="right"][data-align="end"],
  .dx-anchor-popover[popover][data-side="right"][data-align="end"],
  .dx-anchor-popover:modal[data-side="right"][data-align="end"],
  .dx-anchor-dropdown-menu[popover][data-side="right"][data-align="end"],
  .dx-anchor-menubar[popover][data-side="right"][data-align="end"],
  .dx-anchor-navbar[popover][data-side="right"][data-align="end"],
  .dx-anchor-navigation-menu[popover][data-side="right"][data-align="end"],
  .dx-anchor-select[popover][data-side="right"][data-align="end"],
  .dx-anchor-combobox[popover][data-side="right"][data-align="end"] {
    top: anchor(bottom);
    transform: translateY(-100%);
  }
}
`"#;

/// Installs the shared top-layer ink-color baseline described below, at
/// most once per WASM instance -- same idempotent style-tag-injection shape
/// as [`ensure_anchor_positioning_styles`] (a Rust-side `Cell` guard, plus a
/// JS-side `getElementById` check as a second guard that survives a
/// hot-reload resetting Rust statics but leaving the already-injected tag in
/// the live DOM).
///
/// ## The bug class this closes
///
/// Both the `[popover]` and `<dialog>` UA stylesheets set `color: CanvasText`
/// **directly on the element** -- e.g. Chromium's `html.css` has
/// `[popover] { color: CanvasText; ... }` and `dialog { color: CanvasText;
/// ... }`. A directly-cascaded declaration always beats an *inherited* one,
/// regardless of specificity or source order (CSS Cascade §4.1's "winning
/// declaration" step never even reaches the inheritance fallback once any
/// origin has a declaration for the property on the element itself) -- so
/// any top-layer content wrapper that does not declare its own `color`
/// renders the UA default (opaque black in light mode, `CanvasText`
/// resolving dark-on-dark under a dark color-scheme) instead of this app's
/// ink token (`--secondary-color-4`), no matter how deeply that token is
/// set on an ancestor. `primitives/src/toast.rs`'s
/// `ensure_toast_base_styles` already carries a narrower, single-selector
/// instance of this exact fix (`:where(.dx-toast-base-region[popover]) {
/// color: inherit; }`, dev-docs/backlog.md row 44) for the raw toast fallback
/// only; this generalizes it to every `[popover]`/`<dialog>` element in the
/// app, promoted or not, themed or not.
///
/// `:where([popover]), :where(dialog)` is used -- rather than a plain
/// `[popover], dialog` rule -- for the same reason
/// [`ensure_anchor_positioning_styles`]'s own doc gives for its `:where(...)`
/// selectors: `:where()` always contributes zero specificity, so this rule
/// only ever fills a gap where *nothing* is declared. It can out-cascade the
/// UA default (author origin always beats user-agent origin, at any
/// specificity) but can never out-specificity-fight a component's own
/// `color` rule -- `tooltip.rs`'s `.dx-tooltip-content` (which sets its own
/// `color`) keeps winning, so no change to this rule's
/// selector is needed to protect that.
///
/// A second `:where()` rule (see [`TOP_LAYER_INK_STYLES_INJECT_JS`]'s own
/// comment) closes the identical defect one DOM level deeper: `<button>`/
/// `<input>`/`<select>`/`<textarea>` each carry their *own* direct UA
/// `color` declaration too (`buttontext`/`fieldtext`, never inherited by
/// default), so the wrapper rule's `color: inherit` cannot reach a form
/// control nested inside promoted content (`DatePicker`'s calendar
/// day-grid buttons are the concrete case).
///
/// ## Coverage: every top-layer consumer reaches this, by construction
///
/// This crate has exactly two ways an element ever enters the top layer:
/// the `popover` attribute (`showPopover()`/`hidePopover()`) or a native
/// `<dialog>`'s `showModal()`. Every consumer of either mechanism routes
/// through one of three hooks, and this function is called from all three,
/// so no top-layer surface can bypass it:
/// - [`use_popover_sync`] (below) -- every `popover`-promoted consumer with
///   a plain open/close lifecycle: `tooltip.rs`, `hover_card.rs`,
///   `popover.rs`'s non-modal arm (and therefore `date_picker.rs`/
///   `color_picker.rs`, both built on `popover::PopoverRoot`),
///   `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `navbar.rs`.
/// - [`use_popover_shown_while_mounted`] (below) -- every `popover`-promoted
///   consumer that stays mounted through an exit animation:
///   `toast.rs`, `select/components/list.rs`, `combobox/components/list.rs`
///   (and therefore `select.rs`/`combobox.rs`).
/// - `use_dialog_open_driver` (`lib.rs`) -- every native-`<dialog>`
///   (`showModal()`) consumer: `dialog.rs`, `alert_dialog.rs`, `popover.rs`'s
///   modal arm -- and therefore `preview/src/components/sheet/component.rs`'s
///   `Sheet`, which composes `dialog::DialogRoot`/`DialogContent` rather than
///   opening its own `<dialog>`.
///
/// Calling this from all three (instead of only one, the way
/// [`ensure_anchor_positioning_styles`] is called only from
/// [`use_anchor_position_fallback`]) is deliberate and load-bearing: a modal
/// `Dialog`/`AlertDialog`/`Sheet` never takes the anchored-overlay path at
/// all, so it never calls [`use_anchor_position_fallback`] -- putting this
/// rule only there would leave every native-`<dialog>` consumer unfixed.
///
/// ## First-paint window (dev-docs/backlog.md row 50)
///
/// Calling this from a `use_effect` at each hook above is a **best-effort
/// early install** only -- exactly the same relationship
/// [`ensure_anchor_positioning_styles`] has to
/// [`use_anchor_position_fallback`]'s own measurement eval (see that
/// function's own "Best-effort early install for repeat opens" comment). It
/// is not, by itself, what makes the very first cold-load open correct: two
/// *separate* `document::eval()` calls -- one dispatched from this
/// function's own `use_effect`, another from a *different* effect's
/// `document::eval()` that flips `showPopover()`/`showModal()` -- have no
/// guaranteed relative ordering against each other from Rust's point of
/// view, even when both effects are declared, in that order, in the same
/// function. [`anchor_positioning_inject_js`]'s own doc handles the
/// identical defect class (a fresh page load can run the
/// *dependent* eval before the *injecting* one, despite matching
/// declaration order) by prepending the injection statement directly into
/// the dependent eval's own script, so both run as one synchronous JS
/// program with no cross-eval ordering assumption left to make. This
/// function's own three call sites have the same shape and the same
/// residue (a surface measured mid-window, dev-docs/backlog.md row 50;
/// `top-layer-ink.spec.ts` guards it) -- so [`TOP_LAYER_INK_STYLES_INJECT_JS`] is
/// prepended the identical way, into the "signal -> browser" show scripts
/// in [`use_popover_sync`], [`use_popover_shown_while_mounted`], and
/// `lib.rs`'s `use_dialog_open_driver` (the same three exhaustive call
/// sites listed above) -- see each one's own comment. This function and its
/// `use_effect` call sites stay in place regardless: once genuinely
/// installed (the common case -- most pages open more than one top-layer
/// surface), every later prepend is a same-tick, JS-side
/// `getElementById` no-op, cheaper than re-deriving whether it is needed
/// from Rust.
#[cfg(feature = "web")]
pub(crate) fn ensure_top_layer_ink_styles() {
    if TOP_LAYER_INK_STYLES_INSTALLED.with(|installed| installed.replace(true)) {
        return;
    }
    // The modality writer (`modality.rs`) rides along: this runs when the first overlay MOUNTS, long
    // before any user opens one, so an app without the pre-paint script still has
    // `data-dx-modality` in place by the first interaction. Idempotent.
    let eval = document::eval(&format!(
        "{};{}",
        crate::modality::MODALITY_JS,
        TOP_LAYER_INK_STYLES_INJECT_JS
    ));
    let _ = eval;
}

/// The idempotent (JS-side `getElementById` guarded, so safe to run more
/// than once) style-tag-injection script [`ensure_top_layer_ink_styles`]
/// dispatches on its own, and [`use_popover_sync`]/
/// [`use_popover_shown_while_mounted`]/`lib.rs`'s `use_dialog_open_driver`
/// each *also* prepend verbatim onto the front of their own "signal ->
/// browser" show script -- same two-guard shape as
/// `anchor_positioning_inject_js` above and `toast.rs`'s
/// `TOAST_BASE_STYLES_INJECT_JS`, and the same prepend-don't-sequence
/// reason [`anchor_positioning_inject_js`]'s own doc gives (see
/// [`ensure_top_layer_ink_styles`]'s doc, "First-paint window," for the
/// full account). `pub(crate)`, not private to this module, so `lib.rs`'s
/// `use_dialog_open_driver` -- the third of the three exhaustive
/// call sites, and the only one outside this file -- can prepend it too.
#[cfg(feature = "web")]
pub(crate) const TOP_LAYER_INK_STYLES_INJECT_JS: &str = r#"
if (!document.getElementById('dx-top-layer-ink-styles')) {
    const style = document.createElement('style');
    style.id = 'dx-top-layer-ink-styles';
    style.textContent = `
      /* The theme's popup-root rule (dx-components-theme.css, "Popup root") gives a \`dx-\` popup the
         ROOT ink instead of its host's; this later, equally weak rule would out-rank it with
         \`inherit\`, so it reads the same token and only an app without the theme inherits. */
      :where([popover]), :where(dialog) {
        color: var(--dx-foreground, inherit);
      }

      /* The same defect, one level deeper. Neutralising the wrapper's own
         UA color is not enough for a form control INSIDE it: <button>,
         <input>, <select> and <textarea> each carry their own UA rule
         setting color directly on the element (buttontext / fieldtext), and
         a directly-cascaded value beats an inherited one just as it does for
         [popover]/dialog -- so an ancestor's \`color: inherit\` can never
         reach them. Example: DatePicker's calendar day-grid
         cells (\`.dx-calendar-grid-cell\`, real <button>s) stay black with
         only the wrapper rule above -- \`Calendar\`'s own stylesheet
         is not guaranteed to be loaded on the DatePicker route
         (an unrelated composition concern), so those buttons can have no
         author \`color\` of any kind to fall back to, only their own UA
         default. This rule gives every top-layer-descendant form control
         the same DOM-tree-inherited inbox this file's whole construction is
         about, independent of whether that control's own stylesheet
         happens to be wired up. (Select/Combobox's listboxes are plain
         <div> options, not form controls -- unaffected by this rule either
         way; both carry their own explicit, intentionally different
         \`color: var(--dx-popover-foreground)\` declared directly on the list,
         which wins outright regardless of this rule.) Scoped to
         top-layer descendants rather than applied app-wide: this closes the
         class where it actually bites without silently restyling every
         ordinary button on the page. Still :where(), so any component that
         sets its own color continues to win outright. */
      :where([popover]) :where(button, input, select, textarea),
      :where(dialog) :where(button, input, select, textarea) {
        color: inherit;
      }
    `;
    document.head.appendChild(style);
}
"#;

/// Which `popover` dismissal behaviour an element declares.
/// WHATWG HTML §the-popover-attribute (see module docs for the link).
///
/// Only referenced from the `#[cfg(feature = "web")]` leaf render
/// functions in `tooltip.rs`/`hover_card.rs`/`popover.rs` -- the native
/// (Blitz) arm never renders the `popover` attribute at all (see
/// `use_popover_sync`'s doc) -- so this whole type is `#[cfg]`-gated too,
/// rather than degrading to a dead stub on that target.
#[cfg(feature = "web")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopoverKind {
    /// `popover="auto"` — light-dismisses on Escape and on a pointerdown
    /// outside the popover (and any popovers it is nested in); showing one
    /// auto popover also closes unrelated open ones. Used by the non-modal
    /// [`crate::popover`] arm, whose existing `use_outside_dismiss`/
    /// `use_global_escape_listener` wiring this replaces on the web arm;
    /// also `dropdown_menu.rs`, `menubar.rs`, and `select/components/
    /// list.rs`'s `SelectListRendered`, each of
    /// which layers light dismiss on top of -- not instead of -- their own
    /// existing blur-driven dismissal as a backstop; see each's own doc for
    /// why `auto` is safe there specifically.
    ///
    /// **Opening-gesture contract (dev-docs/backlog.md row 40):** the browser's
    /// light dismiss reads only trusted `pointerdown`/`pointerup`, so an
    /// `auto` popover that becomes open *between* a gesture's `pointerdown`
    /// and `pointerup` is hidden by that same gesture's `pointerup`. An
    /// `auto` popover must therefore only be shown from `click`,
    /// `pointerup`, keyboard, hover-compat or timer handlers -- never from
    /// `onpointerdown`. Pinned by
    /// `playwright/oracle/tier2-html/auto-popover-opening-gesture.spec.ts`.
    Auto,
    /// `popover="manual"` — no light-dismiss; only an explicit
    /// `hidePopover()` (here, driven by our own `open` signal going false)
    /// closes it. Used by [`crate::tooltip`] and [`crate::hover_card`],
    /// which already own their entire open/close lifecycle via hover/focus
    /// and gain nothing from -- and would fight with -- native light
    /// dismiss (a `HoverCardContent` mouseenter re-opens across what would
    /// otherwise be an outside click if `auto` briefly closed it first);
    /// also `context_menu.rs` (a point-opened menu with no persistent
    /// trigger for `auto` to reason about) and `combobox/components/
    /// list.rs`'s `ComboboxListRendered` (`auto` fails there -- see that
    /// component's doc for the concrete failure).
    Manual,
}

#[cfg(feature = "web")]
impl PopoverKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Drive `showPopover()`/`hidePopover()` on the element with the given `id`
/// from `open`, and sync the browser's own `toggle` event — fired on
/// light-dismiss (`auto`), Escape (`auto`), or any other script/UA-driven
/// close — back into `set_open`, so the Rust signal never strands. This is
/// the same defect class `dev-docs/reference/platform-rationale.md` Caveat 1
/// documents for `<dialog>`'s one-way `showModal()`/`close()` binding;
/// the eval-channel
/// pattern that fixes it here is the same one already used by
/// `use_form_reset_listener` (`lib.rs`).
///
/// The show/hide effect checks `:popover-open` before calling either
/// method, mirroring the native-`<dialog>` spike's `dialog.open` guard
/// (`dev-docs/archive/research/phase4-spike-findings.md`, "An `.open` guard") — without it, a
/// redundant call (the signal already agrees with the browser's state)
/// throws `InvalidStateError`.
///
/// # Renderer-support note
///
/// Only a real browser or webview implements the Popover API. Blitz
/// (`dioxus-native`) implements neither the `popover` attribute nor
/// `document::eval` (the latter is a no-op there) —
/// `dev-docs/reference/platform-rationale.md` Caveat 2's capability table shows
/// both at zero. This hook must never run on that target: calling it there
/// would be a silent no-op at best (`eval` swallowed) and, worse, its
/// *caller* rendering the `popover` attribute unconditionally would be
/// actively harmful — Blitz's UA stylesheet borrows Firefox's
/// `[popover]:not(:popover-open) { display: none }` rule without
/// implementing `:popover-open` itself, so any element carrying the
/// attribute there would always match `:not(:popover-open)` and render
/// permanently hidden. See each call site (`tooltip.rs`, `hover_card.rs`,
/// `popover.rs`) for the matching `#[cfg]` split on the attribute itself.
///
/// It is gated on `feature = "web"` (a renderer Cargo feature this crate
/// already defines), not `target_family = "wasm"` — `dioxus-desktop` is a
/// non-wasm binary with a real, working webview `eval`, and belongs on this
/// same web-style arm, not on the native/no-op one `target_family = "wasm"`
/// would wrongly exclude it from. A `target_family = "wasm"` gate would
/// also have the fullstack SSG prerender — a host (non-wasm) binary built with
/// this feature on — render the native-arm markup instead
/// (`dev-docs/reference/platform-rationale.md` Caveat 1). See that caveat
/// for the rule: markup/attribute choice keys off this renderer
/// feature; only genuinely wasm-only *execution* internals (none exist in
/// this hook's body — `document::eval` itself compiles and runs inertly on
/// any renderer with no document context) would still key off
/// `target_family`.
#[cfg(feature = "web")]
pub(crate) fn use_popover_sync(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    set_open: Callback<bool>,
) {
    // Installs `ensure_top_layer_ink_styles`'s engine-injected ink baseline
    // -- see that function's own doc for the bug this closes and why every
    // `use_popover_sync` call site (every plain-lifecycle `[popover]`
    // consumer) is one of the three provably-exhaustive call sites it lists.
    use_effect(ensure_top_layer_ink_styles);

    // Browser -> signal.
    let id_for_listener = id.clone();
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                listen(document.getElementById(id), 'toggle', (e) => dioxus.send(e.newState === 'open'));",
            )
            .arg(id_for_listener.clone())
            .on_message(move |is_open: bool| set_open.call(is_open)),
        )
    });

    // Signal -> browser.
    use_effect(move || {
        let want_open = open.cloned();
        let id = id.clone();
        // `inject` is prepended into this exact script, not left to the
        // separate `use_effect(ensure_top_layer_ink_styles)` above --
        // see `ensure_top_layer_ink_styles`'s doc, "First-paint window,"
        // for why relying on that other effect's `document::eval()` having
        // already run by the time *this* one calls `showPopover()` is not
        // safe to assume, and why prepending (one synchronous script, not
        // two separately-ordered ones) is what actually closes the gap.
        let inject = TOP_LAYER_INK_STYLES_INJECT_JS;
        document::eval(&format!(
            "{inject}
            const el = document.getElementById('{id}');
            if (!el) return;
            const isOpen = el.matches(':popover-open');
            if ({want_open} && !isOpen) el.showPopover();
            if (!{want_open} && isOpen) el.hidePopover();"
        ));
    });
}

/// Variant of [`use_popover_sync`] for content whose *own* `data-state`
/// (Radix-style) animates through a "closed but still mounted" phase --
/// `use_animated_open`-driven content, unlike every other
/// `use_popover_sync` call site. Forwards native/browser closes to
/// `on_native_close` exactly like [`use_popover_sync`]; the difference is
/// entirely on the "signal -> browser" side: `showPopover()` still runs
/// reactively, every time `open` becomes `true` (not once, at mount -- see
/// "Why this exists" below for why once is not enough), but
/// **`hidePopover()` is never called on our own closing path at all** --
/// only a real DOM removal (this component's caller unmounting it once its
/// own exit animation settles) ever takes this content out of the top
/// layer.
///
/// ## Why this exists: two bugs the plain sync hook cannot avoid
///
/// [`use_popover_sync`]'s own "signal -> browser" effect calls
/// `hidePopover()` the instant `open` goes `false`. That is exactly right
/// for content with no exit animation (`Tooltip`/`HoverCard`/`Popover`/
/// `DropdownMenu`/`ContextMenu`/`Menubar`, none of which keep their content
/// mounted through a CSS animate-out -- Radix-style `data-state="closed"`
/// animations on those are cosmetic fades that finish well after the
/// element would already be gone). `SelectList`/`ComboboxList` are
/// different: both render through `use_animated_open`, which
/// deliberately keeps the content mounted with `data-state="closed"` for
/// its whole exit animation (plus a settle hold) before actually
/// unmounting it -- `combobox.spec.ts`'s "keeps filtered options during
/// keyboard close animation" test asserts exactly this, that the
/// `data-state="closed"` element is still visible mid-animation.
///
/// **Bug 1 (animation race).** Calling [`use_popover_sync`] directly with the
/// raw `open` signal fails that test: `hidePopover()` fires on the same
/// tick `open` goes `false`, which (per the UA popover stylesheet's
/// `[popover]:not(:popover-open) { display: none }`) sets `display: none`
/// on the content *before* `use_animated_open`'s own rAF-deferred
/// `getAnimations()` check ever runs, so it observes zero running
/// animations and finishes the close cycle immediately -- the exit
/// animation is skipped outright, and Playwright's `toBeVisible()` on the
/// still-should-be-animating `data-state="closed"` element fails. Never
/// calling `hidePopover()` from the closing path at all -- only from a real
/// unmount, once `use_animated_open` itself has already decided the
/// animation (and its settle hold) is done -- sidesteps this entirely: the
/// exit animation plays out undisturbed on an element that is still very
/// much `:popover-open` the whole time.
///
/// **Bug 2 (reopen-while-still-mounted, `auto` only).** Covered by
/// `combobox.spec.ts`'s "filters and selects with the keyboard" test
/// (it goes red intermittently when this is not handled):
/// `auto`'s native light dismiss has no notion of "this input is the
/// trigger that owns me" (`ComboboxInput`/`SelectTrigger` are never
/// declared as this popover's invoker, and neither a `popovertarget`
/// attribute nor `preventDefault()` on the
/// trigger's own `pointerdown`/`click` stops it; WHATWG light dismiss
/// treats *any* pointerdown outside the popover's own DOM subtree as
/// "outside," full stop). So: select an option (closes, `open` -> `false`,
/// content stays mounted mid-exit-animation per Bug 1 above), then
/// -- while that same instance is *still* mounted, `:popover-open` still
/// `true` -- click the trigger again to reopen. The trigger's own
/// `pointerdown` is classified as "outside" by native light dismiss, which
/// closes the *still-showing* popover for real (a genuine
/// `showPopover`/`hidePopover` pair, not merely a `data-state` flip) --
/// forwarded correctly back into `on_native_close`, setting `open` to
/// `false` -- **immediately followed**, same gesture, by the trigger's own
/// `onclick` reading `open` and calling `set_open(true)` to reopen it. A
/// one-time, mount-only `showPopover()` call never re-fires for this
/// already-mounted instance, so `open` becomes `true` again in
/// Rust/`data-state` while the actual DOM element stays hidden
/// (`:not(:popover-open)`) -- a user-visible "second click does nothing"
/// defect. Reacting to `open` on every transition (not just the first)
/// avoids it the same way [`use_popover_sync`]'s own show effect does:
/// whenever `open` is `true` and the element is not yet `:popover-open`,
/// `showPopover()` runs again, regardless of *why* it had stopped being
/// shown.
///
/// A *native* close (Escape / outside pointerdown, `auto` only) still
/// bypasses the exit animation on its own path -- the browser's own hide
/// algorithm sets `display: none` synchronously, before Rust ever learns
/// about it -- but that is an accepted, well-known limit of the plain
/// Popover API without `@starting-style`/`transition-behavior:
/// allow-discrete` (out of scope here), not something the oracle
/// requires animated; the test Bug 1 targets only exercises a
/// script-driven (Enter-key) close, exactly the path this hook keeps
/// animatable.
#[cfg(feature = "web")]
pub(crate) fn use_popover_shown_while_mounted(
    id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    on_native_close: Callback<bool>,
) {
    // Installs `ensure_top_layer_ink_styles`'s engine-injected ink baseline
    // -- see that function's own doc for the bug this closes and why every
    // `use_popover_shown_while_mounted` call site (every animated-exit
    // `[popover]` consumer) is one of the three provably-exhaustive call
    // sites it lists.
    use_effect(ensure_top_layer_ink_styles);

    // Browser -> signal: forwards every native *close* `toggle` for this
    // element's whole mounted lifetime -- and ONLY closes (dev-docs/backlog.md
    // row 115). `toggle` is dispatched asynchronously (a queued task), so
    // the `toggle(newState: "open")` echo of this hook's own `showPopover()`
    // call can arrive AFTER the signal has already gone `false` again (a
    // focus-out/mouse-leave in the same beat as the open: input events
    // outrank that queued task under main-thread load). Mirroring that
    // stale "open" back would flip `open` to `true` with nothing left to
    // close it -- a tooltip/hover card stuck open forever (calling `blur()`
    // right after `showPopover()` triggers it deterministically).
    // The `open` direction is never needed: this hook's effect below is the
    // only `showPopover()` caller for the element (the signal always opens
    // it, never the browser), so a native "open" can only be an echo of our
    // own call. Native closes (light dismiss, Escape) have no such echo --
    // the browser acted first -- and still sync back.
    let id_for_listener = id.clone();
    crate::js_listener::use_js_listeners(move || {
        Some(
            crate::js_listener::JsListeners::install(
                "const id = await dioxus.recv();
                listen(document.getElementById(id), 'toggle', (e) => { if (e.newState !== 'open') dioxus.send(false); });",
            )
            .arg(id_for_listener.clone())
            .on_message(move |is_open: bool| on_native_close.call(is_open)),
        )
    });

    // Signal -> browser, show-only: reacts to the real `open` signal (not a
    // one-shot mount effect -- see "Bug 2" above for why a reopen needs
    // this to run again) so `showPopover()` fires every time `open` is
    // `true` and the browser doesn't already agree; `hidePopover()` is
    // deliberately never called here at all -- see "Bug 1" above -- only a
    // real unmount ever closes this content.
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        let id = id.clone();
        // See `use_popover_sync`'s matching comment / `ensure_top_layer_
        // ink_styles`'s doc, "First-paint window": prepended into this
        // exact script rather than left to the separate `use_effect(
        // ensure_top_layer_ink_styles)` above, for the same reason.
        let inject = TOP_LAYER_INK_STYLES_INJECT_JS;
        let modality = crate::modality::MODALITY_JS;
        let open_focus = crate::modality::OPEN_FOCUS_JS;
        document::eval(&format!(
            "{inject}
            {modality};
            {open_focus}
            const el = document.getElementById('{id}');
            if (!el || el.matches(':popover-open')) return;
            // A `<dialog popover>` is a dialog panel (the non-modal Popover, so DatePicker and
            // ColorPicker): where focus lands follows the opening modality like a modal's does
            // (`dxOpenFocus`, `modality.rs`). Every other popover (tooltip, hover card, menus,
            // select list) keeps the browser's own focusing step, or none.
            if (el.tagName === 'DIALOG') dxOpenFocus(el, () => el.showPopover());
            else el.showPopover();"
        ));
    });
}

/// JS-measured static positioning fallback/self-check for engines where CSS
/// Anchor Positioning doesn't land the content next to its trigger. It is
/// *required*, not merely a nice-to-have: once an element
/// is promoted to the top layer via `popover`, its containing block becomes
/// the viewport regardless of its own `position` value (see
/// [`position_anchor_style`]'s doc) -- which means the `[data-side]`
/// rules in `tooltip`/`hover_card`/`popover`'s `style.css` (written for the
/// DOM-relative `position: absolute`) do not gracefully degrade into
/// "roughly right, just not collision-aware" on an engine that can't resolve
/// `anchor()`; they degrade into detached from the trigger entirely -- the
/// "tooltip not working" symptom this prevents.
///
/// Deliberately does *not* gate on `CSS.supports('anchor-name: --a')` alone,
/// unlike the CSS `@supports` blocks this pairs with (those stay -- they're
/// what makes Chromium's positioning correct from the very first layout,
/// before this effect's synchronous correction below even runs). On Firefox,
/// `CSS.supports('anchor-name: --a')` reports `true` (the property
/// parses), yet a popover-promoted element's `anchor()`-based `top`/`left`
/// still failed to resolve against its trigger -- a real, engine-specific
/// popover/anchor-positioning integration gap that a syntax-support check
/// cannot see. So instead this *measures the outcome*: it compares the
/// content's actual position to the trigger-relative position this
/// component's own `[data-side]`/`[data-align]` contract promises, and
/// only overrides with inline styles when they disagree by more than a
/// couple of pixels (rounding/subpixel noise). On an engine where `anchor()`
/// already resolves correctly (this repo's Chromium),
/// the computed and actual positions match and nothing is
/// touched -- no extra reflow, no risk of fighting a working anchor
/// resolution.
///
/// ## The flip contract (dev-docs/backlog.md row 10)
///
/// The stylesheets this pairs with (`tooltip`/`hover_card`/`popover`
/// `style.css`, `@supports` blocks) declare
/// `position-try-fallbacks: flip-block, flip-inline` on the anchored
/// `[popover]` rule -- the CSS Anchor Positioning spec's own collision-
/// avoidance primitive
/// (<https://www.w3.org/TR/css-anchor-position-1/#fallback-var>). On an
/// engine that implements it, a preferred placement that doesn't fit the
/// viewport can legitimately paint at the *opposite* side on the relevant
/// axis instead -- e.g. `[data-side="bottom"]` rendering physically above
/// its trigger. That is intentional disagreement with a naive "actual
/// equals `[data-side]`'s own formula" check, and this function runs after
/// that flip has already painted (see the mount-order/timing note further
/// down) -- so this function computes **both** candidate positions (the
/// primary side's own formula, and the same formula with only the relevant
/// axis's side swapped -- top/bottom sides flip on the block axis,
/// left/right sides on the inline axis, matching `flip-block`/`flip-inline`
/// exactly) and accepts the painted position if it matches *either* one.
/// Only when it matches **neither** -- meaning this engine's
/// anchor-positioning integration failed outright, the case this function
/// was written for -- does it override with inline styles at all. This is
/// the one load-bearing rule here: re-checking only the primary formula
/// would see a legitimate CSS flip as "wrong" and stomp it back to the
/// off-viewport primary position, fighting the platform's own fix.
///
/// When it does override (no working anchor-positioning engine at all), it
/// makes the *same* flip decision itself, from plain
/// `getBoundingClientRect()`/`window.inner{Width,Height}` viewport math:
/// if the primary placement would overflow the relevant viewport edge, it
/// uses the flipped placement instead, so legacy engines get the flip too
/// (dev-docs/backlog.md row 10). Resizing WIDTH to fit stays out of scope
/// (a too-wide overlay is rare, and this crate's own themed listboxes/menus
/// already cap their own width independently, e.g. `select/style.css`'s
/// `max-width: calc(100vw - 2rem)`) -- but see "Inline-axis shift" below
/// for the one piece of horizontal shift this function *does* do, and
/// "Vertical shift + size clamp" further below for the same treatment on
/// the other axis, HEIGHT included: unlike width, a too-*tall* overlay is
/// the ordinary case for this crate's own listboxes/menus (more options
/// than a short viewport can hold), so resizing -- capping height, with
/// the content scrolling inside that cap -- is in scope there.
///
/// ## Inline-axis shift
///
/// Flip (both the CSS `position-try-fallbacks` primitive this function
/// mirrors, and its own viewport-math equivalent just above) only swaps
/// `side` to its axis opposite -- block for top/bottom, inline for
/// left/right (the `opposite` map above). For a `side="top"`/`side="bottom"`
/// placement, the horizontal position comes entirely from `align`
/// (start/center/end, computed once against the trigger's own rect in
/// `place()` above); neither CSS (this crate's `@supports` block declares
/// no shift primitive) nor flip checks it against the viewport. A
/// center-aligned overlay wider than the room its trigger has on one side
/// would clip against the viewport's edge on a small screen (iOS Safari has
/// no CSS Anchor Positioning at all, the engine this whole function exists
/// for; the `top-layer.spec.ts` `edge-bottom-*` fixture cases of `Tooltip`
/// and non-modal `Popover` land at `left: -192px`/`-194px` without it) --
/// and `flip-block` could never be the remedy: `side="bottom"` only ever
/// flips to `side="top"`, which changes `top`, never `left`.
///
/// So on this JS-fallback path (below, in the "neither matches" branch),
/// after the flip decision, `target.left` is clamped into
/// `[EDGE_MARGIN, vw - EDGE_MARGIN - cw]` (falling back to flush against the
/// left edge if the content is wider than the viewport has room for at
/// all -- resizing to fit stays out of scope, matching the flip-only note
/// above). It is deliberately not mirrored into the CSS `@supports`
/// contract: no `position-try-fallbacks` keyword shifts along an axis the
/// way this function's clamp does (CSS's own analogous primitive,
/// `position-area`/`margin: auto`-driven "shift", is a materially larger
/// change that is not needed to close the gap on the devices concerned) --
/// so a genuinely CSS-Anchor-Positioning-conforming engine (Chromium among
/// them) is untouched by the clamp, exactly as the `matches()` early-return
/// above keeps it for flip. iOS Safari has no CSS Anchor Positioning
/// support at all, so it always takes this exact fallback path regardless.
/// `top-layer.spec.ts` Rule 12 is the regression oracle, reusing Rule 11's
/// own no-anchor-engine simulation (`stripAnchorSupportsBlock`,
/// `MOBILE_VIEWPORT`).
///
/// ## Vertical shift + size clamp (`dev-docs/backlog.md` row 10 residual
/// scope: "shift/size clamping")
///
/// The inline-axis shift directly above covers the horizontal half of the
/// gap only. Two related gaps exist on the other axis, closed together
/// here, still mirroring this function's existing construction rather than
/// adding a second positioning path:
///
/// 1. **Vertical position shift.** The identical bug class as the inline
///    axis, mirrored onto the block axis: flip
///    only ever swaps `side` to its own axis's opposite (the `opposite` map
///    above), so a `side="left"`/`side="right"` placement's *vertical*
///    position -- like a `side="top"`/`side="bottom"` placement's
///    *horizontal* one -- comes entirely from `align`, computed once
///    against the trigger in `place()` above, and is not otherwise checked
///    against the viewport. It also backstops a `side="top"`/`side="bottom"`
///    placement itself: flip picks whichever of `primary`/`flipped` the
///    `if`/`else if` ladder judges to fit *the axis it swaps*, but
///    neither branch confirms the *chosen* one actually lands
///    on-screen -- only possible to miss when the content is taller than
///    the room on both sides of the trigger, i.e. gap 2's own condition.
///    Clamped into `[EDGE_MARGIN, vh - EDGE_MARGIN - ch]`, the same
///    construction as the horizontal clamp above, `Math.max` for the
///    identical reason (a viewport too short for the content at all -- the
///    *top* edge wins here, matching "the left edge wins" for the
///    horizontal case; gap 2 just below is what actually keeps that
///    overflow from reaching the screen, unlike the horizontal case, which
///    leaves it in place by design).
///
/// 2. **Size clamp.** Unlike width (resizing stays out of scope -- see the
///    note on the flip-contract paragraph above, and this crate's own
///    per-component width caps), a too-*tall* overlay is the ordinary case
///    for every listbox/menu here (`Select`, `Combobox`, `DropdownMenu`,
///    `Menubar`) once its option/item count outgrows a short viewport --
///    simply letting it "run past the edge" the way the horizontal clamp
///    does would mean losing access to every option below the fold with no
///    way to reach it. So: when the content's own natural height (`ch`,
///    already reflecting whatever `max-height` a component's own
///    stylesheet applies -- `combobox/style.css`'s
///    `.dx-combobox-list` declares `max-height: 300px; overflow: hidden
///    auto;`, and `content.offsetHeight` is read *after* that CSS has
///    taken effect) exceeds the room gap 1's clamp settles for
///    (`vh - 2 * EDGE_MARGIN`, `Math.max(0, ...)`-floored for a viewport
///    shorter than `2 * EDGE_MARGIN` itself), this caps `content.style.
///    maxHeight` there and sets `overflowY: 'auto'` so the overlay's own
///    content scrolls instead of continuing to overflow the viewport edge.
///
///    Never fights a component's own sizing (this module's own governing
///    rule -- see [`ensure_anchor_positioning_styles`]'s doc): `ch` already
///    reflects any smaller cap a component's own stylesheet imposes, so
///    this branch only ever engages -- and only ever shrinks further --
///    when even that self-imposed cap still doesn't fit the room this
///    specific open actually has; a component whose own sizing already fits
///    never takes this branch at all. `box-sizing: border-box` is forced
///    alongside `maxHeight`: `ch` is `content.offsetHeight`, always the
///    *border-box* height regardless of the element's own `box-sizing`,
///    but `max-height` the CSS property is defined relative to `box-sizing`
///    -- under the initial `content-box` value, a `max-height` caps only
///    the content box, and padding/border then add back on *top* of that
///    for the rendered `offsetHeight`, silently under-clamping by however
///    much padding/border the element carries (the raw, unthemed
///    `primitives::popover::PopoverContent` of `top-layer.spec.ts` Rule 15's
///    fixture, which has no `.dx-popover-content` styling, overshoots
///    `availableHeight` by ~14px from the UA `[popover]` stylesheet's
///    default padding/border). See the `reposition()` code's own comment on
///    this line for the full account, including why this is a real gap for
///    any themed consumer that doesn't independently declare `border-box`
///    itself (or whose declaration doesn't reach the element), not an
///    artifact of that one fixture. Set as a plain inline style, unlike
///    the width contract (`--dx-anchor-width`, `dev-docs/backlog.md` row 47,
///    below) which publishes a custom property instead -- deliberately, not
///    an inconsistency: an inline style always wins the cascade over *any*
///    author-stylesheet declaration regardless of specificity (CSS
///    Cascading and Inheritance's origin/importance ordering), which is
///    required here and not merely convenient. `select/style.css`'s
///    `.dx-select-list` declares `overflow: visible`, by its own
///    comment "never scrolls (no max-height here) ... collision-aware ...
///    positioning belongs upstream" -- this *is* that upstream, and a real
///    class selector this crate's zero-specificity `:where(...)` engine
///    convention (`ensure_anchor_positioning_styles`'s doc) could never
///    out-specificity-fight without abandoning that convention for every
///    anchored overlay, not just this one -- exactly why the width
///    contract's own opt-in-via-`var()` approach is not reused here: this
///    clamp must apply whether or not a component asked for it, so it needs
///    the one thing in the cascade nothing but `!important` can beat.
///    Cleared, not merely left alone, once no longer needed: this function
///    re-runs on every tracked scroll/resize/`visualViewport` event for the
///    life of the open overlay ("Scroll/resize tracking" below), and a
///    viewport that grows back (an iOS keyboard closing, a window
///    un-shrinking) must let a component's own sizing govern again rather
///    than staying pinned to a stale cap from a smaller moment.
///
/// Both gaps stay JS-fallback-only, deliberately, for the same reason the
/// inline-axis shift does (see that section's own reasoning): a
/// genuinely CSS-Anchor-Positioning-conforming engine (Chromium
/// among them) has the identical latent gap for *position* --
/// `position-try-fallbacks` declares no shift-along-an-axis primitive
/// either engine could use, nothing here is Firefox/WebKit-specific -- and
/// its `@supports` block gains no size-clamp coverage from CSS at all
/// either (no W3C-shipped primitive shrinks an anchored box to fit the way
/// Floating UI's `size()` middleware does in userland JS). That is a real,
/// acknowledged residual on the conforming engine: the clamps are scoped to
/// the JS-fallback gap `dev-docs/backlog.md` row 10 explicitly names,
/// matching the horizontal precedent rather than rewriting a working
/// engine's untouched code path. Regression oracle: `top-layer.spec.ts`
/// Rule 15.
///
/// ## Exclusion zone (`dev-docs/backlog.md` row 210 -- supersedes the
/// main-axis parts of "Vertical shift + size clamp" above)
///
/// The open Combobox list must never cover its own input (seen on an
/// iPhone). The class is "the panel is clamped into the viewport along the
/// axis `side` names", which is the axis the anchor sits on: with the
/// keyboard up the visual viewport is short, a flip that picks a side
/// without checking it fit, and a vertical clamp (`top >= EDGE_MARGIN`)
/// then drags the list ONTO the input; a size clamp measured against the
/// WHOLE viewport rather than the room beside the anchor could never shrink
/// the list far enough to avoid it. The CSS Anchor Positioning arm is not
/// safer on its own: trusting any placement that lands on a contract spot
/// (judged against the LAYOUT viewport, which a keyboard does not shrink)
/// caps nothing.
///
/// The construction, for every caller: **the anchor's box is an exclusion zone on
/// the main axis**, in visual-viewport coordinates.
///
/// 1. *Side by room.* `room[side]` is the space between the anchor's edge (plus the gap)
///    and the viewport edge (minus `EDGE_MARGIN`). The preferred side is kept while the
///    panel fits there, else the opposite side if it fits or simply has more room.
/// 2. *No main-axis shift.* Only the cross-axis clamp survives (sliding along the
///    anchor's edge, Floating UI's `shift` on the alignment axis); nothing moves the
///    panel across the anchor.
/// 3. *Size to the room.* A panel taller (wider, for left/right) than the chosen side's
///    room is capped to it and scrolls itself. The limits are published as
///    `--dx-available-height` / `--dx-available-width` (Base UI's `--available-height`)
///    on every call, on both arms, so a sheet can say
///    `max-height: min(18rem, var(--dx-available-height, 18rem))`
///    (`combobox/style.css` does). The panel's natural size is measured with those
///    caps lifted (in the same synchronous task, so nothing paints) and its scroll
///    offset restored.
/// 4. *The CSS arm is trusted only while it honours the zone.* A placement that
///    matches a contract spot is accepted only if the box is inside the visual viewport
///    and does not intersect the anchor; otherwise the JS path above takes over.
/// 5. *Growth while open.* A `ResizeObserver` on the panel re-places it when its
///    content changes size (typing in a Combobox), and a panel above the anchor is
///    pinned by its BOTTOM edge, so growth extends away from the anchor by
///    construction rather than by a re-measure.
///
/// Regression oracle: `playwright/combobox-placement.spec.ts` (geometry sweep over
/// viewports, input positions, schemes, typing and the list's own scroll).
///
/// ## Anchor width contract (`dev-docs/backlog.md` row 47)
///
/// A styled listbox whose stylesheet says `min-width: 100%` (as
/// `preview/src/components/select/style.css`'s `.dx-select-list` does) means
/// 100% of the *trigger* only while the list is `position: absolute` inside a
/// `position: relative` `Select` root; once promoted to the top layer via
/// `popover` its containing block is the viewport (see
/// [`position_anchor_style`]'s doc), so the same percentage means 100% of the
/// *viewport* and the listbox opens the full width of the viewport. A rule
/// `@supports (anchor-name: --a) { .dx-select-list[popover] { min-width:
/// anchor-size(width); } }` cannot restore trigger-relative sizing on the CSS
/// Anchor Positioning path: this crate's `#[css_module]` class-hashing does
/// not scope selectors written only inside an `@supports` body
/// (`dev-docs/issues/css-module-supports-scoping.md`; the same gap
/// [`ensure_anchor_positioning_styles`]'s doc describes for the shared
/// engine stylesheet -- this is a second instance of it, this time in a
/// *component's own* stylesheet rather than a hand-copied engine snippet),
/// so the rule that ships targets the unhashed `.dx-select-list` while the
/// live element carries `.dx-select-list-<hash>`. And even where the
/// `@supports` route applied, it would only cover the
/// CSS-Anchor-Positioning path -- this function's own fallback
/// override above corrects *position* on every other engine, but never
/// touches `width`, so Firefox/WebKit (wherever this fallback
/// engages) would have no width fix regardless.
///
/// So the contract is engine-level, not a one-off CSS patch: this function's
/// own `reposition()` (the measurement step above, which -- unlike the
/// inline-position override -- runs unconditionally on *every*
/// engine, conforming or not) publishes the trigger's live width as the
/// custom property `--dx-anchor-width` on the content element, every time
/// it measures -- see the `content.style.setProperty('--dx-anchor-width',
/// ...)` call right after `t` is computed, deliberately placed *before*
/// the `matches()` early return so a conforming engine gets it too, and
/// kept fresh by the same scroll/resize/`visualViewport` tracking that
/// re-invokes this function for position. A themed stylesheet then
/// opts in with `min-width: var(--dx-anchor-width, 100%)` (`100%` fallback
/// keeps the native/non-web arm, where the list is still DOM-relative,
/// correct) -- a plain custom property, not a class the engine has any
/// opinion about, so this stays consistent with the rest of this module's
/// contract: the engine publishes *facts* (a trigger's current width) and
/// never bakes in a specific component's styling intent. The shared
/// `@supports` stylesheet (`ANCHOR_POSITIONING_CSS_JS_LITERAL` below) also
/// declares `--dx-anchor-width: anchor-size(width)` on the same marker
/// classes, as a CSS-native belt alongside this JS write's suspenders --
/// Chromium substitutes `anchor-size()` correctly when it is only
/// referenced from inside a custom-property declaration and then read back
/// through `var()` at a sizing property's use site, so it is worth keeping.
/// It is still only a belt, not the whole contract: this
/// JS write is what covers Firefox/WebKit and any engine that
/// reaches this fallback at all, and is authoritative regardless of what
/// the CSS path already set. Regression oracle: `select.spec.ts`'s
/// listbox-width assertion; `top-layer.spec.ts` Rule 13 (fit-content
/// width) runs on this file's own *unstyled* fixture and cannot see a
/// themed component's CSS, so it does not cover this contract -- see that
/// rule's own header doc.
///
/// ## Scroll/resize tracking
///
/// Tracking listeners install unconditionally for the life of every open
/// overlay: an anchored overlay (`ColorPicker`'s popup) would otherwise
/// float viewport-fixed while the page scrolls underneath it, detached from
/// its trigger. It is not gated on this function's own measurement having
/// judged CSS Anchor Positioning non-conforming at open time and taken over
/// with inline styles (the "neither matches" branch below) -- see "iOS
/// keyboard" further down for why. The function
/// tracks its own outcome in the `usingFallback` flag closed over by
/// `reposition()`; once a call sets it, every subsequent call -- including
/// ones driven by the listeners added below -- skips the `matches()` check
/// and unconditionally recomputes and re-applies the inline position. When
/// CSS Anchor Positioning is (still) placing the content correctly,
/// `usingFallback` never flips and `reposition()` keeps returning early
/// after one extra `getBoundingClientRect()` pair per event -- the
/// platform's own `anchor()` re-resolution does the real work there, this
/// is just confirming it still is.
///
/// Capture-phase (so scroll inside a nested scroll container is seen too --
/// `scroll` does not bubble, only capture reaches it from `window`),
/// passive `scroll` and a plain `resize` listener on `window` re-run the
/// same `reposition()` function, rAF-throttled (a `rafScheduled` flag drops
/// any additional event that fires before the next paint) so a fast scroll
/// gesture doesn't queue up more synchronous layout work than one
/// measurement per frame. The flip decision is re-evaluated on every
/// re-measure (`reposition()` always recomputes `primary`/`flipped` from
/// the current, live `getBoundingClientRect()`/viewport size), so a resize
/// that changes which side fits gets the same flip treatment the initial
/// placement does. All listeners are removed -- via the same
/// `dioxus.recv()`-gated teardown every other eval-channel hook in this
/// crate uses (see `use_popover_sync` above) -- when `open` goes false or
/// this component unmounts; nothing is left listening on `window` (or
/// `visualViewport`) past the overlay's own lifetime.
/// Deliberately narrow: no `ResizeObserver` on the trigger or content
/// elements for *position* -- only `window` (and `visualViewport`)
/// `scroll`/`resize`, which is all a *position* (not size) correction
/// needs. Re-running `reposition()` on every tracked event re-applies the
/// inline-axis shift (see that section above) exactly as it
/// re-applies flip -- both live inside the same function, so a scroll or
/// resize that changes how much room the trigger has re-clamps the same way
/// the initial placement does, with no separate wiring needed.
///
/// ## iOS keyboard
///
/// On iOS Safari the `Combobox` options list would render on top of its own
/// search input after focusing/typing, from two gaps, both closed here:
///
/// 1. **Visual-viewport awareness.** `reposition()` reading only
///    `window.innerWidth`/`innerHeight` plus `getBoundingClientRect()` would
///    miss that iOS Safari's on-screen keyboard shrinks and offsets
///    `window.visualViewport` while leaving `window.inner{Width,Height}`
///    completely unchanged -- so a keyboard-driven layout change would be
///    invisible to every fit/flip computation in this function, regardless
///    of whether anything re-ran it. `viewportMetrics()` below handles it:
///    every `vw`/`vh` read, and the trigger/content rects compared against
///    them, go through `window.visualViewport` (falling back to
///    `documentElement.client*` when absent, i.e. the plain layout viewport
///    on an engine without it) -- see that function's own doc for the
///    coordinate-space bookkeeping this requires (`position: fixed` stays
///    relative to the *layout* viewport even once the *visual* one is what
///    the fit decision needs to reason about). The size is
///    `min(visualViewport, documentElement.client*)` so a classic
///    scrollbar is never counted as room -- see [`VIEWPORT_METRICS_JS`].
/// 2. **Tracking regardless of how the open was placed.** Attaching the
///    scroll/resize listeners only inside the `if (usingFallback)` branch,
///    i.e. only when the very first `reposition()` call had already judged
///    CSS Anchor Positioning non-conforming, would give an overlay that CSS
///    Anchor Positioning placed correctly at *open* time no listener at all
///    for the rest of that open -- "an engine that gets here already
///    re-resolves `anchor()` on scroll for free" is true for scroll, but
///    silently assumes the set of contract-legal positions itself can never
///    change again for the life of that one open. It can: an iOS keyboard
///    appearing is exactly such a change, and Combobox is the one anchored
///    overlay whose opening reliably summons one (a focused text input).
///    So the listener installation sits outside that `if` entirely (see
///    below): tracking is unconditional for the life of every open overlay,
///    and `reposition()`'s own `matches()` check is what keeps a
///    still-conforming engine's re-checks a no-op.
///
/// Ruled out, so as not to be re-tried: neither gap is a
/// "plain no anchor support" problem -- `use_anchor_position_fallback`
/// self-corrects fully and immediately for an overlay with no
/// CSS-anchor support *from open*, byte-identical to native placement (see
/// "The flip contract" above); and a failed trigger lookup (the
/// `document.querySelector` returning `null`) lands the content off-screen
/// entirely, not on top of its own trigger -- neither symptom matches.
/// `playwright/oracle/tier2-html/top-layer.spec.ts` Rule 11 is the
/// regression oracle for both gaps, including the specific "conforming at
/// open, CSS Anchor Positioning support removed mid-open" shape gap 2 is
/// about (that file's own header doc records which of the seven
/// `use_anchor_position_fallback` consumers this sandbox's Chromium can
/// actually be driven into that exact shape for, and why the rest are
/// marked as needing a real device instead).
///
/// `anchor_id` must be the exact same string [`anchor_name_style`] was
/// built from for this overlay's trigger (every call site threads its
/// content's own id through both, so they always match by construction --
/// see `PopoverCtx::content_id`'s doc in `popover.rs` for the bug this
/// guards against for that component specifically). The trigger element
/// itself is found by matching that id's `anchor-name` back out of its
/// inline `style` attribute (a plain substring match -- cheap, and every
/// trigger always sets this style whether or not it ends up used) rather
/// than requiring a second id to be threaded through separately.
///
/// The query includes the closing `;` [`anchor_name_style`] always ends
/// with (`anchor-name: --dxa-{id};`), not just `--dxa-{id}` bare --
/// required, not cosmetic: this repo's
/// `use_unique_id` ids are plain incrementing integers rendered as decimal
/// strings, so an id like `dxc-4` is a literal string *prefix* of `dxc-40`,
/// `dxc-41`, etc. The `Combobox` page is the first
/// consumer of this fallback with enough sibling anchored instances on one
/// page (several demo variants, each contributing its own `ComboboxInput`)
/// to actually hit this: without the trailing `;`, `document.querySelector`
/// for anchor id `dxc-4`'s substring pattern matches a *different* input's
/// `anchor-name: --dxa-dxc-40;` first in DOM order, measuring the wrong
/// trigger's position entirely and landing the listbox hundreds of pixels
/// off-screen (`combobox.spec.ts`'s "controlled value and controlled open
/// stay in sync" test could never reach an option).
/// The `;` is never itself ambiguous across ids: it is not part of any
/// digit sequence, so `dxc-4;` cannot appear inside `dxc-40;`.
#[cfg(feature = "web")]
pub(crate) fn use_anchor_position_fallback(
    id: String,
    anchor_id: String,
    open: impl Readable<Target = bool> + Copy + 'static,
    side: crate::ContentSide,
    align: crate::ContentAlign,
    gap_px: u32,
) {
    // Best-effort early install for repeat opens in this WASM instance --
    // once this has run once (from any overlay), it's a same-tick Rust-side
    // Cell check on every later call, nothing more. It is *not* what makes
    // the very first, cold-load open correct, though: see
    // `anchor_positioning_inject_js`'s doc for why that guarantee instead
    // comes from prepending the identical injection statement onto the
    // measurement eval's own script below.
    use_effect(ensure_anchor_positioning_styles);

    crate::js_listener::use_js_listeners(move || {
        if !open.cloned() {
            return None;
        }
        let id = id.clone();
        let anchor_id = anchor_id.clone();
        let side = side.as_str();
        let align = align.as_str();
        // Prepended into the same `document::eval()` script as the
        // measurement below -- see `anchor_positioning_inject_js`'s doc for
        // why this specific pairing (one script, not two separate `eval()`
        // calls) is what actually guarantees the stylesheet is in effect
        // before `reposition()`'s first `getBoundingClientRect()` call.
        let inject = anchor_positioning_inject_js();
        let viewport_metrics_js = VIEWPORT_METRICS_JS;
        let measure_js = MEASURE_AT_FULL_WIDTH_JS;
        // Synchronous up to and including the first `reposition()` call
        // below, not deferred to `requestAnimationFrame`: this runs from an
        // effect declared (and therefore, on Dioxus's normal mount-order
        // guarantee, run) after `use_popover_sync`'s own effect in every
        // call site, so `showPopover()` has already been dispatched -- and
        // `getBoundingClientRect()`/`offsetWidth` force a synchronous layout
        // in every engine regardless of paint timing, so there is nothing to
        // wait a frame for. This matters, not just simpler: wrapping the body
        // in `requestAnimationFrame(() => {{ ... }})` silently never runs
        // at all on a real Firefox build -- an eval's `document::eval` call
        // is a bare, uncaptured expression whose returned handle can be
        // dropped at the end of a synchronous closure with no ill effect
        // (the same fire-and-forget shape `use_popover_sync`'s own
        // synchronous `document::eval` calls use throughout this file), but
        // dropping it before a callback deferred to a later task/frame ever
        // fires appears to tear down the JS side before that callback runs.
        // A synchronous body has no such window -- which is exactly why,
        // below, the script *is* kept alive (a `JsListeners`, not a dropped
        // eval handle) for as long as tracking might need it: its
        // listeners are real pending callbacks.
        Some(crate::js_listener::JsListeners::install(&format!(
            r#"
            {inject}
            const content = document.getElementById('{id}');
            const trigger = document.querySelector(
                '[style*="anchor-name: --dxa-{anchor_id};"]'
            );
            const align = '{align}';
            const gap = {gap_px};
            const side = '{side}';
            // The only side `position-try-fallbacks: flip-block,
            // flip-inline` can ever swap this one to: top/bottom flip
            // on the block axis, left/right on the inline axis -- each
            // of the four sides has exactly one relevant flip axis, so
            // there is exactly one opposite candidate, not a set of
            // four.
            const opposite = {{ top: 'bottom', bottom: 'top', left: 'right', right: 'left' }}[side];

            // Sticky once flipped true by a `reposition()` call below --
            // see this function's doc, "Scroll/resize tracking": once this
            // engine's anchor-positioning integration has been judged
            // non-conforming, every later re-measure (scroll/resize-driven
            // or not) skips the `matches()` check and unconditionally
            // re-applies the inline position, rather than re-litigating
            // whether CSS anchoring "started working" mid-overlay.
            let usingFallback = false;

            // iOS keyboard handling (see this function's doc, "iOS
            // keyboard"): visual-viewport-aware metrics.
            // `window.innerWidth`/`innerHeight` do NOT shrink when iOS
            // Safari's on-screen keyboard appears; `window.visualViewport`'s
            // `width`/`height` do -- so every fit/flip decision below reads
            // through here instead of `window.inner*` directly (and the
            // helper, `VIEWPORT_METRICS_JS`, also excludes the classic
            // scrollbar via `documentElement.clientWidth/clientHeight`, which
            // `window.inner*` and some engines' `visualViewport` include),
            // and this is the one thing that makes the fix keyboard-*aware* rather than
            // merely keyboard-*re-triggered* (gap 2, fixed separately below,
            // is what makes it re-run at all). `offsetLeft`/`offsetTop` is
            // the visual viewport's own origin offset from the *layout*
            // viewport's (mainly a pinch-zoom-panning concept, included for
            // completeness): `getBoundingClientRect()` always reports
            // layout-viewport-relative coordinates regardless of which
            // viewport is visually showing, and the `position: fixed` this
            // function's override sets is likewise positioned relative to
            // the layout viewport, not the visual one -- so trigger/content
            // rects are shifted into visual-viewport space by this offset
            // before comparison, and shifted back out of it when writing
            // the final inline `top`/`left`. Falls back to
            // `documentElement.client` width/height and `0` on an engine
            // with no `visualViewport` at all. Defined once, shared with
            // `use_point_anchor_clamp`: see `VIEWPORT_METRICS_JS`.
            {viewport_metrics_js}
            {measure_js}

            // Exclusion-zone placement (backlog row 210, see this
            // function's doc, "Exclusion zone"): the anchor's box is a no-go
            // area for the panel on the MAIN axis (the one `side` names), in
            // visual-viewport coordinates, for every caller. EDGE_MARGIN is
            // the gap kept to the viewport edge.
            const EDGE_MARGIN = 4;
            const vertical = side === 'top' || side === 'bottom';

            // The panel's box where the engine placed it, without its own
            // running animations: an open recipe's `translate` (the theme's
            // 4px drop) or scale is already applied at the synchronous first
            // measurement, and would make a correct `anchor()` placement miss
            // `matches()` by its offset and hand a working CSS path to the JS
            // fallback (backlog row 378). Each finite animation is seeked to
            // its end for one read and put back in the same task, so nothing
            // paints and none of them finishes.
            function restingRect(el) {{
                const anims = el.getAnimations().filter((a) => a.currentTime !== null && a.effect
                    && Number.isFinite(a.effect.getComputedTiming().endTime));
                const saved = anims.map((a) => a.currentTime);
                for (const a of anims) a.currentTime = a.effect.getComputedTiming().endTime;
                const r = el.getBoundingClientRect();
                anims.forEach((a, i) => {{ a.currentTime = saved[i]; }});
                return r;
            }}

            function reposition() {{
                if (!content || !trigger) return;
                const t = trigger.getBoundingClientRect();
                // Engine contract (dev-docs/backlog.md row 47):
                // every anchored content's trigger width is published as
                // `--dx-anchor-width`, unconditionally -- not gated on
                // `usingFallback` below -- since this write needs to reach
                // a themed stylesheet's `min-width: var(--dx-anchor-width,
                // ...)` rule regardless of which positioning path this
                // engine takes.
                content.style.setProperty('--dx-anchor-width', t.width + 'px');
                // Measure the panel's NATURAL size: our own caps (the
                // published `--dx-available-*` properties a sheet may read in
                // its `max-height`, and the inline caps the JS path sets
                // below) feed back into the very size the side decision
                // reasons about, so they are dropped, measured around, and
                // written again in this same synchronous task (no frame
                // paints the uncapped state). The panel's own scroll offset
                // is restored because removing a cap can clamp it.
                const keepScroll = content.scrollTop;
                content.style.removeProperty('--dx-available-height');
                content.style.removeProperty('--dx-available-width');
                if (usingFallback) {{
                    content.style.maxHeight = '';
                    content.style.maxWidth = '';
                    content.style.overflowY = '';
                    content.style.overflowX = '';
                    content.style.boxSizing = '';
                }}
                const c = restingRect(content);
                // `let`: re-measured at full width below once this call has
                // committed to overriding the position (`measureAtFullWidth`).
                let cw = content.offsetWidth;
                let ch = content.offsetHeight;
                const vp = viewportMetrics();
                const vw = vp.width;
                const vh = vp.height;
                // See `viewportMetrics()`'s doc just above: re-express the
                // rects `getBoundingClientRect()` gave in layout-viewport
                // coordinates as visual-viewport-relative ones, so they
                // compare like-for-like against `vw`/`vh` in every formula
                // below.
                const tTop = t.top - vp.offsetTop;
                const tBottom = t.bottom - vp.offsetTop;
                const tLeft = t.left - vp.offsetLeft;
                const tRight = t.right - vp.offsetLeft;
                const cTop = c.top - vp.offsetTop;
                const cLeft = c.left - vp.offsetLeft;

                // Room on each side of the anchor, in visual-viewport space:
                // from the anchor's edge (plus the gap) to the viewport edge
                // (minus the margin). This is the space a panel on that side
                // may occupy WITHOUT touching the anchor.
                const room = {{
                    bottom: vh - EDGE_MARGIN - (tBottom + gap),
                    top: tTop - gap - EDGE_MARGIN,
                    right: vw - EDGE_MARGIN - (tRight + gap),
                    left: tLeft - gap - EDGE_MARGIN,
                }};

                // The side to use: the preferred one while the panel fits
                // there, else the opposite one if it fits there or simply has
                // more room (Floating UI's flip with a preferred-side bias).
                // Never anything but the two sides of the `side` axis.
                function chooseSide() {{
                    const fits = (s) => (vertical ? ch <= room[s] : cw <= room[s]);
                    if (fits(side)) return side;
                    return (fits(opposite) || room[opposite] > room[side]) ? opposite : side;
                }}
                let chosen = chooseSide();

                // The size limits the chosen side imposes, published as
                // `--dx-available-height` / `--dx-available-width` (Base UI's
                // `--available-height`) so a sheet can say
                // `max-height: min(18rem, var(--dx-available-height, 18rem))`.
                // On the main axis it is the room beside the anchor; on the
                // cross axis it is the whole viewport minus the margins.
                function availableFor(s) {{
                    return vertical
                        ? {{ h: Math.max(0, room[s]), w: Math.max(0, vw - 2 * EDGE_MARGIN) }}
                        : {{ h: Math.max(0, vh - 2 * EDGE_MARGIN), w: Math.max(0, room[s]) }};
                }}
                function publish(a) {{
                    content.style.setProperty('--dx-available-height', a.h + 'px');
                    content.style.setProperty('--dx-available-width', a.w + 'px');
                }}

                // Same trigger-relative formula the `[data-side]`/
                // `[data-align]` CSS contract promises, parameterised over
                // *which* side is currently being placed. Operates entirely
                // in visual-viewport space (the `t*` locals above).
                // `align` is logical: under RTL the inline start edge is the
                // right one (the panel inherits `direction` from its trigger).
                const rtl = getComputedStyle(content).direction === 'rtl';
                const hAlign = rtl && align === 'start' ? 'end' : (rtl && align === 'end' ? 'start' : align);
                function place(effectiveSide) {{
                    let top;
                    let left;
                    if (effectiveSide === 'top') {{
                        top = tTop - gap - ch;
                    }} else if (effectiveSide === 'bottom') {{
                        top = tBottom + gap;
                    }} else if (align === 'start') {{
                        top = tTop;
                    }} else if (align === 'end') {{
                        top = tBottom - ch;
                    }} else {{
                        top = tTop + (tBottom - tTop) / 2 - ch / 2;
                    }}
                    if (effectiveSide === 'left') {{
                        left = tLeft - gap - cw;
                    }} else if (effectiveSide === 'right') {{
                        left = tRight + gap;
                    }} else if (hAlign === 'start') {{
                        left = tLeft;
                    }} else if (hAlign === 'end') {{
                        left = tRight - cw;
                    }} else {{
                        left = tLeft + (tRight - tLeft) / 2 - cw / 2;
                    }}
                    return {{ top, left }};
                }}

                // Placement contract (published at EVERY exit below, CSS-anchor path and JS
                // fallback alike, so a stylesheet never re-measures or guesses): the side the
                // panel actually sits on (`data-placed-side`, which differs from the requested
                // `data-side` after a flip) and the anchor's centre in the panel's own space
                // (`--dx-anchor-offset-x/-y`, px from the panel's left/top edge; physical, so
                // right for RTL too). `left`/`top` are the panel's final position in the same
                // visual-viewport space as `tLeft`/`tTop`.
                function publishPlacement(placedSide, left, top) {{
                    const r1 = (n) => Math.round(n * 10) / 10;
                    content.setAttribute('data-placed-side', placedSide);
                    content.style.setProperty('--dx-anchor-offset-x', r1(tLeft + (tRight - tLeft) / 2 - left) + 'px');
                    content.style.setProperty('--dx-anchor-offset-y', r1(tTop + (tBottom - tTop) / 2 - top) + 'px');
                }}

                const primary = place(side);
                const flipped = place(opposite);

                // Tolerance for rounding/subpixel noise -- see this
                // function's doc for why this checks the *outcome* rather
                // than trusting a feature-detection flag.
                function matches(pos) {{
                    return Math.abs(cTop - pos.top) <= 2 && Math.abs(cLeft - pos.left) <= 2;
                }}

                if (!usingFallback && (matches(primary) || matches(flipped))) {{
                    // A working anchor-positioning engine placed this at one
                    // of the two contract-legal spots. It is only trusted
                    // while that placement honours the exclusion zone: the
                    // engine judges "fits" against the LAYOUT viewport (which
                    // an on-screen keyboard does not shrink) and caps nothing,
                    // so a box it placed may reach under the keyboard or off
                    // the visual viewport. Either violation (box outside the
                    // visual viewport, or box intersecting the anchor) hands
                    // this call to the JS path below, which is the authority.
                    const cBottom = cTop + ch;
                    const cRight = cLeft + cw;
                    const outside = cTop < -1 || cLeft < -1 || cBottom > vh + 1 || cRight > vw + 1;
                    const overlaps = cLeft < tRight - 1 && cRight > tLeft + 1 && cTop < tBottom - 1 && cBottom > tTop + 1;
                    if (!outside && !overlaps) {{
                        const cssSide = matches(primary) ? side : opposite;
                        publish(availableFor(cssSide));
                        publishPlacement(cssSide, cLeft, cTop);
                        return;
                    }}
                }}

                // Neither matches, or the engine's placement broke the
                // exclusion zone (or a previous call already committed to
                // this branch -- see `usingFallback` above): make the side
                // decision from plain viewport math. Sticky.
                usingFallback = true;
                // Measure-then-move class (see `MEASURE_AT_FULL_WIDTH_JS`):
                // re-measure at the viewport origin (full available width)
                // and redo the side decision with the true size; the final
                // write below is in this same synchronous block, so the
                // probe never paints.
                content.style.position = 'fixed';
                content.style.margin = '0';
                const full = measureAtFullWidth(content, vp);
                cw = full.width;
                ch = full.height;
                chosen = chooseSide();
                const avail = availableFor(chosen);

                // Size clamp: the panel shrinks (and scrolls itself) to the
                // room of the chosen side instead of overlapping the anchor
                // or leaving the viewport. `box-sizing: border-box` is forced
                // alongside the caps because `ch`/`cw` are border-box
                // `offset*` sizes while `max-height` is measured against
                // `box-sizing`; an inline style always wins the cascade over
                // a component's own rule, which is what a clamp that must
                // apply to every consumer needs.
                if (ch > avail.h || cw > avail.w) {{
                    content.style.boxSizing = 'border-box';
                }}
                if (ch > avail.h) {{
                    content.style.maxHeight = avail.h + 'px';
                    content.style.overflowY = 'auto';
                }}
                if (cw > avail.w && !vertical) {{
                    // Width is capped only when it is the MAIN axis (a
                    // left/right placement): a top/bottom panel wider than the
                    // viewport keeps the long-standing "left edge wins" rule.
                    content.style.maxWidth = avail.w + 'px';
                    content.style.overflowX = 'auto';
                }}
                publish(avail);
                ch = content.offsetHeight;
                cw = content.offsetWidth;

                let target = place(chosen);

                // Cross-axis shift (kept, = Floating UI's `shift` on the
                // alignment axis): the panel may slide ALONG the anchor's
                // edge to stay on screen, never across it. top/bottom shift
                // horizontally, left/right vertically. The main axis is
                // deliberately NOT clamped: a clamp there would drag a
                // list onto its own input.
                if (vertical) {{
                    if (target.left < EDGE_MARGIN) {{
                        target = {{ top: target.top, left: EDGE_MARGIN }};
                    }} else if (target.left + cw > vw - EDGE_MARGIN) {{
                        // `Math.max`: on a viewport too narrow for the content
                        // the left edge wins.
                        target = {{ top: target.top, left: Math.max(EDGE_MARGIN, vw - EDGE_MARGIN - cw) }};
                    }}
                }} else if (target.top < EDGE_MARGIN) {{
                    target = {{ top: EDGE_MARGIN, left: target.left }};
                }} else if (target.top + ch > vh - EDGE_MARGIN) {{
                    target = {{ top: Math.max(EDGE_MARGIN, vh - EDGE_MARGIN - ch), left: target.left }};
                }}

                content.style.position = 'fixed';
                content.style.margin = '0';
                content.style.inset = 'auto';
                content.style.transform = 'none';
                content.style.setProperty('position-try-fallbacks', 'none');
                // Shift back out of visual-viewport space (see
                // `viewportMetrics()`'s doc above): `position: fixed` is
                // relative to the layout viewport. A panel above the anchor is
                // pinned by its BOTTOM edge, so content that grows while open
                // (typing in a Combobox) extends upward, away from the anchor,
                // by construction rather than by a re-measure.
                if (chosen === 'top') {{
                    content.style.top = 'auto';
                    content.style.bottom = (vp.cbBottom - (target.top + ch + vp.offsetTop)) + 'px';
                }} else {{
                    content.style.bottom = 'auto';
                    content.style.top = (target.top + vp.offsetTop) + 'px';
                }}
                content.style.left = (target.left + vp.offsetLeft) + 'px';
                publishPlacement(chosen, target.left, target.top);
                if (content.scrollTop !== keepScroll) content.scrollTop = keepScroll;
            }}

            reposition();

            // Settle loop, deferred across animation frames, *only* once the
            // synchronous call just above has already committed to the
            // fallback (`usingFallback`): the synchronous call above can run
            // *before* this content's own children have finished growing into
            // their final size (`date-picker.spec.ts`'s "anchors the popup"
            // case) -- `DatePickerPopover`'s calendar grid, populated
            // by a nested component's own reactive render, still
            // measures an intermediate 352x178px box (narrower-grid/
            // shorter-before-rows placeholder shape) at the exact tick this
            // effect runs, rather than its final 276x278px, throwing off
            // `place()`'s `cw`/`ch`-dependent math (here, the align="center"
            // horizontal centering) by however much the box still has left
            // to grow -- 38px off center on that box's ~76px of subsequent
            // width growth. A **content-size** race, not a scroll/layout-
            // timing one: `getBoundingClientRect()`/`offsetWidth` already
            // force a synchronous style/layout recalc (this function's doc
            // above), so the geometry read back is never stale relative to
            // the DOM as it stood at that instant -- the DOM itself just
            // wasn't finished changing shape yet. A single extra
            // `requestAnimationFrame` call is not enough: it still observes
            // the pre-grow 352px box even several real frames and 300ms
            // later, since whatever later render pass grows the grid does
            // not land within one, or even a handful of, animation frames of
            // this effect's own -- so instead of guessing a frame count, this
            // polls `content.offsetWidth`/`offsetHeight` once per frame and
            // keeps re-running `reposition()` until two consecutive frames
            // read the *same* box size (the settle signal: nothing is still
            // resizing), capped at `MAX_SETTLE_FRAMES` frames as a backstop
            // against a pathological page that never stabilizes.
            //
            // Gated on `usingFallback` -- *not* run unconditionally -- for a
            // reason `top-layer.spec.ts` Rule 8's ColorPicker case pins (an
            // unconditional loop regresses it): once the first call has
            // judged CSS Anchor Positioning conforming, every extra
            // `reposition()` call in a naive unconditional loop re-runs the
            // *same* `matches()` decision from scratch, and each of those
            // re-checks is one more chance for one transient frame --
            // mid-fade-in-animation, mid this-same content-growth window,
            // anything that momentarily moves the content's painted rect
            // off the anchor-computed one by more than the 2px tolerance --
            // to be wrongly read as "the engine failed," latching
            // `usingFallback` true *permanently* (it is deliberately sticky,
            // this function's doc above) for a case that was never actually
            // broken. Only entering this loop when the first call already
            // committed to the fallback removes that risk entirely: from
            // that point on every `reposition()` call skips the `matches()`
            // check outright (`usingFallback` short-circuits the `if` below)
            // and unconditionally re-applies fresh inline geometry -- the
            // decision to accept the fallback's placement is never
            // relitigated, only its geometry is refined. A CSS-Anchor-
            // Positioning engine that placed this correctly on the first
            // call is instead left completely alone by this loop.
            if (usingFallback) {{
                const MAX_SETTLE_FRAMES = 30; // ~0.5s at 60fps
                let lastW = content.offsetWidth;
                let lastH = content.offsetHeight;
                let stableStreak = 0;
                let frame = 0;
                const settle = () => {{
                    frame++;
                    reposition();
                    const w = content.offsetWidth;
                    const h = content.offsetHeight;
                    stableStreak = (w === lastW && h === lastH) ? stableStreak + 1 : 0;
                    lastW = w;
                    lastH = h;
                    if (stableStreak < 2 && frame < MAX_SETTLE_FRAMES) {{
                        requestAnimationFrame(settle);
                    }}
                }};
                requestAnimationFrame(settle);
            }}

            // iOS keyboard, gap 2 -- see this function's doc,
            // "iOS keyboard": tracking listeners install
            // UNCONDITIONALLY, for the life of every open overlay, not only
            // once `usingFallback` is already `true` at this point in the
            // script. An overlay that CSS Anchor Positioning placed
            // correctly at open time still needs them: "an
            // engine that gets here already re-resolves `anchor()` on
            // scroll for free" is true for scroll, but silently
            // assumes the *set of contract-legal positions* itself never
            // changes for the rest of that open. An iOS Safari on-screen
            // keyboard breaks exactly that assumption mid-open (shrinking
            // `visualViewport` out from under an already-placed overlay),
            // and Combobox is the one anchored overlay whose opening
            // reliably summons a keyboard (a focused text input).
            // `reposition()` itself
            // still only overrides when `matches()` fails (or `usingFallback`
            // is already sticky-true), so a still-conforming CSS-anchor
            // engine re-running `reposition()` on every scroll/resize costs
            // one extra `getBoundingClientRect()` pair and returns
            // immediately -- no behavior change for the common case,
            // only for the one this tracking exists to close.
            //
            // `scroll` is added on `window` with `capture: true` because
            // the event does not bubble -- capture on `window` is the only
            // way to observe a scroll on a nested scroll container, not
            // just the document itself. `visualViewport`'s own `resize`
            // (fires when the keyboard opens/closes, or on pinch-zoom) and
            // `scroll` (fires when the visual viewport pans, e.g. while a
            // focused input is scrolled into view above the keyboard) are
            // added too, when present, alongside the existing `window`
            // listeners -- `window`'s own `resize` does not fire for a
            // visual-viewport-only change, since `window.inner` width/
            // height do not change when the keyboard appears (this
            // function's doc, gap 1). rAF-throttled: `rafScheduled`
            // collapses any number of scroll/resize events firing before
            // the next paint (from either source) into a single
            // `reposition()` call. Registered through `listen`, so they are
            // removed on close/unmount exactly as every other eval-channel
            // listener in this file (`JsListeners`' derived teardown).
            let rafScheduled = false;
            const onTrack = () => {{
                if (rafScheduled) return;
                rafScheduled = true;
                requestAnimationFrame(() => {{
                    rafScheduled = false;
                    reposition();
                }});
            }};
            // A scroll INSIDE the panel (the user scrolling the list) cannot
            // move the anchor, and `reposition()` itself briefly lifts the
            // panel's size cap (which can nudge its scroll offset): reacting
            // to it would be a pointless layout pass per scroll frame and
            // could feed back on itself.
            const onScroll = (e) => {{
                if (e.target !== document && content && content.contains(e.target)) return;
                onTrack();
            }};
            listen(window, 'scroll', onScroll, {{ capture: true, passive: true }});
            listen(window, 'resize', onTrack, {{ passive: true }});
            // An anchor inside an ANIMATING ancestor (a submenu's trigger inside its root menu's
            // `dx-pop-in`, scale .95 -> 1) is measured at the animation's current scale by
            // `getBoundingClientRect()`, so a panel opened inside that window is placed against a
            // stale, shrunken box (rtl.spec's Sub "opens to the LEFT" overlapped its trigger by
            // 1.1px). Re-place once the ancestor's animation ends: the anchor is then at rest.
            const onAnimEnd = (e) => {{
                if (trigger && e.target !== content && e.target.contains && e.target.contains(trigger)) onTrack();
            }};
            listen(document, 'animationend', onAnimEnd, {{ capture: true, passive: true }});
            const vv = window.visualViewport;
            listen(vv, 'resize', onTrack, {{ passive: true }});
            listen(vv, 'scroll', onTrack, {{ passive: true }});
            // Content that grows or shrinks while open (typing in a Combobox
            // changes the filtered list) is re-placed in the same frame the
            // browser lays it out, so a list opened upward never extends over
            // its anchor and one opened downward re-checks its room.
            const resizeObserver = typeof ResizeObserver === 'function' && content
                ? new ResizeObserver(onTrack)
                : null;
            if (resizeObserver) {{
                resizeObserver.observe(content);
                cleanup(() => resizeObserver.disconnect());
            }}
            // No cleanup of data-placed-side / --dx-anchor-offset-*: a superseded run's teardown
            // resolves AFTER its successor's first (synchronous) publish and would erase it; the
            // next open republishes before paint, and a closed panel is display:none.
            "#
        )))
    });
}

// No native (Blitz) counterpart: every call site
// (`tooltip.rs`/`hover_card.rs`/`popover.rs`) is itself inside a
// `#[cfg(feature = "web")]`-only component -- the modal/non-modal
// and mount/unmount boundaries in those files are real child-component
// boundaries specifically so the *hook call* can differ per arm without a
// conditional-hook-call hazard (see each file's comment), so a same-named
// no-op stub here would never be called and is dead code by construction,
// like `PopoverKind` and `position_anchor_style` above.

/// The ONE viewport measurement every JS collision/clamp script in this
/// crate shares -- defines `viewportMetrics()` inside whatever `eval` string
/// it is interpolated into ([`use_anchor_position_fallback`]'s
/// `reposition()` and [`use_point_anchor_clamp`] today). Any future script
/// that needs "how much room is there" must splice this in rather than read
/// `window.inner{Width,Height}` or `visualViewport.{width,height}` itself
/// (`viewport_metrics_js_is_the_only_viewport_read` below fails the build
/// otherwise).
///
/// **Why not `window.innerWidth`/`visualViewport.width` alone.** What
/// content can occupy is the viewport *minus the classic scrollbar* -- a
/// 15px gutter on a desktop browser with always-on scrollbars, on both
/// axes. `window.innerWidth/innerHeight` always include it, and engines
/// disagree on whether `visualViewport.width/height` do (Chromium
/// excludes it; a clamp that read the scrollbar-inclusive value would land
/// an overlay `vw - 4` = ~11px past the content area).
/// `document.documentElement.clientWidth/clientHeight` is the layout
/// viewport with scrollbars excluded, by definition, in every engine.
///
/// **Why still `visualViewport`.** It is what shrinks when iOS Safari's
/// on-screen keyboard appears (while `clientHeight` and `innerHeight` do not)
/// and under pinch-zoom (`width = clientWidth / scale`) -- the iOS keyboard
/// handling (`dev-docs/backlog.md` row 28) depends on that. So the
/// usable size is the *smaller* of the two on each axis:
/// `min(visualViewport.size, documentElement.client size)`. Keyboard or
/// zoom: the visual viewport is smaller and wins.
/// Scrollbar on an engine whose `visualViewport` ignores it: `client*` is
/// smaller and wins. Where both already agree (this Chromium): unchanged.
/// `offsetLeft`/`offsetTop` stay purely `visualViewport`'s (the pan origin
/// the callers shift rects by); `0` without it. `|| innerWidth` guards the
/// quirks-mode/detached-root case where `clientWidth` reads `0`.
///
/// **Why also the fixed containing block.** The panels are `position: fixed`,
/// so the box they are placed in is the fixed containing block, which is NOT
/// always `clientWidth`: with `html { scrollbar-gutter: stable }` this
/// Chromium reports `clientWidth` 390 while the containing block (and
/// `html.getBoundingClientRect().width`) is 375, so a `clientWidth` clamp left
/// wide panels (the multi-month date picker) 11-15px past the right edge at
/// 390/600/900px. The clamp box is therefore also measured from a cached
/// `position: fixed; inset: 0` probe (one invisible node, created on first
/// use and re-attached if the page removes it; reading its rect is the same
/// single layout read `clientWidth` costs), and the smallest of the three wins.
///
/// Spliced into the scripts as a function declaration, so it hoists and may
/// be called anywhere in them.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
pub(crate) const VIEWPORT_METRICS_JS: &str = r#"
            function viewportMetrics() {
                const vv = window.visualViewport;
                const root = document.documentElement;
                // The box a `position: fixed` panel is placed in: a cached,
                // invisible `fixed; inset: 0` probe. `clientWidth` is NOT that
                // box under `scrollbar-gutter: stable` (390 vs 375 in this
                // Chromium), so it is only ever the upper bound below.
                let probe = window.__dxFixedCb;
                if (!probe || !probe.isConnected) {
                    probe = document.createElement("div");
                    probe.setAttribute("aria-hidden", "true");
                    probe.setAttribute("data-dx-fixed-cb", "");
                    probe.style.cssText = "position:fixed;inset:0;visibility:hidden;pointer-events:none;";
                    root.appendChild(probe);
                    window.__dxFixedCb = probe;
                }
                const cb = probe.getBoundingClientRect();
                const clientW = Math.min(root.clientWidth || window.innerWidth, cb.width || Infinity);
                const clientH = Math.min(root.clientHeight || window.innerHeight, cb.height || Infinity);
                return {
                    width: vv ? Math.min(vv.width, clientW) : clientW,
                    height: vv ? Math.min(vv.height, clientH) : clientH,
                    offsetLeft: vv ? vv.offsetLeft : 0,
                    offsetTop: vv ? vv.offsetTop : 0,
                    // The fixed containing block's own bottom edge: what a
                    // panel pinned by `bottom:` is measured from.
                    cbBottom: cb.bottom,
                };
            }
"#;

// ---------------------------------------------------------------------------
// Close a non-modal anchored panel when the page scrolls away from it
// (backlog row 193, item 7 of `dev-docs/research/next-round-plan-2026-10-14.md`).
// ---------------------------------------------------------------------------

/// What makes [`use_close_on_scroll`] close a panel.
#[cfg(feature = "web")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollClose {
    /// Any document / scroll-container scroll closes it (Tooltip, HoverCard, menus and Select
    /// when they do not lock the page). These hold nothing a user would mind losing.
    AnyScroll,
    /// Only the ANCHOR leaving the viewport closes it (Popover and the DatePicker / ColorPicker
    /// popovers built on it, Combobox): they can hold half-entered input, and a small scroll that
    /// keeps the anchor in view must not throw it away.
    AnchorLeavesViewport,
}

#[cfg(feature = "web")]
impl ScrollClose {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AnyScroll => "any",
            Self::AnchorLeavesViewport => "anchor",
        }
    }
}

/// The script behind [`use_close_on_scroll`]. Parameters, in order: the panel's id, the anchor's
/// id (`""` for a point-anchored panel such as a context menu), the policy string.
///
/// Every listener goes through `listen` (`js_listener.rs`), so it is removed when the hook's
/// scope ends or `active` goes false, and the scroll listener is `passive`
/// (`scripts/check-blocking-scroll-listeners.sh`). The observer is released through `cleanup`.
///
/// **The mobile keyboard caveat.** Opening the virtual keyboard resizes and pans the VISUAL
/// viewport and makes the browser scroll the focused field into view; if that counted as "the
/// page scrolled", a Combobox would close the moment it is tapped. By construction it cannot:
///  * nothing listens to `visualViewport` scroll/resize as a close signal -- they only extend a
///    quiet window;
///  * a scroll within `QUIET_MS` of a focus change into the panel or the anchor, a key press in
///    them, or a visual-viewport resize/pan is ignored (the browser's focus-driven
///    scroll-into-view lands in that window);
///  * the `AnchorLeavesViewport` policy ignores scrolling altogether and watches the anchor with
///    an `IntersectionObserver`: a focus-driven scroll reveals the field, it never hides it. If the
///    anchor goes out of view inside a quiet window (a keyboard that resized the viewport before
///    the browser scrolled the field back), the decision waits for the window to end.
///
/// A scroll INSIDE the panel (the user scrolling a long list) or inside the anchor (an `<input>`
/// scrolling its own text) is never a reason to close.
#[cfg(feature = "web")]
const CLOSE_ON_SCROLL_JS: &str = r#"
    const id = await dioxus.recv();
    const anchorId = await dioxus.recv();
    const policy = await dioxus.recv();
    const QUIET_MS = 300;
    const content = document.getElementById(id);
    const trigger = anchorId
        ? document.querySelector('[style*="anchor-name: --dxa-' + anchorId + ';"]')
        : null;
    let sent = false;
    const close = () => {
        if (sent) return;
        sent = true;
        dioxus.send(true);
    };
    // Opening a panel focuses things and may scroll them into view.
    let quietUntil = performance.now() + QUIET_MS;
    const quiet = () => { quietUntil = performance.now() + QUIET_MS; };
    const mine = (node) =>
        node instanceof Node &&
        ((content && content.contains(node)) || (trigger && trigger.contains(node)));
    listen(document, 'focusin', (e) => { if (mine(e.target)) quiet(); }, true);
    listen(document, 'keydown', (e) => { if (mine(e.target)) quiet(); }, true);
    const vv = window.visualViewport;
    if (vv) {
        listen(vv, 'resize', quiet, { passive: true });
        listen(vv, 'scroll', quiet, { passive: true });
    }
    if (policy === 'any') {
        // `window` + capture: `scroll` does not bubble, so this is the only way to see a nested
        // scroll container. Document and scroll-container scrolls only -- never the visual viewport.
        listen(window, 'scroll', (e) => {
            if (mine(e.target)) return;
            if (performance.now() < quietUntil) return;
            close();
        }, { capture: true, passive: true });
    } else if (trigger && typeof IntersectionObserver === 'function') {
        // Closes on the in-view -> out-of-view transition only: an anchor that is already out of
        // view when the panel opens is not "leaving". An anchor that goes out of view inside a
        // quiet window (a keyboard resizing the viewport before the browser has scrolled the
        // focused field back into it) is looked at again once the window is over; re-observing
        // delivers a fresh notification with the then-current state.
        let seen = false;
        let timer = 0;
        const observer = new IntersectionObserver((entries) => {
            for (const entry of entries) {
                if (entry.isIntersecting) {
                    seen = true;
                } else if (seen) {
                    const wait = quietUntil - performance.now();
                    if (wait <= 0) {
                        close();
                    } else if (!timer) {
                        timer = setTimeout(() => {
                            timer = 0;
                            observer.unobserve(trigger);
                            observer.observe(trigger);
                        }, wait + 50);
                    }
                }
            }
        });
        observer.observe(trigger);
        cleanup(() => { clearTimeout(timer); observer.disconnect(); });
    }
"#;

/// Closes the panel `content_id` (anchored to `anchor_id`; `""` for a point-anchored one) by
/// calling `set_open(false)` according to `policy`, for as long as `active` is true. The one
/// helper every non-modal anchored primitive calls, so the policy cannot drift per component;
/// modal / scroll-locking surfaces do not call it (they keep their `ScrollLockGuard`).
/// See [`CLOSE_ON_SCROLL_JS`] for the mobile-keyboard caveat.
#[cfg(feature = "web")]
pub(crate) fn use_close_on_scroll(
    content_id: String,
    anchor_id: String,
    active: impl Readable<Target = bool> + Copy + 'static,
    policy: ScrollClose,
    set_open: Callback<bool>,
) {
    crate::js_listener::use_js_listeners(move || {
        if !active.cloned() {
            return None;
        }
        Some(
            crate::js_listener::JsListeners::install(CLOSE_ON_SCROLL_JS)
                .arg(content_id.clone())
                .arg(anchor_id.clone())
                .arg(policy.as_str())
                .on_message(move |_: bool| set_open.call(false)),
        )
    });
}

/// The ONE way a clamp script measures a panel it is about to move:
/// defines `measureAtFullWidth(content, vp)` (returns `{ width, height }`,
/// the border-box `offsetWidth`/`offsetHeight`), spliced in next to
/// [`VIEWPORT_METRICS_JS`].
///
/// **The class this closes (measure-then-move on a shrink-to-fit box).** A
/// `position: fixed` panel with `width: auto` is sized by shrink-to-fit:
/// `min(max-content, room between its `left` and the containing block's right
/// edge)`. So the width a clamp reads depends on the very position the clamp is
/// about to replace. A context menu opened 2px from the right edge would measure its
/// squeezed ~148px instead of its real 160px max-content width, be placed for
/// 148 (`vw - 4 - 148`), then grow back to 160 at that `left` and end flush
/// with the edge instead of `EDGE_MARGIN` away from it. It stays hidden only
/// while a large `min-width` happened to exceed the content.
///
/// **The construction.** Park the panel at the viewport origin (`inset: auto`,
/// `left`/`top` at the visual viewport's own origin, where it has the widest
/// room it can ever have) and read there, so the measured size is the size it
/// will have anywhere the clamp can put it. The caller writes its final
/// `left`/`top` in the same synchronous block, so no frame paints the probe
/// position. Sizes stay `offsetWidth`/`offsetHeight` (layout sizes, unaffected
/// by an open animation's `scale`), never `getBoundingClientRect()`.
#[cfg_attr(not(feature = "web"), allow(dead_code))]
pub(crate) const MEASURE_AT_FULL_WIDTH_JS: &str = r#"
            function measureAtFullWidth(content, vp) {
                content.style.inset = 'auto';
                content.style.left = vp.offsetLeft + 'px';
                content.style.top = vp.offsetTop + 'px';
                return { width: content.offsetWidth, height: content.offsetHeight };
            }
"#;

/// Clamps a **point-anchored** overlay -- one positioned at a raw `(x, y)`
/// coordinate with no anchor element at all -- into the viewport.
/// `dev-docs/backlog.md` row 10, item 5.2: the one piece of that row's
/// collision-detection scope [`use_anchor_position_fallback`] above cannot
/// cover, because that function is keyed on an *anchor element*: it resolves
/// a trigger via `document.querySelector('[style*="anchor-name: ..."]')` and
/// every one of its flip/shift/size-clamp branches reasons in terms of that
/// trigger's own rect (`place()`'s `side`/`align` formulas) and a CSS
/// Anchor Positioning conformance check it can defer to
/// (`matches()`/`usingFallback`). `ContextMenu`'s content has none of that
/// -- it opens at a raw click/long-press point with no persistent trigger
/// element to anchor off of at all (see
/// `context_menu::ContextMenuContentRendered`'s own doc for why `anchor-
/// name`/`position-anchor` are never wired up for it) -- so there is no
/// anchor rect to measure, no flip to compute, and no conforming-engine
/// case to defer to (a raw point is never something the platform's
/// `anchor()` could have placed in the first place). This hook is the
/// "virtual-anchor" construction that row explicitly calls for: the point
/// itself stands in for the anchor.
///
/// **A parallel, smaller construction, not a branch inside
/// [`use_anchor_position_fallback`]**: that function's script is already a
/// long, heavily cross-referenced sequence (flip
/// decision, inline/vertical shift, size clamp, `--dx-anchor-width`
/// publication, scroll/resize/`visualViewport` tracking, a multi-frame
/// settle loop) built and tested entirely around resolving and measuring an
/// anchor element. Threading a no-anchor special case through it would mean
/// either an anchor-shaped no-op path through every one of those stages
/// (`place()` has nothing to compute from, `matches()` has nothing to
/// compare against, `usingFallback` protects a conformance check that never
/// applies here) or an early branch that skips most of the function's body
/// for this one caller -- both add real risk to that function's own
/// passing oracle coverage (`top-layer.spec.ts` rules 5-7, 9, 11-15) for a
/// case that shares only the final clamp *arithmetic*, not the surrounding
/// machinery. This hook instead reuses just that shared arithmetic --
/// the same `EDGE_MARGIN` (4px) convention, the same `Math.max` viewport-
/// too-small-to-fit-at-all fallback, the same `border-box`-forced max-height
/// clamp -- as its own small, independent script, mirroring
/// [`use_anchor_position_fallback`]'s "Vertical shift + size clamp"
/// doc section's construction rather than its code.
///
/// `x`/`y` are `ContextMenuTrigger`'s own already visual-viewport-corrected
/// click coordinates (see that file's `visual_viewport_offset` doc) --
/// i.e. already layout-viewport-relative, exactly matching this content's
/// `position: fixed; left/top` inline styles. Re-expressed here relative to
/// `visualViewport`'s own origin before clamping (mirroring
/// [`use_anchor_position_fallback`]'s `viewportMetrics()`/`tTop`/`tLeft`
/// treatment) so the point being clamped and the bounds it's clamped
/// against are in the same coordinate space, then shifted back before the
/// final inline-style write, since `position: fixed` is relative to the
/// *layout* viewport, not the visual one.
///
/// Measured and clamped once, synchronously, right after the popover is
/// promoted to the top layer -- must be called after [`use_popover_sync`]'s
/// own effect at the call site, the same ordering requirement
/// [`use_anchor_position_fallback`] documents for itself (Dioxus's normal
/// same-component effect-ordering guarantee: this hook's own `use_effect`
/// call must be the later one). Unlike that function, no scroll/resize
/// tracking and no multi-frame settle loop: `ContextMenuContent`'s items
/// are ordinary, synchronously-rendered children (not
/// `DatePickerPopover`'s asynchronously-growing calendar grid --
/// [`use_anchor_position_fallback`]'s own "settle loop" doc), and the menu
/// already suppresses page scroll while open
/// (`ContextMenuContentRendered`'s own wheel/touchmove effect) rather than
/// following it, so there is nothing this hook would need to re-measure
/// against after the initial open.
#[cfg(feature = "web")]
pub(crate) fn use_point_anchor_clamp(
    id: String,
    x: i32,
    y: i32,
    open: impl Readable<Target = bool> + Copy + 'static,
) {
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        let id = id.clone();
        let _ = document::eval(&format!(
            r#"
            {viewport_metrics_js}
            {measure_js}
            const content = document.getElementById('{id}');
            if (content) {{
                const vp = viewportMetrics();
                const vw = vp.width;
                const vh = vp.height;
                const offsetLeft = vp.offsetLeft;
                const offsetTop = vp.offsetTop;
                // Measured at full available width, NOT at the click point:
                // a context menu opened near the right edge is a shrink-to-
                // fit box with almost no room there, so `offsetWidth` at the
                // click point is its squeezed width, and the clamp below would
                // place it for a width it no longer has once moved. The final
                // `left`/`top` writes at the end of this block land before any
                // paint, so the probe position is never shown.
                const full = measureAtFullWidth(content, vp);
                const cw = full.width;
                const ch = full.height;

                // Same EDGE_MARGIN convention as `use_anchor_position_fallback`'s
                // own shift clamp (`dev-docs/backlog.md` row 10) -- see this
                // function's own doc for the full construction this mirrors.
                // `x`/`y` re-expressed relative to `visualViewport`'s own
                // origin so the point and the bounds it's clamped against
                // share one coordinate space; shifted back before the final
                // write below.
                const EDGE_MARGIN = 4;
                let left = {x} - offsetLeft;
                let top = {y} - offsetTop;
                if (left < EDGE_MARGIN) {{
                    left = EDGE_MARGIN;
                }} else if (left + cw > vw - EDGE_MARGIN) {{
                    // `Math.max`: on a viewport too narrow for the content
                    // at all, the left edge wins rather than the two
                    // branches disagreeing about which edge to honor --
                    // matches `use_anchor_position_fallback`'s identical
                    // horizontal clamp exactly (width stays out of scope,
                    // same as there).
                    left = Math.max(EDGE_MARGIN, vw - EDGE_MARGIN - cw);
                }}
                if (top < EDGE_MARGIN) {{
                    top = EDGE_MARGIN;
                }} else if (top + ch > vh - EDGE_MARGIN) {{
                    top = Math.max(EDGE_MARGIN, vh - EDGE_MARGIN - ch);
                }}

                // Size clamp: same construction as `use_anchor_position_
                // fallback`'s own gap 2 ("Vertical shift + size clamp")
                // -- once even a fully top-clamped position
                // still can't fit the content in the viewport's remaining
                // height, shrink instead of shifting further, and let the
                // menu's own items scroll instead of running off the
                // bottom of the screen. `box-sizing: border-box` forced
                // alongside `maxHeight` for the identical reason that
                // function's own doc gives (an inline style, so it always
                // wins the cascade over whatever a themed menu stylesheet
                // does or doesn't declare for `box-sizing`).
                const availableHeight = Math.max(0, vh - 2 * EDGE_MARGIN);
                if (ch > availableHeight) {{
                    content.style.boxSizing = 'border-box';
                    content.style.maxHeight = availableHeight + 'px';
                    content.style.overflowY = 'auto';
                }} else {{
                    content.style.maxHeight = '';
                    content.style.overflowY = '';
                    content.style.boxSizing = '';
                }}

                content.style.left = (left + offsetLeft) + 'px';
                content.style.top = (top + offsetTop) + 'px';
            }}
            "#,
            id = id,
            x = x,
            y = y,
            viewport_metrics_js = VIEWPORT_METRICS_JS,
            measure_js = MEASURE_AT_FULL_WIDTH_JS,
        ));
    });
}

// No native (Blitz) counterpart: `ContextMenuContentRendered`'s native
// (Blitz) arm (`context_menu.rs`) renders the raw click point with no
// clamping at all -- the same functional
// floor every other web-only construction in this file leaves that arm at
// (see the identical note just above, for `use_anchor_position_fallback`).
// Blitz has no popover-API/top-layer support to promote the content into in
// the first place, so this is consistent with the rest of that arm rather
// than a new gap.

/// CSS Anchor Positioning wiring — *not* part of the Popover API itself, but
/// required for it to be usable at all here.
///
/// Moving an element into the top layer changes its containing block to the
/// initial containing block (viewport) regardless of its own `position`
/// value: a `Tooltip` given `popover="manual"`
/// with no other change keeps `position: absolute` but its rendered box
/// jumps hundreds of pixels from its trigger, because "absolute, relative
/// to the nearest positioned ancestor" stops applying once the element
/// is promoted. The `[data-side]` rules in the preview
/// stylesheets (`tooltip`/`hover_card`/`popover` `style.css`) assume a
/// DOM-relative containing block, so without anchoring every non-modal
/// overlay promoted to the top layer would render detached from its trigger.
///
/// CSS Anchor Positioning is the platform's own, still-CSS answer (no
/// `floating-ui`, no JS-computed geometry — that stays out of scope; see
/// `dev-docs/plan.md` "Collision detection", which this is not): an
/// `anchor-name` declared on the trigger and referenced from the content
/// via `position-anchor` lets `top`/`left`/etc. in the *stylesheet* use
/// `anchor()` to mean "relative to that named anchor" even from the top
/// layer. Works in this repo's Chromium
/// (`CSS.supports('anchor-name: --x')` true) — see the preview
/// stylesheets for the `anchor()`-based rules this pairs with, `@supports`-
/// gated so an engine without Anchor Positioning falls back to the plain
/// (non-anchored) rules rather than an invalid declaration.
///
/// Each overlay instance gets its own anchor name, built from its own
/// unique id (already threaded through every one of these components for
/// ARIA), so unrelated instances on the same page never collide. Only
/// meaningful alongside `popover`, so it degrades to an inert empty `style`
/// value off the web arm rather than being `#[cfg]`-gated at every call
/// site.
#[cfg(feature = "web")]
pub(crate) fn anchor_name_style(id: &str) -> String {
    format!("anchor-name: --dxa-{id};")
}

/// See [`anchor_name_style`]. Set on the content element, referencing the
/// same id the trigger's `anchor_name_style` was built from. Only called
/// from the web-arm leaf render functions (the native arm never promotes
/// content to the top layer, so it has no containing-block problem to
/// solve), so -- like [`PopoverKind`] -- this has no native counterpart.
#[cfg(feature = "web")]
pub(crate) fn position_anchor_style(id: &str) -> String {
    format!("position-anchor: --dxa-{id};")
}

/// Fold [`position_anchor_style`] and a caller's own `style` into exactly
/// one `style` attribute, then merge the rest of `attributes` on top.
///
/// Every anchored-content leaf (`TooltipContentRendered`,
/// `HoverCardContentRendered`, `DropdownMenuContentRendered`,
/// `MenubarContentRendered`, `PopoverModalContent`/`PopoverNonModalContent`,
/// `SelectListRendered`, `ComboboxListRendered`) must not set
/// `style: position_anchor_style(&id)` as a bare literal on the element
/// while *also* spreading `..attributes` -- a caller passing its own
/// `style` (shorthand or plain, e.g. the `top_layer` preview fixture's
/// `MenubarContent { style: "min-height: 100px;", .. }`) would then produce two
/// `style` attributes on one tag: the exact duplicate-attribute hazard
/// `dev-docs/conformance-harness.md` hydration-parity Rule 4 documents.
/// `dioxus-ssr` keeps the *first* `style` it renders (`position-anchor` for
/// these components), while the WASM client's `set_attribute` overwrites
/// with the *last* one it applies (the caller's) -- so SSR and CSR silently
/// disagree about which `style` wins, and on the client the anchor binding
/// this whole module exists to set up is lost, breaking CSS Anchor
/// Positioning without any error. `merge_attributes` alone cannot fix this:
/// it dedupes by `(name, namespace)`, and a bare `style: "..."` literal (as
/// written here) is a different attribute from the `attributes` list's own
/// `style`/style-shorthand entries from its point of view, so plain
/// `merge_attributes` never sees them as the same key to begin with.
///
/// So instead: run `attributes` through [`crate::fold_style_attributes`]
/// first (same construction as `ContextMenuTrigger`'s touch-suppression
/// style, `context_menu.rs`) to pull out whatever `style` the caller
/// supplied -- shorthand props, a plain `style` literal, or both -- as one
/// string, prepend `position_anchor_style(id)` to it (anchor binding first
/// so the caller's own declarations, listed after, can still override any
/// property they want without ever being able to drop the anchor binding
/// itself), and merge that single resulting `style` attribute back in with
/// everything else `fold_style_attributes` left untouched. Exactly one
/// `style` attribute reaches the `rsx!` call either way, so SSR and CSR
/// agree by construction and there is nothing left for Rule 4 to catch.
///
/// Call this as the last step before an anchored leaf's `rsx!` block, on
/// whatever `attributes` variable would otherwise be spread with
/// `..attributes` -- including one a call site already ran through its own
/// `merge_attributes` for a marker class or a default `aria-labelledby`, so
/// this never needs a second, separate merge pass for those. Drop the
/// call site's own `style: position_anchor_style(&id)` literal once this is
/// wired in; the returned list already carries that `style`.
#[cfg(feature = "web")]
pub(crate) fn anchored_content_attributes(id: &str, attributes: Vec<Attribute>) -> Vec<Attribute> {
    let (caller_style, rest) = crate::fold_style_attributes(attributes);
    let style = match caller_style {
        Some(caller_style) => format!("{} {caller_style}", position_anchor_style(id)),
        None => position_anchor_style(id),
    };
    crate::merge_attributes(vec![
        dioxus_attributes::attributes!(div { style: "{style}" }),
        rest,
    ])
}

/// No-op whenever this crate's `web` feature is off -- see
/// [`anchor_name_style`]'s doc. Unlike `position_anchor_style`, this one
/// *is* called unconditionally (every trigger sets it, regardless of
/// build), so it needs a real native stub rather than being `#[cfg]`-gated
/// away entirely.
#[cfg(not(feature = "web"))]
pub(crate) fn anchor_name_style(_id: &str) -> String {
    String::new()
}

/// Fold [`anchor_name_style`] and a caller's own `style` into exactly one
/// `style` attribute, then merge the rest of `attributes` on top.
///
/// The trigger-side mirror of `anchored_content_attributes` -- same
/// problem, same fix, the other end of the same anchor pair. Every
/// anchored trigger (`ContextMenuSubTrigger`, `DropdownMenuTrigger`,
/// `DropdownMenuSubTrigger`, `HoverCardTrigger`, `MenubarTrigger`,
/// `NavbarTrigger`, `NavigationMenuTrigger`, `PopoverTrigger`,
/// `TooltipTrigger`) ties itself to its content's `position-anchor` via
/// [`anchor_name_style`]. A bare literal
/// directly alongside a raw `..attributes` spread on the same element
/// would be the exact duplicate-`style` hazard `anchored_content_attributes`'s own
/// doc describes (`dev-docs/conformance-harness.md` hydration-parity Rule 4;
/// `scripts/check-attr-spread-collision.sh` ratchets against it, and
/// `playwright/oracle/attr-synth` synthesizes it directly). A trigger can
/// instead run its own literal through `merge_attributes` before the
/// `rsx!` spread (`TooltipTrigger` does), which *does* avoid that
/// literal HTML duplicate. `merge_attributes` itself folds a plain
/// `style` the way it folds `class` (earlier declarations first), so a
/// caller's `style` does not replace the anchor-name binding through it
/// (an overwrite would silently stop CSS Anchor Positioning the moment any
/// caller set `style` on the trigger; see `lib.rs`'s
/// `style_attribute_is_folded_not_overwritten`). This function exists for
/// what `merge_attributes` cannot do: absorb style set the *shorthand* way (`padding: ".."`, namespace
/// `Some("style")`, a different attribute name to `merge_attributes`), so
/// exactly one `style` reaches the element, and guarantee the anchor binding
/// comes first regardless of which list a call site put it in. So: pull every
/// style-contributing attribute out of `attributes` with
/// [`crate::fold_style_attributes`], prepend the anchor-name binding, and
/// merge the single resulting `style` back in with everything else
/// `fold_style_attributes` left untouched.
///
/// Unlike `anchored_content_attributes`, this carries no `#[cfg(feature =
/// "web")]` guard: every trigger above calls this unconditionally on both
/// arms (see [`anchor_name_style`]'s own doc for why a trigger, unlike a
/// content leaf, is never itself split into separate web/native component
/// fns), so this absorbs the resulting empty anchor-name string itself
/// (contributing nothing to the folded `style` off the web arm) rather than
/// pushing that concern onto every call site.
///
/// Call this as the last step before an anchored trigger's `rsx!` block, on
/// whatever `attributes` variable would otherwise be spread with
/// `..attributes` -- including one a call site already ran through its own
/// `merge_attributes` for an `id`/event handlers/other literals, so this
/// never needs a second, separate merge pass for those. Drop the call
/// site's own `style: anchor_name_style(&id)` literal once this is wired
/// in; the returned list already carries that `style`.
pub(crate) fn anchored_trigger_attributes(id: &str, attributes: Vec<Attribute>) -> Vec<Attribute> {
    let anchor = anchor_name_style(id);
    let (caller_style, rest) = crate::fold_style_attributes(attributes);
    let style = match (anchor.is_empty(), caller_style) {
        (true, None) => None,
        (true, Some(caller_style)) => Some(caller_style),
        (false, None) => Some(anchor),
        (false, Some(caller_style)) => Some(format!("{anchor} {caller_style}")),
    };
    match style {
        Some(style) => crate::merge_attributes(vec![
            dioxus_attributes::attributes!(div { style: "{style}" }),
            rest,
        ]),
        None => rest,
    }
}

#[cfg(all(test, feature = "web"))]
mod tests {
    use super::*;
    use dioxus_core::AttributeValue::Text;

    fn plain_style(value: &str) -> Attribute {
        Attribute {
            name: "style",
            namespace: None,
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    fn shorthand_style(prop: &'static str, value: &str) -> Attribute {
        Attribute {
            name: prop,
            namespace: Some("style"),
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    fn other(name: &'static str, value: &str) -> Attribute {
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    /// Every style attribute in the returned list, as `(name, value)`
    /// pairs -- lets the assertions below check there is exactly one no
    /// matter which merge order `merge_attributes` happens to produce.
    fn style_attrs(attrs: &[Attribute]) -> Vec<(&'static str, String)> {
        attrs
            .iter()
            .filter(|a| a.name == "style")
            .map(|a| match &a.value {
                Text(s) => (a.name, s.clone()),
                _ => (a.name, String::new()),
            })
            .collect()
    }

    #[test]
    fn no_caller_style_yields_just_the_anchor_binding() {
        let result = anchored_content_attributes("x", vec![other("role", "tooltip")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        assert_eq!(styles[0].1, "position-anchor: --dxa-x;");
        // Non-style attributes pass through untouched.
        assert!(result.iter().any(|a| a.name == "role"));
    }

    #[test]
    fn plain_caller_style_is_folded_after_the_anchor_binding() {
        let result = anchored_content_attributes("x", vec![plain_style("min-height: 100px;")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("position-anchor: --dxa-x;"));
        assert!(style.contains("min-height: 100px;"));
        // Anchor binding first, so the caller's own declaration -- listed
        // after -- can override any property it names without ever being
        // able to drop the anchor binding itself.
        assert!(style.find("position-anchor").unwrap() < style.find("min-height").unwrap());
    }

    #[test]
    fn shorthand_caller_style_is_folded_after_the_anchor_binding() {
        let result = anchored_content_attributes("x", vec![shorthand_style("padding", "1rem")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("position-anchor: --dxa-x;"));
        assert!(style.contains("padding:1rem;"));
        assert!(style.find("position-anchor").unwrap() < style.find("padding").unwrap());
    }

    #[test]
    fn shorthand_and_plain_caller_style_both_fold_into_the_one_attribute() {
        // The shape that breaks without folding: a caller mixing a plain
        // `style` literal with shorthand style props (or a themed wrapper
        // contributing one of each), on top of this component's own
        // `position-anchor` literal -- three style-contributing values that
        // must all end up in the one served `style="..."`.
        let result = anchored_content_attributes(
            "x",
            vec![
                plain_style("min-height: 100px;"),
                shorthand_style("padding", "1rem"),
                other("role", "tooltip"),
            ],
        );
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("position-anchor: --dxa-x;"));
        assert!(style.contains("min-height: 100px;"));
        assert!(style.contains("padding:1rem;"));
        assert!(style.find("position-anchor").unwrap() < style.find("min-height").unwrap());
        assert!(result.iter().any(|a| a.name == "role"));
    }

    // -----------------------------------------------------------------
    // `anchored_trigger_attributes` -- the trigger-side mirror of the
    // `anchored_content_attributes` tests above. Same four shapes (no
    // caller style / plain / shorthand / both), asserted the same way,
    // just against `anchor-name` instead of `position-anchor`. The web
    // arm always yields a non-empty `anchor_name_style`, so these only
    // exercise the `(false, ..)` arms of its `match`; the empty-anchor
    // (native) arms are covered by `trigger_attributes_native_tests`
    // below, which is not `feature = "web"`-gated.
    // -----------------------------------------------------------------

    #[test]
    fn trigger_no_caller_style_yields_just_the_anchor_binding() {
        let result = anchored_trigger_attributes("x", vec![other("role", "button")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        assert_eq!(styles[0].1, "anchor-name: --dxa-x;");
        assert!(result.iter().any(|a| a.name == "role"));
    }

    #[test]
    fn trigger_plain_caller_style_is_folded_after_the_anchor_binding() {
        let result = anchored_trigger_attributes("x", vec![plain_style("color: red;")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("anchor-name: --dxa-x;"));
        assert!(style.contains("color: red;"));
        // Anchor binding first, so the caller's own declaration -- listed
        // after -- can override any property it names without ever being
        // able to drop the anchor binding itself (the silent-loss failure
        // mode of an overwriting merge -- see this function's own
        // doc and `style_attribute_is_folded_not_overwritten` in
        // `primitives/src/lib.rs`).
        assert!(style.find("anchor-name").unwrap() < style.find("color").unwrap());
    }

    #[test]
    fn trigger_shorthand_caller_style_is_folded_after_the_anchor_binding() {
        let result = anchored_trigger_attributes("x", vec![shorthand_style("cursor", "pointer")]);
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("anchor-name: --dxa-x;"));
        assert!(style.contains("cursor:pointer;"));
        assert!(style.find("anchor-name").unwrap() < style.find("cursor").unwrap());
    }

    #[test]
    fn trigger_shorthand_and_plain_caller_style_both_fold_into_the_one_attribute() {
        let result = anchored_trigger_attributes(
            "x",
            vec![
                plain_style("color: red;"),
                shorthand_style("cursor", "pointer"),
                other("role", "button"),
            ],
        );
        let styles = style_attrs(&result);
        assert_eq!(
            styles.len(),
            1,
            "expected exactly one style attribute: {result:?}"
        );
        let style = &styles[0].1;
        assert!(style.contains("anchor-name: --dxa-x;"));
        assert!(style.contains("color: red;"));
        assert!(style.contains("cursor:pointer;"));
        assert!(style.find("anchor-name").unwrap() < style.find("color").unwrap());
        assert!(result.iter().any(|a| a.name == "role"));
    }

    /// A caller-supplied `id` (or any other non-style attribute) passes
    /// through untouched, deduped against nothing since this function only
    /// ever contributes `style` -- guards against a future edit accidentally
    /// routing more than style through the `dioxus_attributes::attributes!`
    /// base list this function builds.
    #[test]
    fn trigger_non_style_attributes_pass_through_untouched() {
        let result = anchored_trigger_attributes(
            "x",
            vec![other("id", "caller-id"), other("role", "button")],
        );
        assert!(result
            .iter()
            .any(|a| a.name == "id" && matches!(&a.value, Text(s) if s == "caller-id")));
        assert!(result.iter().any(|a| a.name == "role"));
    }
}

/// `anchored_trigger_attributes` is NOT `feature = "web"`-gated (see its own
/// doc for why: every trigger calls it unconditionally, on both arms), so
/// unlike every other test in this module it is meaningful -- and needed --
/// on the native (non-`web`) arm too, which is what `cargo test --workspace`
/// exercises by default (`primitives/Cargo.toml`'s `default = ["router"]`
/// does not include `web`). This covers the `anchor.is_empty()` arms of its
/// `match` that `feature = "web"` can never reach (`anchor_name_style`
/// always returns a real, non-empty string there) -- the web-arm folding
/// behavior itself is covered by `tests::trigger_*` above.
///
/// `not(feature = "web")`, not just `test`: under `--features web`,
/// `anchor_name_style` stops being empty and these exact assertions would
/// fail -- this module is only meaningful on the arm its own name says.
#[cfg(all(test, not(feature = "web")))]
mod trigger_attributes_native_tests {
    use super::*;
    use dioxus_core::AttributeValue::Text;

    fn plain_style(value: &str) -> Attribute {
        Attribute {
            name: "style",
            namespace: None,
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    fn other(name: &'static str, value: &str) -> Attribute {
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value: Text(value.to_string()),
        }
    }

    #[test]
    fn native_anchor_is_empty_so_no_caller_style_yields_no_style_attribute_at_all() {
        // Off the web arm, `anchor_name_style` is the inert `String::new()`
        // stub -- with no caller style either, there is nothing to say, so
        // this must not manufacture a stray empty `style=""`.
        let result = anchored_trigger_attributes("x", vec![other("role", "button")]);
        assert!(!result.iter().any(|a| a.name == "style"));
        assert!(result.iter().any(|a| a.name == "role"));
    }

    #[test]
    fn native_anchor_is_empty_so_caller_style_passes_through_unchanged() {
        // No anchor binding to fold in on this arm -- the caller's own
        // `style` should come through exactly as given, not prefixed with
        // an empty anchor segment (no stray leading space either).
        let result = anchored_trigger_attributes("x", vec![plain_style("color: red;")]);
        let style = result.iter().find(|a| a.name == "style").expect("style");
        assert!(matches!(&style.value, Text(s) if s == "color: red;"));
    }
}

#[cfg(test)]
mod viewport_tests {
    use super::VIEWPORT_METRICS_JS;

    /// The clamp/flip scripts must measure the viewport ONLY through
    /// [`VIEWPORT_METRICS_JS`]: a direct `window.innerWidth` /
    /// `visualViewport.width` read re-opens the scrollbar-overshoot bug
    /// (overlay lands ~11px past the content area). Source-level guard so a
    /// new collision script cannot quietly bypass the shared helper.
    #[test]
    fn viewport_metrics_js_is_the_only_viewport_read() {
        let source = include_str!("top_layer.rs");
        let rest = source.replacen(VIEWPORT_METRICS_JS, "", 1);
        assert!(
            rest.len() < source.len(),
            "VIEWPORT_METRICS_JS text must appear verbatim in top_layer.rs"
        );
        // Only this test module's own needle strings may mention these.
        let code = rest.split("mod viewport_tests").next().unwrap();
        for (n, line) in code.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            for needle in [
                "window.innerWidth",
                "window.innerHeight",
                "vv.width",
                "vv.height",
                "visualViewport.width",
                "visualViewport.height",
            ] {
                assert!(
                    !line.contains(needle),
                    "top_layer.rs:{} reads `{needle}` outside VIEWPORT_METRICS_JS; use viewportMetrics()",
                    n + 1
                );
            }
        }
    }

    #[test]
    fn viewport_metrics_js_takes_the_smaller_of_visual_and_client() {
        let js = VIEWPORT_METRICS_JS;
        assert!(js.contains("Math.min(vv.width, clientW)"));
        assert!(js.contains("Math.min(vv.height, clientH)"));
        assert!(js.contains("documentElement"));
        assert!(js.contains("clientWidth"));
        assert!(js.contains("clientHeight"));
    }
}
