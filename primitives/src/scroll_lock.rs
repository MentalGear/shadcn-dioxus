//! Body scroll lock -- keeps the page from scrolling behind an open modal
//! surface.
//!
//! Ported from dignifiedquire/dx-components (MIT OR Apache-2.0),
//! `primitives/src/scroll_lock.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99.
//! Adapted:
//! 1. That file coordinates nested locks with a `window.__dxScrollLockCount`
//!    global read and written from inside `document::eval`. This version
//!    keeps its refcounting and its restore-the-original-state behavior,
//!    but moves the counter into Rust -- one page-wide count shared by every
//!    call site (backlog row 375: two sibling dialogs share one count;
//!    see `ScrollLockInner`) --
//!    so "is this the outermost lock/unlock" is decided synchronously in
//!    Rust *before* any `document::eval` runs, per dev-docs/plan.md Phase 3.2.
//!    That reproduces the guard sarendipitee/dx-components' `overlay.rs`
//!    uses against the unlock-flash (@ ~L625-655: only unlock once the
//!    shared count reaches zero) without adopting that fork's
//!    `OverlayCtx`/`OverlayManager` architecture -- see
//!    dev-docs/reference/platform-rationale.md §5.
//! 2. That file's unmount cleanup guards on `*active.peek()` being true,
//!    i.e. it only releases the lock if `active` is *still* true at drop
//!    time. In dq's own tree that's fine, because their `DialogContent`
//!    always renders (a native `<dialog>`'s `showModal()`/`close()` toggle
//!    visibility, not mount/unmount) so their `active.peek()` genuinely
//!    reflects "was this modal open when the whole `Dialog` disappeared".
//!    In this crate's tree, the components that call this hook unmount
//!    *because* `active` (`is_modal() && open()`) just became false -- so by
//!    drop time `active.peek()` is always false, and porting that guard
//!    verbatim would acquire the lock and never release it. This version
//!    instead tracks, per hook instance, whether *this instance* actually
//!    incremented the counter, and releases based on that -- independent of
//!    `active`'s value at unmount.
//!
//! ## How this hook blocks scrolling: event interception, `<html>` and `<body>` untouched
//! The lock blocks the scroll *input*, not the scroll *capability*. The
//! constraints below rule out every simpler alternative (see
//! `playwright/oracle/tier3-radix/scroll-lock.spec.ts`), and they are
//! non-obvious, so a change to this file must not silently reintroduce one
//! of them.
//!
//! **Why not `overflow: hidden`** (on `<html>` and `<body>`, or on `<html>`'s
//! `overflow-y` alone). It reliably blocks wheel/keyboard scroll
//! (`dev-docs/archive/research/phase4-spike-findings.md` "Round 2"), but
//! removing a classic (non-overlay) scrollbar without reserving its space
//! shifts the whole page horizontally by that width the instant the lock
//! engages. [`ensure_scrollbar_gutter_baseline`] (below) reserves it, but
//! `scrollbar-gutter`'s reservation on `<html>`, on Chromium,
//! requires `overflow-y` to remain **exactly**
//! `auto` (or `scroll`) at every instant -- switching it to `hidden`, even
//! momentarily, drops the reservation for as long as it stays `hidden`.
//! `playwright/oracle/tier3-radix/scroll-lock.spec.ts`'s "scrollbar-gutter
//! reservation survives the lock itself" test pins this.
//!
//! **Why not a compensated `<body>` freeze** (`position: fixed; top:
//! -{scrollY}px; left: 0; right: 0;` on `<body>`, `<html>`'s `overflow`
//! untouched -- Construction A of
//! `dev-docs/archive/research/phase4-spike-findings.md` "Round 2"). It blocks
//! scroll through a real mechanism -- taking `<body>` out of flow via
//! `position: fixed` collapses `<html>`'s own scrollable content to nothing,
//! and `<html>`'s `scrollTop` clamps to `0` immediately as a structural
//! consequence, not an event-interception race -- and never touches
//! `overflow-y`, but it still defeats the gutter. On Chromium,
//! `scrollbar-gutter: stable` on the *root* scroller keeps its reservation
//! only while the root keeps genuinely overflowing -- `overflow-y: auto`
//! alone is necessary but not sufficient -- and a `<body>` freeze (by
//! design) removes all of `<body>`'s contribution to `<html>`'s scrollable
//! content, whether via `position: fixed` or by simply reducing its `height`
//! in normal flow. The identical `scrollbar-gutter: stable` on an ordinary
//! (non-root) scrolling `<div>` keeps its reservation even after its content
//! shrinks below its own height. (A probe that measures a
//! `position: fixed` marker's on-screen edge would miss it, because a
//! `scrollbar-gutter` regression on the *root* also moves that edge; only
//! asserting `document.documentElement.clientWidth` against the *root*
//! scroller in isolation, the way this repo's oracle does, catches it.)
//!
//! **What the lock does instead.** If anything that changes `<html>`'s
//! computed `overflow` *or* reduces its genuine overflow amount defeats
//! `scrollbar-gutter: stable` on this engine, the only lever that cannot
//! trip either failure mode is to never pull it at all -- block the scroll
//! *input* instead of the scroll *capability*. A set of `window`-level,
//! capturing (`{ capture: true, passive: false }`) `wheel`, `keydown` and
//! `touchmove` listeners (plus a passive `touchstart` that remembers where
//! the gesture began) call `preventDefault()` on an event that would
//! otherwise reach the document root -- and nothing else. They exist
//! **only while a lock is held**: the outermost lock installs them through
//! [`crate::js_listener::JsListeners`] and the last release drops that
//! value, which removes every one of them (`window.__dxScrollLocked`
//! mirrors "held" for tests). `clientWidth` stays bit-for-bit
//! constant through a full lock/unlock cycle, because nothing about
//! `<html>`/`<body>`'s style ever changes; wheel and the keyboard's
//! Page/Home/End/Arrow scroll keys are fully blocked while locked and fully
//! restored on unlock.
//!
//! **Why they must not outlive the lock (backlog rows 149 and 159).**
//! Installed once, at the first lock ever taken on the page, and kept for
//! the rest of the visit behind a boolean, a non-passive `wheel`/`touchmove`
//! listener on `window` makes the compositor wait for the main thread
//! before it may scroll, so after the first Dialog, Popover or menu had
//! opened once, *every* wheel tick on the page would pay that wait again
//! (backlog rows 149 and 159; measurements in
//! `dev-docs/archive/research/scroll-jank-2026-10-04.md` section 7.3).
//! `DOMDebugger.getEventListeners(window)` after one open-and-close must
//! show no `wheel`/`touchmove` left.
//!
//! **When the lock releases.** At the real close of the surface, never at the
//! state flip: a modal `<dialog>` keeps playing its exit animation, still in the
//! top layer, after `open` has gone `false`
//! (`crate::use_dialog_open_driver`), and until its `close()` the page behind
//! must still not scroll. [`use_modal_closed`] is the hand-off: a
//! `use_scroll_lock` publishes a release handle to its subtree and the shared
//! `close`-event sync (`crate::use_dialog_close_sync`) calls it when the
//! `<dialog>` has really closed, ~250 ms (`use_animated_open`'s unmount hold)
//! before the component unmounts. The unmount cleanup stays as the fallback
//! (a surface that is not a `<dialog>`, a dialog removed while open), and both
//! paths are idempotent.
//!
//! This follows Radix's own reference implementation:
//! `@radix-ui/react-dialog` et al. delegate to `react-remove-scroll`, which
//! itself blocks scroll via wheel/touch/key *event interception*, not
//! `overflow: hidden`.
//!
//! **Not blocked unconditionally -- deliberately, as this crate's own
//! components require:**
//! - **A scroll/key event whose target has a scrollable ancestor that can
//!   still consume it** (walked from `event.target` up to, but not
//!   including, `<body>`/`<html>`, checking `overflow-y` computes to
//!   `auto`/`scroll`/`overlay`, `scrollHeight > clientHeight`, and the
//!   relevant edge hasn't already been reached) is let through untouched.
//!   The axis follows the gesture's dominant delta (`deltaX` vs `deltaY`
//!   for wheel, the touch's dx vs dy), checking `overflow-x`/`scrollLeft`
//!   for a horizontal one -- a vertical-only check would block every purely
//!   horizontal wheel/touch scroll (a code block or wide table inside a
//!   locked dialog/sheet/drawer).
//!   Wheeling over an unrelated `position: fixed`,
//!   independently `overflow-y: auto` scrollable box (the shape every
//!   `popover`-attributed element in this crate's web arm has by the
//!   WHATWG UA popover stylesheet -- `Tooltip`/`HoverCard`/non-modal
//!   `Popover`/menu content) scrolls *that* box and leaves
//!   `window.scrollY` at `0`, both before and after. Without this check, a
//!   long dropdown menu or popover would lose the ability to scroll its own
//!   overflowing content the instant its own lock engaged.
//! - **A key event whose target is a text-editable control** (`<input>`,
//!   `<textarea>`, `<select>`, or `[contenteditable]`) is never touched --
//!   Home/End/arrow keys have their ordinary cursor-movement meaning there,
//!   which must keep working inside any locked surface's own form fields.
//! - **A key event whose target (or an ancestor, via the scrollable-ancestor
//!   walk stopping short) carries an ARIA `role` attribute is never
//!   touched.** Every
//!   keyboard-navigable widget in this crate that could plausibly sit
//!   inside a locked surface -- `RadioGroup` (`role="radio"`), `Tabs`
//!   (`role="tab"`), `Slider` (`role="slider"`), `Menubar`/`DropdownMenu`/
//!   `ContextMenu` (`role="menuitem"`/`"option"`), `TagGroup`
//!   (`role="gridcell"`), `Toolbar` (`role="toolbar"`) -- implements its own
//!   Page/Home/End/arrow-key handling on an element that carries a `role`.
//!   This hook's listeners never call `stopPropagation()`, so those
//!   widgets' own handlers always still run regardless; the `role` check
//!   only decides whether *this* listener additionally suppresses the
//!   browser's own default action for that key on that element, which for
//!   a `role`-bearing custom widget is never scrolling the page (their own
//!   handler already does whatever the key should do) and skipping it here
//!   avoids ever second-guessing a widget's own keyboard model.
//! - **The Space key is deliberately never included** in the blocked key
//!   set at all (unlike Page/Home/End/arrow keys). Space's browser-default
//!   action is context-dependent -- "scroll the page" only when focus is on
//!   `<body>` or a plain, non-interactive element, but "activate the
//!   focused control" (a real `.click()`) when focus is on a button or
//!   similar. Every surface that calls this hook holds a focus trap while
//!   open, so focus inside a locked surface is essentially never on `<body>`
//!   -- meaning Space's only realistic default action while locked is
//!   activation, which must never be suppressed (a real hazard: see
//!   `dev-docs/archive/research/phase4-spike-findings.md`, Round 1,
//!   experiment 4's note on a dialog's own focused Close button).
//!
//! **What it does not attempt:** no `padding-right`
//! compensation (it does not work, `dev-docs/archive/research/phase4-spike-findings.md`, Round 2, for
//! exactly the `position: fixed` marker shape this crate's own overlays and
//! a realistic right-aligned navbar action both have); no iOS
//! momentum-scroll handling (see "Known limitation" below); touch is
//! handled by the same scroll-chain-aware check on `touchmove` (blocks a
//! drag gesture that would otherwise pan the background), but a fling's
//! post-release inertial scrolling on iOS Safari is a platform behavior no
//! `preventDefault()` placement reaches once the gesture has already ended,
//! same limitation `react-remove-scroll` documents for itself.
//!
//! ## Permanent `scrollbar-gutter: stable` baseline
//! From `dev-docs/archive/research/phase4-spike-findings.md`, "Round 2 -- solved by
//! construction", Construction A/C. The defect it prevents: naively removing scroll
//! capability never compensates for a classic (non-overlay) scrollbar's
//! width on its own, so doing so shifts everything horizontally by that
//! width the instant a lock engages -- 15px under a forced real
//! scrollbar (`playwright/xvfb.local.config.ts`), invisible under this
//! repo's default headless Chromium, which renders 0-width overlay
//! scrollbars regardless of how tall the page is. Most users' actual
//! browsers (Windows/Linux Chrome and Firefox defaults, and other engines)
//! render classic, space-reserving scrollbars, so this is a real
//! shift that is silent in the default test run -- see
//! `playwright/oracle/tier3-radix/scroll-lock.spec.ts`'s
//! `assertNoHorizontalShift`.
//!
//! [`ensure_scrollbar_gutter_baseline`] installs `scrollbar-gutter: stable`
//! on `<html>` **permanently** -- once, idempotently, the first time any
//! scroll-lock-capable primitive mounts on the page -- and never toggles it
//! again, in either direction, for any reason. The lock mechanism above
//! never touches `<html>`/`<body>` at all, so
//! there is no moment in the lock's lifecycle where this
//! baseline's own precondition (`overflow-y` computing to `auto`/`scroll`
//! on `<html>`, *and* `<html>` genuinely still overflowing -- both
//! required, as above) could stop holding. That call is made from
//! each primitive's *root* component (`DialogRoot`, `AlertDialogRoot`,
//! `PopoverRoot`, `DropdownMenu`, `ContextMenu`), not only from
//! [`use_scroll_lock`] here: those roots mount as soon as the primitive
//! appears at all, while the `*Content`/`ScrollLockGuard` components that
//! call `use_scroll_lock` mount lazily, only once the surface first opens --
//! installing the baseline that late would make the very first open on a
//! page double as the moment the gutter reservation appears, i.e. exactly
//! the one-time shift a *permanent* baseline exists to avoid. Two
//! constructions are rejected (full detail and measurements in the findings
//! doc):
//! 1. **The textbook `padding-right` recipe** (`react-remove-scroll` and
//!    similar: measure `innerWidth - documentElement.clientWidth`, add it as
//!    `padding-right` on `<body>`). It does not work for `position: fixed`
//!    elements: a fixed-position box (a realistic right-aligned navbar
//!    action, or this crate's own `Popover`/menu content) is positioned
//!    against the *initial containing block*, whose size tracks the true
//!    viewport net of the real scrollbar and is not influenced by any
//!    element's padding, so it still shifts the full scrollbar width.
//! 2. **A transient `scrollbar-gutter: stable` toggle**, applied together
//!    with `overflow: hidden` only for the duration of the lock. This fixes
//!    the classic-scrollbar case, but Chromium reserves `stable`'s gutter
//!    width unconditionally the instant the property takes effect,
//!    regardless of whether the platform was otherwise showing an overlay
//!    scrollbar -- so on an overlay-scrollbar platform (this repo's default
//!    headless Chromium included), turning it on *at lock time* introduces
//!    exactly the shift being fixed, just newly, on the other kind of
//!    platform, and reverts it again on unlock -- a shift on every open and
//!    every close, oscillating.
//!
//! Applying it *permanently* rather than transiently means overlay-scrollbar
//! platforms pay the same fixed one-time gutter reservation classic-scrollbar
//! platforms always pay anyway (this is `scrollbar-gutter: stable`'s
//! documented, unconditional behavior), instead of oscillating on every
//! lock cycle.
//!
//! The baseline is installed via a plain (non-`!important`) `:where(html) {
//! scrollbar-gutter: stable; overflow-y: auto; }` rule in an injected
//! `<style>` tag, not an inline style, and specifically through
//! `:where()`'s zero specificity: any author rule touching `html`/`:root` --
//! even a bare element selector -- wins outright, so an app that already
//! declares its own `scrollbar-gutter` or `overflow-y` on the root is left
//! alone.
//!
//! **Old-engine note:** `scrollbar-gutter` is unsupported on older
//! WebKit/Safari. There, the injected rule is inert (an unrecognized
//! property is dropped), so those engines get no scrollbar-gap
//! compensation -- never worse than without the baseline. The
//! scroll-blocking mechanism does not depend on `scrollbar-gutter` at all,
//! so scroll blocking keeps working there regardless.
//!
//! ## Known limitation
//! No iOS momentum-scroll handling: this hook can block the touch drag that
//! *initiates* a scroll gesture, but a fling's post-release inertial
//! scrolling on iOS Safari is a platform behavior no event listener reaches
//! once the gesture has ended. Radix delegates this same case to
//! `react-remove-scroll`, which this crate has no equivalent of. See
//! dev-docs/reference/platform-rationale.md §5.
//!
//! ## Scroll-position capture and restore, and `use_early_scroll_capture`
//! This hook captures `window.scrollX`/`scrollY` before locking and
//! restores them on unlock. That is independent of *how* scrolling is
//! blocked, and exists for a separate reason: opening a native `<dialog>`
//! (`DialogContent`/`AlertDialogContent`'s modal arm, `crate::dialog`)
//! reliably shifts `window.scrollY` by several hundred pixels the instant
//! `showModal()` runs (under a forced classic scrollbar,
//! `playwright/xvfb.local.config.ts`, on the dialog demo page, in both
//! Chromium and Firefox; most likely the browser's default
//! scroll-into-view-on-focus behavior for the dialog's autofocus target).
//! Because this hook's own
//! lock-acquisition effect and the dialog's `showModal()`-driving effect
//! (`crate::use_dialog_open_driver`) are two separate effects that, per
//! Dioxus's normal same-commit effect ordering, may run in either order
//! from this hook's point of view, [`use_early_scroll_capture`] is called
//! from the *root* component (`DialogRoot`/`AlertDialogRoot`) -- an
//! already-mounted component reacting to the same `open` flip that
//! `*Content` mounting-and-effects-running is itself a *consequence* of --
//! so its capture is dispatched, and therefore executes, before
//! `showModal()` gets a chance to move anything.
//!
//! [`use_scroll_lock`]'s own lock-acquisition effect still captures a
//! scroll position too (`get_or_insert`, never clobbering an earlier,
//! more-trustworthy capture from [`use_early_scroll_capture`]) -- both so
//! every other caller of this hook (`Popover`, `DropdownMenu`,
//! `ContextMenu`, none of which have a native `showModal()` of their own to
//! race) has a capture at all, and as the value restored on unlock.
//! Restoring to a value that could itself already reflect the dialog's own
//! post-jump position (possible only in the specific race where
//! `use_early_scroll_capture` hasn't resolved yet and this hook's own
//! fallback capture reads a jumped `scrollY`) is still strictly no worse
//! than not restoring at all, and does not affect `Popover`/`DropdownMenu`/
//! `ContextMenu`, none of which have any focus-driven scroll of their own to
//! correct for. This hook does not prevent the dialog's own initial jump.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::document;
use dioxus::prelude::*;

