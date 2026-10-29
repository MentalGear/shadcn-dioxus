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
//! ## Who locks
//! Only the modal surfaces take this lock: `Dialog`, `AlertDialog` (and so `Sheet` and `Drawer`),
//! a `Popover` with `is_modal: true`, and `ContextMenu`. An anchored overlay
//! (`DropdownMenu`, `Menubar`, `Select`, a default `Popover`, and the rest) leaves the page
//! scrollable and stays open while it scrolls: an open panel may not be fully in view, and the
//! user must be able to scroll to it.
//!
//! ## How the lock blocks scrolling: Base UI's `useScrollLock`
//! The method is Base UI's (`@base-ui/utils` `useScrollLock`, three branches). The element that
//! scrolls the viewport is `<html>` when `<html>` establishes its own scroll container (the baseline
//! below makes it one) and `<body>` otherwise; an `overflow` set on the other element would not lock
//! the page. Wheel, touch, keyboard and scrollbar scrolling all stop at once, with no event listener
//! on `window` to decide per event: a scroll container inside the surface (a long menu, a code block,
//! a scrollable dialog body) keeps scrolling by itself and cannot chain into the page. A scripted
//! `window.scrollTo()` still moves the page, as it does under any `overflow: hidden`.
//!
//! 1. **Overlay scrollbars, and iOS** (`window.innerWidth - <html>.clientWidth` is 0, or an iOS
//!    user agent): the scroller gets `overflow: hidden`. The scrollbar takes no width, so there is
//!    nothing to keep and no layout shift.
//! 2. **A classic scrollbar, and an engine whose `scrollbar-gutter: stable` survives hidden
//!    overflow** (a probe sets both on the scroller and compares `<html>.clientWidth` under `scroll` and
//!    `hidden`; Firefox passes): `scrollbar-gutter: stable` (`both-edges` when the page asked for it)
//!    on `<html>` plus `overflow: hidden`; the gutter keeps the scrollbar's width.
//! 3. **A classic scrollbar, and an engine that drops the gutter under hidden overflow** (Chromium,
//!    the default browser of Windows and Linux): hiding the root's overflow would give the page the
//!    scrollbar's width, so the root stays `overflow: scroll` (when the page overflows) with the
//!    gutter, and `<body>` is clipped to the visible viewport (`position: relative`, the size of
//!    `<html>`'s client box less the body margin, `overflow: hidden`) with its scroll offset moved from the
//!    root, so the page has nothing left to scroll and the scrollbar stays in place, inert. While
//!    this branch is held `window.scrollY` reads 0; the offset comes back on release. A window resize
//!    re-computes this branch (and branch 2) one frame later.
//!
//! The script restores the exact previous inline values of both elements, so an app's stylesheet and
//! inline styles survive a lock cycle. `playwright/oracle/tier3-radix/scroll-lock.spec.ts` pins
//! `clientWidth`, the sticky nav and a `position: fixed; right: 0` probe constant through a lock
//! cycle in the headless run (branch 1) and in `playwright/xvfb.local.config.ts`'s classic scrollbar
//! (branch 3 on Chromium). Not ported from Base UI: the wait for an app-held lock to clear (a
//! `MutationObserver`; a page that is already locked stays locked and is restored as it was) and its
//! WebKit pinch-zoom skip.
//!
//! **One script for the whole page, none between locks.** The outermost lock installs the script
//! through [`crate::js_listener::JsListeners`] and the last release drops it, which restores the
//! styles (`window.__dxScrollLocked` mirrors "held" for tests). No listener stays on `window`
//! between locks, and the script adds no wheel, touch or key listener at all (only a `resize` one
//! while branch 2 or 3 is held), so a held lock does not make
//! the compositor wait for the main thread before it scrolls a container inside the surface
//! (backlog rows 149 and 159: a non-passive `wheel`/`touchmove` listener on `window` taxes every
//! scroll on the page; `DOMDebugger.getEventListeners(window)` after an open-and-close shows none left).
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
//! **iOS.** Branch 1, as in Base UI: only the overflow is hidden. Whether that stops the inertial
//! scroll of a fling that began before the lock engaged is a device check (row 479), not a
//! Playwright one.
//!
//! ## Permanent `scrollbar-gutter: stable` baseline
//! [`ensure_scrollbar_gutter_baseline`] installs `scrollbar-gutter: stable` and `overflow-y: auto`
//! on `<html>` **permanently** -- once, idempotently, the first time any scroll-lock-capable
//! primitive mounts on the page -- and never toggles it again. Page chrome that is laid out against
//! the root (`position: fixed` overlays, the sticky top nav) then sees the same width before,
//! during and after every lock, and an overlay-scrollbar platform pays the one fixed gutter that a
//! classic-scrollbar platform pays anyway (`stable` reserves its width unconditionally), instead of
//! gaining and losing it on every lock cycle. The call is made from each primitive's *root*
//! component (`DialogRoot`, `AlertDialogRoot`, `PopoverRoot`, `ContextMenu`), not only
//! from [`use_scroll_lock`]: those roots mount as soon as the primitive appears, while the
//! `*Content`/`ScrollLockGuard` components that call `use_scroll_lock` mount lazily, once the
//! surface first opens, and installing the baseline that late would make the first open on a page
//! double as the moment the reservation appears. The lock does not depend on the baseline: branches 2
//! and 3 set the gutter themselves.
//!
//! The textbook `padding-right` recipe (`react-remove-scroll`: measure the scrollbar, add it as
//! `padding-right` on `<body>`) is not used: a `position: fixed` box (a right-aligned navbar
//! action, this crate's own `Popover`/menu content) is positioned against the initial containing
//! block, which no element's padding reaches, so it still shifts by the full scrollbar width.
//!
//! The baseline is a plain (non-`!important`) `:where(html) { scrollbar-gutter: stable;
//! overflow-y: auto; }` rule in an injected `<style>` tag, not an inline style, and specifically
//! through `:where()`'s zero specificity: any author rule touching `html`/`:root` -- even a bare
//! element selector -- wins outright, so an app that already declares its own `scrollbar-gutter`
//! or `overflow-y` on the root is left alone.
//!
//! ## Scroll-position capture and restore, and `use_early_scroll_capture`
//! This hook captures `window.scrollX`/`scrollY` before locking and
//! restores them on unlock. Opening a native `<dialog>`
//! (`DialogContent`/`AlertDialogContent`'s modal arm, `crate::dialog`)
//! reliably shifts `window.scrollY` by several hundred pixels the instant
//! `showModal()` runs (under a forced classic scrollbar,
//! `playwright/xvfb.local.config.ts`, on the dialog demo page, in both
//! Chromium and Firefox; most likely the browser's default
//! scroll-into-view-on-focus behavior for the dialog's autofocus target), and `overflow: hidden`
//! does not stop a script-driven or focus-driven scroll. Because this hook's own
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
//! [`use_scroll_lock`]'s own lock-acquisition script still captures a
//! scroll position too (`get_or_insert`, never clobbering an earlier,
//! more-trustworthy capture from [`use_early_scroll_capture`]) -- both so
//! every other caller of this hook (`Popover`, `ContextMenu`, none of which
//! have a native `showModal()` of their own to race) has a capture at all, and as the value
//! restored on unlock. Restoring to a value that could itself already reflect the dialog's own
//! post-jump position (possible only in the specific race where
//! `use_early_scroll_capture` hasn't resolved yet and this hook's own
//! fallback capture reads a jumped `scrollY`) is still strictly no worse
//! than not restoring at all. This hook does not prevent the dialog's own initial jump.

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
/// `PopoverRoot`, `ContextMenu`) rather than only from
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

