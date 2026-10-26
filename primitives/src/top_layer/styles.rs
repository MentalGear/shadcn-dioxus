//! Injected anchor-positioning stylesheet and top-layer ink baseline.

#[cfg_attr(
    not(feature = "web"),
    expect(unused_imports, reason = "only the web build uses it")
)]
use super::*;

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
/// ## Why it is injected once, engine-side
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
/// have to select on something that is not a `Styles::`-routed class. A
/// consuming page that lacked its copy would silently fall back to
/// un-anchored positioning, and every anchored overlay would have to
/// remember to duplicate this same block into its own page's stylesheet, or
/// fail the identical way. Injecting this once, engine-side, from the same
/// Rust module that defines the anchor contract
/// (`anchor_name_style`/`position_anchor_style`/`anchored_content_attributes`
/// below) makes anchor positioning "just work" for every anchored overlay
/// everywhere in the app, with no per-page CSS to remember.
///
/// ## One attribute, not a list of classes (backlog row 432)
///
/// Every rule below is scoped by `[data-dx-anchored]`, the one attribute
/// `anchored_content_attributes` writes from a [`AnchorKind`] (the same
/// shape as `data-dx-modality`: one writer, one reader). The stylesheet
/// therefore never names a kind except where kinds really differ (a hover
/// card's wider gap): a new anchored overlay is anchored by calling the
/// function it must already call for its `position-anchor`, and there is no
/// list here to forget it in. The selectors rank exactly as a class plus
/// `[popover]` would (`[data-dx-anchored]:is([popover], :modal)` is
/// `(0,2,0)`; `:where()` carries the hover-card exclusion at zero), so no
/// component sheet's override is outranked by them.
/// `scripts/check-anchor-marker.sh` keeps a marker class from coming back.
///
/// Call this once per overlay content mount -- [`use_anchor_position_fallback`]
/// does, since every current anchored consumer (`tooltip.rs`,
/// `hover_card.rs`, `popover.rs`, `dropdown_menu.rs`, `menubar.rs`,
/// `navbar.rs`, `select/components/list.rs`, `combobox/components/list.rs`,
/// and anything built on `crate::popover` like `ColorPicker`/`DatePicker`)
/// already calls that hook. A future anchored consumer that doesn't call
/// that hook for some reason should call this directly instead.
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
pub(super) fn anchor_positioning_inject_js() -> String {
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
/// why it is injected once and why `border`/`overflow` resets are
/// deliberately not part of it.
///
/// Every `[data-dx-anchored][popover]` selector below also matches `:modal`
/// (`:is([popover], :modal)`): a modal `Popover`'s web arm
/// is a real `<dialog>` opened with `showModal()`, so it carries
/// `data-dx-anchored="popover"` the same way the non-modal arm's
/// `<dialog popover="auto">` does, but never the `popover` attribute itself
/// (`showModal()` and the Popover API are two different top-layer
/// mechanisms -- see `popover.rs`'s module doc). `:modal` is the CSS
/// pseudo-class a `<dialog>` matches for as long as it is shown via
/// `showModal()` -- the natural selector to key the exact same reset/
/// `anchor()` treatment off for that arm, mirroring `[popover]`'s role for
/// the Popover-API arms without needing a second attribute. (The RTL rules are
/// `[popover]` only: a modal dialog never matched them.)
#[cfg(feature = "web")]
const ANCHOR_POSITIONING_CSS_JS_LITERAL: &str = r#"`
:where([data-dx-anchored]:is([popover], :modal)) {
  margin: 0;
  inset: auto;
}

@supports (anchor-name: --a) {
  [data-dx-anchored]:where(:not([data-dx-anchored="hover-card"])):is([popover], :modal) {
    position: fixed;
    margin: 0;
    inset: auto;
    transform: none;
    position-try-fallbacks: flip-block, flip-inline;
    /* Anchor width contract (dev-docs/backlog.md row 47): publish
       the trigger's width as the same --dx-anchor-width custom property
       use_anchor_position_fallback's JS measurement writes on every
       engine (see that function's doc, "Anchor width contract") -- a CSS-
       native belt on a conforming engine, alongside that JS suspenders
       everywhere else. In Chromium, anchor-size() substitutes correctly
       when only referenced from inside a custom-property declaration and then read
       back through var() at a sizing property's use site -- so this is
       safe to declare generically here, on the data-dx-anchored attribute every
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

  [data-dx-anchored]:where(:not([data-dx-anchored="hover-card"])):is([popover], :modal)[data-side="top"] {
    bottom: anchor(top);
    left: anchor(center);
    margin-bottom: 8px;
    transform: translateX(-50%);
  }

  [data-dx-anchored]:where(:not([data-dx-anchored="hover-card"])):is([popover], :modal)[data-side="right"] {
    top: anchor(center);
    left: anchor(right);
    margin-left: 8px;
    transform: translateY(-50%);
  }

  [data-dx-anchored]:where(:not([data-dx-anchored="hover-card"])):is([popover], :modal)[data-side="bottom"] {
    top: anchor(bottom);
    left: anchor(center);
    margin-top: 8px;
    transform: translateX(-50%);
  }

  [data-dx-anchored]:where(:not([data-dx-anchored="hover-card"])):is([popover], :modal)[data-side="left"] {
    top: anchor(center);
    right: anchor(left);
    margin-right: 8px;
    transform: translateY(-50%);
  }

  [data-dx-anchored="hover-card"][popover] {
    position: fixed;
    margin: 0;
    inset: auto;
    position-try-fallbacks: flip-block, flip-inline;
    /* See the anchor width contract comment on the shared data-dx-anchored
       rule above (no backticks in this comment either -- same reason) --
       same reasoning, this selector just lives in its own rule because
       HoverCard uses a wider default gap. */
    --dx-anchor-width: anchor-size(width);
  }

  [data-dx-anchored="hover-card"][popover][data-side="top"] {
    bottom: anchor(top);
    left: anchor(center);
    margin-bottom: 10px;
    transform: translateX(-50%);
  }

  [data-dx-anchored="hover-card"][popover][data-side="right"] {
    top: anchor(center);
    left: anchor(right);
    margin-left: 10px;
    transform: translateY(-50%);
  }

  [data-dx-anchored="hover-card"][popover][data-side="bottom"] {
    top: anchor(bottom);
    left: anchor(center);
    margin-top: 10px;
    transform: translateX(-50%);
  }

  [data-dx-anchored="hover-card"][popover][data-side="left"] {
    top: anchor(center);
    right: anchor(left);
    margin-right: 10px;
    transform: translateY(-50%);
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="top"][data-align="start"],
  [data-dx-anchored]:is([popover], :modal)[data-side="bottom"][data-align="start"] {
    left: anchor(left);
    transform: none;
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="top"][data-align="center"],
  [data-dx-anchored]:is([popover], :modal)[data-side="bottom"][data-align="center"] {
    left: anchor(center);
    transform: translateX(-50%);
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="top"][data-align="end"],
  [data-dx-anchored]:is([popover], :modal)[data-side="bottom"][data-align="end"] {
    left: anchor(right);
    transform: translateX(-100%);
  }

  /* RTL (backlog row 137): align is logical -- start is the INLINE start
     edge, so under dir=rtl a top/bottom panel aligned to start hangs from the
     trigger's right edge and one aligned to end from its left. The panel is a
     DOM descendant of its trigger, so :dir() sees the page direction even in
     the top layer. (left/right sides align on the block axis, which direction does not touch.) */
  [data-dx-anchored][popover][data-side="top"][data-align="start"]:dir(rtl),
  [data-dx-anchored][popover][data-side="bottom"][data-align="start"]:dir(rtl) {
    left: anchor(right);
    transform: translateX(-100%);
  }

  [data-dx-anchored][popover][data-side="top"][data-align="end"]:dir(rtl),
  [data-dx-anchored][popover][data-side="bottom"][data-align="end"]:dir(rtl) {
    left: anchor(left);
    transform: none;
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="left"][data-align="start"],
  [data-dx-anchored]:is([popover], :modal)[data-side="right"][data-align="start"] {
    top: anchor(top);
    transform: none;
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="left"][data-align="center"],
  [data-dx-anchored]:is([popover], :modal)[data-side="right"][data-align="center"] {
    top: anchor(center);
    transform: translateY(-50%);
  }

  [data-dx-anchored]:is([popover], :modal)[data-side="left"][data-align="end"],
  [data-dx-anchored]:is([popover], :modal)[data-side="right"][data-align="end"] {
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
/// ## Why the ink baseline exists
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
/// ink token (`--dx-foreground`), no matter how deeply that token is
/// set on an ancestor. `primitives/src/toast.rs`'s
/// `ensure_toast_base_styles` carries a narrower, single-selector
/// instance of the same rule (`:where(.dx-toast-base-region[popover]) {
/// color: inherit; }`, dev-docs/backlog.md row 44) for the raw toast fallback
/// only; this one covers every `[popover]`/`<dialog>` element in the
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

      /* The same rule, one level deeper. Neutralising the wrapper's own
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