use crate::js_listener::JsListeners;

thread_local! {
    /// Whether [`ensure_scrollbar_gutter_baseline`] has already inserted its
    /// `<style>` once in this WASM instance. `use_scroll_lock`
    /// is called afresh every time a modal/menu opens (this crate's
    /// `*Content` components mount lazily -- see the component doc below),
    /// so without this guard the baseline install would re-run, harmlessly
    /// but wastefully, on every single open for the rest of the session.
    /// A hot-reload that resets Rust statics but leaves the already-inserted
    /// tag in the live DOM inserts an identical second one, which is harmless.
    static SCROLLBAR_GUTTER_BASELINE_INSTALLED: Cell<bool> = const { Cell::new(false) };

    /// The one page-wide lock (backlog row 375). See [`ScrollLockInner`].
    static PAGE_SCROLL_LOCK: ScrollLockState = ScrollLockState::default();
}

/// Installs the permanent `scrollbar-gutter: stable` baseline described in
/// the module docs, at most once per WASM instance. Deliberately not gated
/// on `active` and not part of the lock/unlock toggle below -- see the
/// module docs for why this must never be toggled per lock.
///
/// Called from an unconditional `use_effect` in each scroll-lock-capable
/// primitive's *root* component (`DialogRoot`, `AlertDialogRoot`,
/// `PopoverRoot`, `DropdownMenu`, `ContextMenu`) rather than only from
/// [`use_scroll_lock`] below: those roots mount as soon as the primitive
/// appears on the page at all, while the `*Content`/`ScrollLockGuard`
/// components that call `use_scroll_lock` mount lazily, only once the
/// surface first opens. Installing only at that later point would make the
/// very first open on a given page load double as the moment the baseline
/// (and therefore its gutter reservation) appears -- a real, if one-time,
/// shift, on overlay-scrollbar platforms, that defeats the purpose of a
/// baseline meant to be there before any interaction. [`use_scroll_lock`]
/// also calls this, as a defensive fallback for any caller that reaches it
/// without going through one of those root components; both call sites are
/// idempotent and race-free (WASM is single-threaded).
pub(crate) fn ensure_scrollbar_gutter_baseline() {
    if SCROLLBAR_GUTTER_BASELINE_INSTALLED.with(|installed| installed.replace(true)) {
        return;
    }
    crate::top_layer::insert_head_style(
        "dx-scrollbar-gutter-baseline",
        include_str!("scrollbar_gutter.css"),
    );
}