/// The script that locks the page while a lock is held (module docs, "How the lock blocks scrolling").
/// Run through [`JsListeners::install`] by the outermost lock and torn down when the last lock
/// releases: `cleanup` hands the styles back. It first sends `[scrollX, scrollY]` so the position
/// can be restored on release.
///
/// `__dxScrollLockN` counts the scripts alive and `__dxScrollLocked` mirrors "a lock is held". The
/// Rust side keeps one page-wide lock, but a new one can start before the previous script's teardown
/// has run: the first script applies the styles, the last one restores them. `gutterSurvives` is
/// Base UI's `supportsStableScrollbarGutter` probe, `lock` its three branches.
const SCROLL_LOCK_JS: &str = r#"
    dioxus.send([window.scrollX, window.scrollY]);
    if (!window.__dxScrollLockN) {
        const w = window, html = document.documentElement, body = document.body;
        const P = ['overflowX', 'overflowY', 'scrollbarGutter', 'scrollBehavior', 'position', 'height', 'width', 'boxSizing'];
        const snap = (el) => P.map((p) => el.style[p]);
        const put = (el, v) => P.forEach((p, i) => { el.style[p] = v[i]; });
        const ios = /iPad|iPhone|iPod/.test(navigator.userAgent) || (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);
        const gutterSurvives = (el) => {
            if (!(w.CSS && CSS.supports('scrollbar-gutter', 'stable'))) return false;
            const [g, y] = [html.style.scrollbarGutter, el.style.overflowY];
            html.style.scrollbarGutter = 'stable';
            el.style.overflowY = 'scroll';
            const shown = html.clientWidth;
            el.style.overflowY = 'hidden';
            const same = shown === html.clientWidth;
            el.style.overflowY = y;
            html.style.scrollbarGutter = g;
            return same;
        };
        let undo = () => {}, inset = true, frame = 0;
        const lock = () => {
            const hc = getComputedStyle(html), bc = getComputedStyle(body);
            const el = hc.overflowX !== 'visible' || hc.overflowY !== 'visible' ? html : body;
            const [hv, bv, top, left] = [snap(html), snap(body), html.scrollTop, html.scrollLeft];
            undo = () => put(html, hv) || put(body, bv);
            const hide = () => { el.style.overflowX = el.style.overflowY = 'hidden'; };
            if (ios || w.innerWidth - html.clientWidth <= 0) return (inset = false) || hide();
            const gutter = hc.scrollbarGutter.includes('both') ? 'stable both-edges' : 'stable';
            if (gutterSurvives(el)) return (html.style.scrollbarGutter = gutter) && hide();
            const mx = parseFloat(bc.marginLeft) + parseFloat(bc.marginRight), my = parseFloat(bc.marginTop) + parseFloat(bc.marginBottom);
            const [bw, bh] = [html.clientWidth - mx, html.clientHeight - my];
            const canY = html.scrollHeight > html.clientHeight || /scroll/.test(hc.overflowY + bc.overflowY);
            const canX = html.scrollWidth > html.clientWidth || /scroll/.test(hc.overflowX + bc.overflowX);
            Object.assign(html.style, { scrollbarGutter: gutter, scrollBehavior: 'unset', overflowX: canX ? 'scroll' : 'hidden', overflowY: canY ? 'scroll' : 'hidden' });
            Object.assign(body.style, { position: 'relative', boxSizing: 'border-box', overflowX: 'hidden', overflowY: 'hidden',
                height: `${bh}px`, width: `${bw}px` });
            [body.scrollTop, body.scrollLeft] = [top, left];
            undo = () => { put(html, hv); put(body, bv); [html.scrollTop, html.scrollLeft] = [top, left]; };
        };
        lock();
        if (inset) listen(window, 'resize', () => { undo(); cancelAnimationFrame(frame); frame = requestAnimationFrame(lock); });
        window.__dxScrollUnlock = () => { cancelAnimationFrame(frame); undo(); };
    }
    window.__dxScrollLockN = (window.__dxScrollLockN || 0) + 1;
    window.__dxScrollLocked = true;
    cleanup(() => {
        window.__dxScrollLocked = (window.__dxScrollLockN = Math.max(0, window.__dxScrollLockN - 1)) > 0;
        if (!window.__dxScrollLocked && window.__dxScrollUnlock) { window.__dxScrollUnlock(); window.__dxScrollUnlock = null; }
    });
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
    /// The lock script, alive from the 0 -> 1 transition of `count` until a 1 -> 0 one that is
    /// not followed by a new lock within a frame (see [`release`]). Dropping it is what gives the
    /// viewport scroller its overflow back.
    script: Option<JsListeners>,
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
/// run -- see there). Popover and ContextMenu have no equivalent
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