/// The script that blocks page scroll while a lock is held (module docs, "How this hook blocks scrolling").
/// Run through [`JsListeners::install`] by the outermost lock and torn down, listeners and
/// all, when the last lock releases: it contains no `removeEventListener`, because `listen`
/// derives the removal. It first sends `[scrollX, scrollY]` so the position can be restored
/// on release.
const SCROLL_BLOCK_JS: &str = r#"
    dioxus.send([window.scrollX, window.scrollY]);
    // `__dxScrollLocked` mirrors "a lock is held" (counted defensively; the Rust side keeps
    // one page-wide lock, so at most one copy runs); the listeners exist only while it is true.
    window.__dxScrollLockN = (window.__dxScrollLockN || 0) + 1;
    window.__dxScrollLocked = true;
    cleanup(() => {
        window.__dxScrollLockN = Math.max(0, (window.__dxScrollLockN || 0) - 1);
        window.__dxScrollLocked = window.__dxScrollLockN > 0;
    });

    const dxHasRole = (el) => !!(el && el.getAttribute && el.getAttribute('role'));
    const dxIsFormControl = (el) => {
        if (!el) return false;
        const tag = el.tagName;
        return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
    };
    // Walks from `el` up to (but not including) <body>/<html>
    // looking for a scroll container that can still consume `delta`
    // along one axis (`horizontal` picks overflow-x/scrollLeft, else
    // overflow-y/scrollTop) -- see the module docs' "Not blocked
    // unconditionally" section for why this exists. The axis matters:
    // a purely horizontal wheel/touch gesture has no vertical delta,
    // so a vertical-only check never finds a horizontally scrollable
    // region (a code block, a wide table) and blocks it.
    const dxFindScrollableAncestor = (el, delta, horizontal) => {
        while (el && el !== document.body && el !== document.documentElement) {
            if (el.nodeType === 1) {
                const style = getComputedStyle(el);
                if (horizontal) {
                    if (/(auto|scroll|overlay)/.test(style.overflowX) && el.scrollWidth > el.clientWidth) {
                        // scrollLeft is 0 at the start edge and runs
                        // negative in RTL, so normalise to "distance
                        // scrolled from the start edge".
                        const max = el.scrollWidth - el.clientWidth;
                        const rtl = style.direction === 'rtl';
                        const fromLeft = rtl ? el.scrollLeft + max : el.scrollLeft;
                        if (delta < 0 && fromLeft > 0) return el;
                        if (delta > 0 && fromLeft < max - 1) return el;
                    }
                } else if (/(auto|scroll|overlay)/.test(style.overflowY) && el.scrollHeight > el.clientHeight) {
                    if (delta < 0 && el.scrollTop > 0) return el;
                    if (delta > 0 && el.scrollTop + el.clientHeight < el.scrollHeight - 1) return el;
                }
            }
            el = el.parentElement;
        }
        return null;
    };

    // blocking-ok: this lock exists to cancel the wheel; the listener is installed only while a lock is held and removed with it
    listen(window, 'wheel', (e) => {
        const horizontal = Math.abs(e.deltaX) > Math.abs(e.deltaY);
        if (dxFindScrollableAncestor(e.target, horizontal ? e.deltaX : e.deltaY, horizontal)) return;
        e.preventDefault();
    }, { passive: false, capture: true });

    // Space is deliberately excluded -- see the module docs'
    // "Not blocked unconditionally" section.
    const DX_SCROLL_KEYS = { PageUp: -1, PageDown: 1, Home: -1, End: 1, ArrowUp: -1, ArrowDown: 1 };
    listen(window, 'keydown', (e) => {
        if (!(e.key in DX_SCROLL_KEYS)) return;
        if (dxIsFormControl(e.target) || dxHasRole(e.target)) return;
        if (dxFindScrollableAncestor(e.target, DX_SCROLL_KEYS[e.key], false)) return;
        e.preventDefault();
    }, { passive: false, capture: true });

    let dxTouchStartX = null;
    let dxTouchStartY = null;
    listen(window, 'touchstart', (e) => {
        const one = e.touches && e.touches.length === 1;
        dxTouchStartX = one ? e.touches[0].clientX : null;
        dxTouchStartY = one ? e.touches[0].clientY : null;
    }, { passive: true, capture: true });
    // blocking-ok: same as the wheel listener above -- held only while a lock is held
    listen(window, 'touchmove', (e) => {
        if (dxTouchStartY === null) return;
        if (!e.touches || e.touches.length !== 1) return;
        const dx = dxTouchStartX - e.touches[0].clientX;
        const dy = dxTouchStartY - e.touches[0].clientY;
        const horizontal = Math.abs(dx) > Math.abs(dy);
        if (dxFindScrollableAncestor(e.target, horizontal ? dx : dy, horizontal)) return;
        e.preventDefault();
    }, { passive: false, capture: true });
"#;

/// The page's lock count, and the page scroll position captured just before the
/// first lock engaged.
///
/// ONE per page, not one per nesting chain (backlog row 375). The page has one scroll
/// position, so every surface that locks it shares one count and one saved position. A
/// per-chain count would give two sibling surfaces two counters: when one click closes
/// dialog A and opens dialog B, A's release (after its exit animation) would be the last
/// of *its* chain and run `window.scrollTo(A's saved position)` with B open, moving the
/// page under B.
#[derive(Default)]
struct ScrollLockInner {
    count: usize,
    original_scroll: Option<(f64, f64)>,
    /// The block listeners, alive from the 0 -> 1 transition of `count` until a 1 -> 0 one
    /// that is not followed by a new lock within a frame (see [`release`]). Dropping it is
    /// what removes them from `window`.
    listeners: Option<JsListeners>,
    /// Bumped by every 1 -> 0 and 0 -> 1 transition, so a deferred final release can tell
    /// that a lock was taken (and possibly released again) while it waited.
    epoch: u64,
}

#[derive(Clone, Default)]
struct ScrollLockState(Rc<RefCell<ScrollLockInner>>);

/// The page-wide [`ScrollLockState`].
fn use_scroll_lock_state() -> ScrollLockState {
    use_hook(|| PAGE_SCROLL_LOCK.with(Clone::clone))
}