/// Prevents the page from scrolling while `active` is true, by hiding the overflow of the
/// viewport scroller as the module docs describe ("How the lock blocks scrolling"); the
/// scrollbar's width stays reserved, so the page does not shift.
///
/// Every lock on the page -- a nested dialog, a sibling dialog opened by the click
/// that closed another, a menu item's dialog -- shares one counter
/// ([`ScrollLockInner`]): only the count's 0 -> 1 transition installs the lock
/// script, and only a 1 -> 0 transition not followed by a new lock within a
/// frame removes it. That
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
    // below. The lock script is NOT part of it: it exists only while a lock is held.
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
        let needs_script = {
            let mut inner = lock_state.0.borrow_mut();
            inner.count += 1;
            if inner.count == 1 {
                // Cancels a final release still waiting for its frame: the lock was handed
                // over, so the script and the saved position stay as they are.
                inner.epoch += 1;
            }
            inner.script.is_none()
        };
        if needs_script {
            let position_state = lock_state.clone();
            let script =
                JsListeners::install(SCROLL_LOCK_JS).on_message(move |(x, y): (f64, f64)| {
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
            lock_state.0.borrow_mut().script = Some(script);
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
/// the lock script (dropping [`JsListeners`], which restores the viewport scroller's styles) and
/// restores the pre-lock scroll position, one frame later: a surface that closes in the same click that opens another (a menu item
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
        finish_release(&state, epoch).await;
    });
}

/// The deferred half of [`release`]: unlocks the page unless a lock was taken since `epoch`, then
/// puts the page back where it was.
async fn finish_release(state: &ScrollLockState, epoch: u64) {
    let (script, restore) = {
        let mut inner = state.0.borrow_mut();
        if inner.count > 0 || inner.epoch != epoch {
            return;
        }
        (inner.script.take(), inner.original_scroll.take())
    };
    // Its teardown, a message to the page, hands the styles back (and, for a clipped `<body>`, the
    // scroll offset the page had when the lock engaged).
    drop(script);
    if let Some((x, y)) = restore.filter(|(x, y)| x.is_finite() && y.is_finite()) {
        // The restore follows the teardown: a task later, never before it, or the teardown's own
        // offset would win.
        crate::sleep::sleep(std::time::Duration::ZERO).await;
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
/// closes -- `ContextMenuContent` renders its own
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