/// Captures the page's scroll position *before* a modal surface has any
/// chance to move it, for [`use_scroll_lock`]'s restore-on-unlock mitigation
/// (see the module docs, "Scroll-position capture and restore") to have a
/// pre-jump baseline to restore in the one case that most needs it: a native
/// `<dialog>`'s `showModal()` call, which reliably scrolls the page on its
/// own before `use_scroll_lock`'s *own* capture --
/// which runs from `DialogContent`, mounted and effects-run only as a
/// consequence of the same `open` flip -- would otherwise get a chance to.
///
/// Call this from the *root* component (`DialogRoot`, `AlertDialogRoot`),
/// passing the same `open` the content components key off of: roots are
/// already-mounted, existing components reacting to `open` changing, while
/// `*Content` is a brand-new mount conditioned on that same change --
/// Dioxus's normal top-down effect ordering within one commit means this
/// root-level capture's `document::eval` (which reads `window.scrollX`/
/// `scrollY` the instant it runs, independent of when its result is later
/// received) is dispatched, and therefore executes, before the content's own
/// `showModal()`-driving effect gets a chance to move anything.
///
/// A no-op if a lock is already engaged anywhere on the page (the first lock's
/// captured position is the one worth keeping) or if this cycle's position
/// was already captured (guards a redundant capture racing with
/// [`use_scroll_lock`]'s own, which only captures when this hasn't already
/// run -- see there). Popover/DropdownMenu/ContextMenu have no equivalent
/// call: none of them have a native `showModal()` of their own to race, so
/// [`use_scroll_lock`]'s own capture is early enough for them as-is.
pub(crate) fn use_early_scroll_capture(open: Memo<bool>) {
    let state = use_scroll_lock_state();
    use_effect(move || {
        if !open.cloned() {
            return;
        }
        {
            let inner = state.0.borrow();
            if inner.count > 0 || inner.original_scroll.is_some() {
                return;
            }
        }
        let state = state.clone();
        spawn(async move {
            let mut eval = document::eval("dioxus.send([window.scrollX, window.scrollY]);");
            if let Ok(position) = eval.recv::<(f64, f64)>().await {
                let mut inner = state.0.borrow_mut();
                // Belt-and-suspenders re-check: `use_scroll_lock`'s own
                // capture (or a concurrent instance of this same effect for
                // a sibling root) may have raced ahead while this awaited.
                if inner.original_scroll.is_none() {
                    inner.original_scroll = Some(position);
                }
            }
        });
    });
}

/// Prevents the page from scrolling while `active` is true, via the
/// event-interception mechanism described in the module docs ("How this hook
/// blocks scrolling") -- `<html>`/`<body>` are never touched in any way, so the permanent
/// `scrollbar-gutter: stable` baseline ([`ensure_scrollbar_gutter_baseline`])
/// keeps reserving the same width before, during, and after every lock
/// cycle, on an engine where that reservation depends on `<html>` never
/// having its `overflow` changed *or* its genuine overflow amount reduced.
///
/// Every lock on the page -- a nested dialog, a sibling dialog opened by the click
/// that closed another, a menu item's dialog -- shares one counter
/// ([`ScrollLockInner`]): only the count's 0 -> 1 transition installs the block
/// listeners, and only a 1 -> 0 transition not followed by a new lock within a
/// frame removes them. That
/// means a still-open outer modal is never affected by an inner one
/// closing, and there is no window in which the page is briefly, visibly
/// unlocked between an inner modal's close and an outer modal's
/// already-locked state.
///
/// Call this from a component that mounts exactly while the lock should be
/// held and unmounts when it shouldn't (this crate's `*Content` components,
/// which are conditionally rendered by `use_animated_open`) -- the lock is
/// acquired in this hook's effect. It is released at the first of:
/// * the surface's real close, when a modal `<dialog>` below this component
///   reports it ([`use_modal_closed`]) and `active` is already false -- after
///   the exit animation, never at the `open` flip;
/// * this component's unmount.
pub(crate) fn use_scroll_lock(active: Memo<bool>) {
    let state = use_scroll_lock_state();

    // Permanent baseline (see module docs) -- unconditional, run once per
    // WASM instance, and never re-run or reverted by the lock/unlock toggle
    // below. The scroll-blocking listeners are NOT part of it: they exist only
    // while a lock is held.
    use_effect(ensure_scrollbar_gutter_baseline);

    // Whether *this* hook instance is the one that incremented the shared
    // counter -- tracked separately from `active` because by the time this
    // component unmounts, `active` has typically already flipped to false
    // (which is what releases the lock), so it can't be used to decide whether
    // the matching decrement is still owed. A plain `Rc<Cell>`, not a signal: it
    // is read from the close-event task and from the unmount cleanup, never reactively,
    // and must stay readable after this component's own signals are gone.
    let acquired = use_hook(|| Rc::new(Cell::new(false)));

    // Handed to the `<dialog>` driver below this component, which calls it when the
    // dialog has really closed. A no-op once this component is gone.
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    use_hook({
        let state = state.clone();
        let acquired = acquired.clone();
        let alive = alive.clone();
        move || {
            provide_context(ModalClosedRelease(Rc::new(move || {
                // Still wanted (a controlled surface that was not closed, or one that
                // reopened while the close was in flight): keep the lock. `try_read`, not
                // `try_peek`: a memo's peek returns its last computed value without
                // recomputing a dirty one, so a reopen whose `open` write has not re-rendered
                // yet (the quiet close of row 223) would read as "closed" and drop the lock.
                if alive.get() && !active.try_read().map(|v| *v).unwrap_or(true) {
                    release(&state, &acquired);
                }
            })))
        }
    });

    let lock_state = state.clone();
    let lock_acquired = acquired.clone();
    use_effect(move || {
        if !active() || lock_acquired.get() {
            return;
        }
        lock_acquired.set(true);
        let needs_listeners = {
            let mut inner = lock_state.0.borrow_mut();
            inner.count += 1;
            if inner.count == 1 {
                // Cancels a final release still waiting for its frame: the lock was handed
                // over, so the listeners and the saved position stay as they are.
                inner.epoch += 1;
            }
            inner.listeners.is_none()
        };
        if needs_listeners {
            let position_state = lock_state.clone();
            let listeners =
                JsListeners::install(SCROLL_BLOCK_JS).on_message(move |(x, y): (f64, f64)| {
                    // Don't clobber an earlier, pre-jump capture --
                    // `use_early_scroll_capture` (called from `DialogRoot`/
                    // `AlertDialogRoot`) may already have run by now with a
                    // more trustworthy value; see its doc.
                    position_state
                        .0
                        .borrow_mut()
                        .original_scroll
                        .get_or_insert((x, y));
                });
            lock_state.0.borrow_mut().listeners = Some(listeners);
        }
    });

    // Cleanup only runs when this hook's owning component unmounts. `acquired` (not
    // `active`) records whether this instance still owes a decrement -- see adaptation
    // note 2 above. Idempotent with the early release above.
    crate::use_effect_cleanup(move || {
        alive.set(false);
        release(&state, &acquired);
    });
}

/// What a `use_scroll_lock` publishes to its subtree: "the surface under me really closed,
/// release the lock if nothing wants it any more". See [`use_modal_closed`].
#[derive(Clone)]
struct ModalClosedRelease(Rc<dyn Fn()>);

/// A handle for the `<dialog>` driver to say that its dialog has really closed (the
/// `close` event, fired after the exit animation by `crate::use_dialog_open_driver`'s
/// `close()`). A no-op when no [`use_scroll_lock`] sits above the caller.
#[derive(Clone)]
#[cfg_attr(
    not(feature = "web"),
    expect(dead_code, reason = "only the web build uses it")
)]
pub(crate) struct ModalClosed(Option<Rc<dyn Fn()>>);

#[cfg_attr(
    not(feature = "web"),
    expect(dead_code, reason = "only the web build uses it")
)]
impl ModalClosed {
    /// Release the lock held for this surface, if it is no longer wanted.
    pub(crate) fn notify(&self) {
        if let Some(release) = &self.0 {
            release();
        }
    }
}

/// Finds the [`use_scroll_lock`] above (or in) the calling component. Call it after
/// `use_scroll_lock` in the same component, or from any descendant.
#[cfg_attr(
    not(feature = "web"),
    expect(dead_code, reason = "only the web build uses it")
)]
pub(crate) fn use_modal_closed() -> ModalClosed {
    use_hook(|| ModalClosed(try_consume_context::<ModalClosedRelease>().map(|r| r.0)))
}

/// How long the final release waits for a lock handed over in the same click: one 60 Hz frame.
const RELEASE_DELAY: std::time::Duration = std::time::Duration::from_millis(16);

/// Gives back this instance's share of the lock, if it holds one. The last release removes
/// the block listeners (dropping [`JsListeners`]) and restores the pre-lock scroll position,
/// one frame later: a surface that closes in the same click that opens another (a menu item
/// that opens a dialog) may release before the new one has locked, and the hand-over must not
/// unlock or scroll the page in between. A lock taken in that frame cancels the final release.
fn release(state: &ScrollLockState, acquired: &Cell<bool>) {
    if !acquired.replace(false) {
        return;
    }
    let epoch = {
        let mut inner = state.0.borrow_mut();
        inner.count = inner.count.saturating_sub(1);
        if inner.count > 0 {
            return;
        }
        inner.epoch += 1;
        inner.epoch
    };
    let state = state.clone();
    // The root scope's task: this runs from an unmount cleanup too, whose own scope is going.
    // interval-ok: one bounded wait for a frame, not a timer
    crate::dioxus_core::spawn_forever(async move {
        // One frame, as a task: there is no JS timer to clear, and a hidden tab, where no frame
        // ever comes, still releases.
        crate::sleep::sleep(RELEASE_DELAY).await;
        finish_release(&state, epoch);
    });
}

/// The deferred half of [`release`]: unlocks the page unless a lock was taken since `epoch`.
fn finish_release(state: &ScrollLockState, epoch: u64) {
    let (listeners, restore) = {
        let mut inner = state.0.borrow_mut();
        if inner.count > 0 || inner.epoch != epoch {
            return;
        }
        (inner.listeners.take(), inner.original_scroll.take())
    };
    // Removes `wheel`/`keydown`/`touchstart`/`touchmove` from `window`.
    drop(listeners);
    if let Some((x, y)) = restore.filter(|(x, y)| x.is_finite() && y.is_finite()) {
        // Restores whatever page position was in effect just before the lock engaged --
        // see the module docs ("Scroll-position capture and restore") for why this
        // exists: a page-scroll jump (seen on both Chromium and Firefox, most
        // visibly a native `<dialog>`'s own `showModal()` autofocus/
        // scroll-into-view behavior) otherwise persists silently through the lock and
        // is never undone once it releases, leaving the page permanently scrolled away
        // from where the user left it -- e.g. its sticky top nav rendering off-screen.
        // This is a mitigation for that symptom, not a fix for whatever causes the
        // scroll to move in the first place while locked.
        let _ = document::eval(&format!("window.scrollTo({x}, {y});"));
    }
}

/// Mount-scoped host for [`use_scroll_lock`], for a `*Content` component
/// that (unlike `DialogContent`/`AlertDialogContent`/
/// `PopoverContentRendered`) does not itself unmount when its surface
/// closes -- `DropdownMenuContent`/`ContextMenuContent` render their own
/// `if render() { div { ... } }` internally rather than being conditionally
/// rendered by a parent, so the *component function* stays mounted for the
/// whole lifetime of the menu, open or closed, and `use_scroll_lock` called
/// directly there would never see its unmount cleanup run. Render this
/// inside that same `if render() { ... }` block instead (as a zero-output
/// child) so *it* mounts and unmounts on the right cycle.
#[component]
pub(crate) fn ScrollLockGuard(active: Memo<bool>) -> Element {
    use_scroll_lock(active);
    rsx! {}
}
