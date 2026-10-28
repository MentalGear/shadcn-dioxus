use super::*;

/// How long, in milliseconds, user scroll input (a wheel event, or a finger
/// on the screen) must have been quiet before the browser counts as truly
/// idle -- the one signal every scroll-end reaction in this module waits on
/// ([`CAROUSEL_SCROLL_TRACKING_JS`]'s "browser-idle gate").
///
/// **The class this closes.** Logic that reacts to "the scroll
/// has ended" can run while the browser is still scrolling and fight it:
/// a seamless loop re-centring while momentum is still arriving (a visible
/// double move), or a settle running on the first `scrollend` while a
/// trackpad is still delivering input (the virtualised demos' custom
/// scroll-end snap would interfere with the gesture;
/// `playwright/carousel-virtual-wheel.spec.ts` guards it), so the window
/// re-renders and the re-align write runs with snapping suspended, all
/// mid-gesture.
/// `scrollend` alone is not "idle": a trackpad can end the pan's scroll and
/// keep sending its momentum, and a scroll clamped at an edge (the virtual
/// window's own end) ends while the fingers keep pushing. So instead of
/// patching each reaction, every one of them is gated on the same signal:
/// the latest scroll has ended AND no input for this long. What that
/// subsumes: the tracking settle itself (`selected`, the visible range and
/// `inert`), everything downstream of it ([`CarouselVirtualContent`]'s
/// re-anchor, window re-render and instant re-align, and the plain API's
/// settle-driven paging effect), and every [`CAROUSEL_SCROLL_TO_JS`] write
/// (buttons, keys, autoplay, indicators, `scroll_to`) requested while input
/// is still arriving, including its `scroll-snap-type` suspension. Not
/// subsumed, by design: [`CAROUSEL_DRAG_JS`] (a mouse/pen drag has no native
/// scroll behind it and owns the position -- THE RULE's own exception), and
/// the wheel band (`CAROUSEL_WHEEL_BAND_JS`, which never writes the scroll
/// position at all).
///
/// **Why 150.** Within one gesture, recorded trackpad events
/// arrive every 17-20ms (p99 20ms, `playwright/fixtures/carousel-wheel-recordings.json`),
/// and the wheel band already treats 100ms of silence as a
/// gesture boundary (`GAP_MS`). 150ms is comfortably past both, and short
/// enough that a settle after a real lift is not visibly late.
pub(super) const CAROUSEL_INPUT_QUIET_MS: f64 = 150.0;

/// Movement, in CSS pixels, a mouse/pen pointer must travel from its
/// `pointerdown` origin before [`CAROUSEL_DRAG_JS`] treats the gesture as
/// a drag rather than a click -- see [`CarouselContent`]'s own "Pointer
/// drag" doc for what this threshold is for.
pub(super) const CAROUSEL_DRAG_THRESHOLD_PX: f64 = 5.0;

/// How long, in milliseconds, a suspended `scroll-snap-type` waits for a
/// `scrollend` event before falling back to restoring it unconditionally
/// -- only reached on an engine without `scrollend` support (this
/// module's own doc: Safari before v26.2). Shared by both places that
/// suspend `scroll-snap-type` for a discrete `scrollBy`:
/// [`CAROUSEL_DRAG_JS`]'s own release-time settle, and
/// [`CAROUSEL_SCROLL_TO_JS`]'s every-other-paging-path call (see that
/// constant's own doc for why it needs the identical suspend/restore
/// construction). Comfortably longer than a `scroller.scrollBy({behavior:
/// 'smooth'})` transition normally takes to finish, so the fallback
/// essentially never fires *before* that transition has visibly
/// completed.
pub(super) const CAROUSEL_SNAP_RESTORE_FALLBACK_MS: f64 = 500.0;

/// Long-lived (mount-to-unmount): a mouse/pen drag-to-scroll gesture on
/// [`CarouselContent`]'s own element, layered on the *same* scroll-snap
/// track [`CAROUSEL_SCROLL_TO_JS`]/[`CAROUSEL_SCROLL_TRACKING_JS`]
/// already use -- not a second, competing engine. See [`CarouselContent`]'s
/// own "Pointer drag" doc for the construction and why each piece is
/// shaped the way it is; the short version: every `scrollBy` this issues
/// fires the exact same native `scroll` events [`CAROUSEL_SCROLL_TRACKING_JS`]
/// (already attached to the same element) already listens for, so a drag
/// settles on `CarouselContext`'s `selected` through that one existing
/// bridge, never a second source of truth for the index.
///
/// Mirrors [`CAROUSEL_SCROLL_TRACKING_JS`]'s own long-lived shape: a
/// `JsListeners` script that reads its static config with one
/// `await dioxus.recv()` and registers real listeners through `listen`
/// (removed by the derived teardown), driving
/// no Rust-side state at all while active (see the module doc's "Engine"
/// section for why this is a legacy `document::eval` script (ported by
/// row 434) -- `setPointerCapture` and a real, ordered
/// capture-phase `click` listener are what Dioxus's synthetic event props
/// don't reach, the same reasoning `drawer.rs`'s own drag gesture
/// documents for its own escape to `crate::pointer`).
///
/// One non-obvious piece, visible only when driving a real drag:
/// `scroll-snap-type: ... mandatory` re-snaps after *every* scroll
/// operation it considers complete, and a plain `scrollBy` -- with
/// nothing telling the browser that many small ones in a row are one
/// continuous gesture, the way a real touch/wheel drag is -- counts as
/// one. Left enabled during the drag, it fights every single
/// `scrollBy` call and sends a short, slow real-mouse drag flying several
/// slides past where the pointer actually stops. So the drag
/// temporarily sets `el.style.scrollSnapType = 'none'` the moment it
/// starts (after the movement threshold, alongside `data-dragging`).
/// It is not restored right on release: setting the property back on an
/// already-stationary position is a static re-evaluation, not an
/// animated scroll, so the browser's own re-snap would not glide to the
/// nearest slide the way it does after a real touch/trackpad drag (see
/// [`CarouselContent`]'s own "Release settle" doc). `endDrag` below
/// settles explicitly instead, and defers restoring this property until
/// that explicit settle has actually finished.
pub(super) const CAROUSEL_DRAG_JS: &str = concat!(carousel_true_end_js!(), carousel_trace_js!(), "\
    const [id, orientation, thresholdPx, enabled, snapRestoreFallbackMs, align] = await dioxus.recv();
    const el = document.getElementById(id);
    if (!el || !enabled) return;
    // See `CAROUSEL_SCROLL_TO_JS`'s own identical helper's doc -- release
    // settle below finds the 'nearest' slide (and the delta to it) by the
    // same align-aware anchor point, so a drag lands consistently with
    // wherever a button/keyboard/native scroll would have.
    // Logical start/end: under `dir=rtl` the leading edge is the physical
    // right (what `scroll-snap-align: start` means there), so a free
    // (unsnapped) position finds the same 'nearest' slide the snap would.
    const rtlAxis = orientation === 'horizontal' && getComputedStyle(el).direction === 'rtl';
    const anchorOf = (start, end) => {
        const lead = rtlAxis ? end : start;
        const trail = rtlAxis ? start : end;
        if (align === 'center') return (start + end) / 2;
        if (align === 'end') return trail;
        return lead;
    };
    const thresholdSq = thresholdPx * thresholdPx;
    let pointerId = null;
    let originX = 0;
    let originY = 0;
    let lastX = 0;
    let lastY = 0;
    let dragging = false;
    let suppressNextClick = false;
    // `drag_free` release: recent pointer samples ({t, p} along the axis)
    // feed the fling velocity, and `coastToken` supersedes a coast still
    // running when a new gesture starts (same idea as `bounceToken`).
    let samples = [];
    let coastToken = 0;
    const axisPos = (e) => (orientation === 'horizontal' ? e.clientX : e.clientY);
    // Cancels whichever of `endDrag`'s two release-settle mechanisms
    // (the `scrollend` listener or its timeout fallback) is currently
    // pending, or null if neither is. See `onPointerDown`'s own comment.
    let cancelPendingSnapRestore = null;

    // -- Edge rubber-band (mode B3, port of the closed design in
    // dev-docs/archive/research/carousel-overscroll-2026-09-23.md; the eight
    // numbered invariants below are that doc's own §6) --
    //
    // A mouse/pen drag has no native scrolling behind it, so THE RULE
    // (research doc §3) permits this path to keep owning the scroll
    // position (scrollBy-driven) -- unlike the
    // wheel bridge (`CAROUSEL_WHEEL_BAND_JS`, mode D, attached separately to
    // this same element), which never may, because a wheel/trackpad
    // gesture has a compositor and a snap engine also trying to write it.
    let rawOver = 0; // signed content displacement the scroller refused, physical px
    let bounceToken = 0; // see `bounceHome`'s own doc -- invariant 4
    // Per gesture: may the band engage at the physical 'min' (left/top) /
    // 'max' (right/bottom) edge? `null` = not decided yet. Decided ONCE, the
    // first time this drag overdrags past that edge (see `feedOverdrag`), by
    // the shared `dxIsTrueEnd` (`carousel_true_end_js!`) -- the pointer
    // counterpart of the wheel band's once-per-gesture ownership decision.
    // Why there and not at `pointerdown`: the wheel band decides at gesture
    // start from where the scroller RESTS because it has to tell its own push
    // apart from the platform's momentum; a drag has no native scroll behind
    // it, so the only question is whether the edge it actually runs into is
    // a true end, and a drag can start mid-track and reach either edge. The
    // rendered slice cannot change mid-drag (the tracking settle is frozen
    // while `data-dragging` is set), so the cached answer stays correct.
    let edgeAllowed = { min: null, max: null };
    const RUBBER_C = 0.55; // WebKit's own published rubber-band constant (research doc §4)

    function axisSize() {
        // The clipping viewport wrapper's own size (`el`'s direct parent,
        // `CarouselContent`'s own \"data-slot=carousel-viewport\" div --
        // see that component's own doc), NOT `el.clientWidth`/`clientHeight`
        // directly: the gap model's own negative `margin-inline-start`/
        // `-block-start` compensation (`content_gap_margin`'s own doc)
        // deliberately widens the scroller's own layout box by the gap's
        // width, clipped by this wrapper -- so `el`'s own clientWidth is
        // no longer the track's true VISUAL size once a non-zero
        // `--dx-carousel-gap` is in effect, and using it here would subtly
        // recalibrate the rubber-band feel by the gap's width for no
        // reason connected to this feature at all (a 16px gap
        // widens a 320px track's `clientWidth` to 336px, a ~5% rubber
        // limit drift).
        const track = el.parentElement;
        const size = orientation === 'horizontal'
            ? (track ? track.clientWidth : el.clientWidth)
            : (track ? track.clientHeight : el.clientHeight);
        return size || 320;
    }
    function rubberLimit() {
        // Asymptote at trackWidth/0.55 (research doc §4/§5) -- no fixed
        // room to run out of, so the depth can never reach a wall the way
        // a padded-margin approach's would.
        return axisSize() / RUBBER_C;
    }
    function rubber(depth) {
        const L = rubberLimit();
        return (L * depth) / (depth + L);
    }
    function rubberSigned(x) {
        return x < 0 ? -rubber(-x) : rubber(x);
    }
    // Invariant 1: never `scrollLeft`/`scrollTop` -- a `getBoundingClientRect`
    // diff is a physical rectangle regardless of `dir`, so this needs no
    // RTL branch at all.
    function contentOffset() {
        const first = el.children[0];
        if (!first) {
            return 0;
        }
        const r = first.getBoundingClientRect();
        const c = el.getBoundingClientRect();
        return orientation === 'horizontal' ? r.left - c.left : r.top - c.top;
    }
    function applyBounce() {
        if (!rawOver) {
            el.style.transform = '';
            return;
        }
        const depth = rubberSigned(rawOver).toFixed(2);
        el.style.transform =
            orientation === 'horizontal' ? `translateX(${depth}px)` : `translateY(${depth}px)`;
    }
    // Ask the scroller for the whole of `cd` first (a real `scrollBy`,
    // the same call a plain drag makes); whatever it
    // REFUSES -- because it is already at an edge -- is the overdrag. That
    // refusal is how the edge is located: no margin, no scroll-coordinate
    // reasoning, no direction branch. `prefers-reduced-motion` drops the
    // whole effect rather than merely skipping the spring-back animation:
    // this is cosmetic feedback, not functional scrolling, so 'no bounce'
    // is the correct reading of that preference here.
    function feedOverdrag(cd) {
        if (!cd) {
            return;
        }
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            if (rawOver) {
                rawOver = 0;
                applyBounce();
            }
            dxTrace(el, 'scroll-write', { by: 'drag-move', delta: -cd, behavior: 'instant', snapBefore: el.style.scrollSnapType, reduced: true });
            if (orientation === 'horizontal') {
                el.scrollBy({ left: -cd, behavior: 'instant' });
            } else {
                el.scrollBy({ top: -cd, behavior: 'instant' });
            }
            return;
        }
        const before = contentOffset();
        if (orientation === 'horizontal') {
            el.scrollBy({ left: -cd, behavior: 'instant' });
        } else {
            el.scrollBy({ top: -cd, behavior: 'instant' });
        }
        const moved = contentOffset() - before;
        let refused = cd - moved;
        if (dxTraceOn) {
            dxTrace(el, 'scroll-write', { by: 'drag-move', delta: -cd, moved: Math.round(moved * 10) / 10, refused: Math.round(refused * 10) / 10, behavior: 'instant', snapBefore: el.style.scrollSnapType });
        }
        if (refused) {
            // Content pushed toward the right/bottom opens a gap at the min
            // edge, and vice versa.
            const side = refused > 0 ? 'min' : 'max';
            if (edgeAllowed[side] === null) {
                edgeAllowed[side] = dxIsTrueEnd(el, orientation === 'horizontal', side);
            }
            if (!edgeAllowed[side]) {
                // Not a true end: behave like a drag that simply cannot pass
                // the edge -- the scroller has already clamped, and the
                // refused motion is dropped rather than drawn as a band.
                refused = 0;
            }
        }
        if (!refused && !rawOver) {
            return;
        }
        const prev = rawOver;
        const next = prev + refused;
        // Collapse through zero rather than let a sign flip leave a
        // residual fractional depth once the scroller has fully absorbed
        // the request -- a corner case the bench's own equivalent
        // (`feedBounce`) does not guard against.
        rawOver = prev !== 0 && prev > 0 !== next > 0 ? 0 : next;
        applyBounce();
    }
    // The rubber-band return, driven by us (never the discrete-paging
    // `scrollBy`-by-delta helper -- invariant 3) so it can be SUPERSEDED
    // rather than merely cancelled: a
    // fresh drag starting mid-ease bumps `bounceToken` (see `onPointerDown`),
    // and the superseded loop below stops touching `rawOver` on its very
    // next frame instead of racing a new gesture's own `feedOverdrag`
    // writes to the same variable -- invariant 4, restated for a token
    // rather than the bench's own boolean guard, which lets a stale
    // in-flight loop free to keep writing after being 'cancelled.'
    function bounceHome(done) {
        const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        const from = rawOver;
        if (!from || reduce) {
            rawOver = 0;
            applyBounce();
            done && done();
            return;
        }
        const myToken = ++bounceToken;
        const t0 = performance.now();
        const DUR = 340;
        (function step() {
            if (myToken !== bounceToken) {
                return;
            }
            const t = Math.min(1, (performance.now() - t0) / DUR);
            const k = 1 - Math.pow(1 - t, 3); // ease-out cubic
            rawOver = from * (1 - k);
            applyBounce();
            if (t < 1) {
                window.requestAnimationFrame(step);
            } else {
                rawOver = 0;
                applyBounce();
                done && done();
            }
        })();
    }

    const onPointerDown = (e) => {
        // Mouse and pen only -- touch already scrolls this track natively
        // (this module's own 'never intercept touch' rule -- touch-action
        // is never set to 'none' here, see style.css's own comment), and a
        // second pointer going down while one is already tracked is
        // ignored rather than restarting the gesture from the new one.
        if (e.pointerType === 'touch' || pointerId !== null || e.button !== 0) {
            return;
        }
        suppressNextClick = false;
        pointerId = e.pointerId;
        edgeAllowed = { min: null, max: null };
        originX = lastX = e.clientX;
        originY = lastY = e.clientY;
        dragging = false;
        // Supersede any spring-back still easing from the previous gesture
        // (invariant 4) -- this new gesture's own `feedOverdrag` is about to
        // start writing `rawOver` again, and a stale `bounceHome` loop must
        // stop touching it rather than race those writes.
        bounceToken++;
        coastToken++;
        samples = [{ t: e.timeStamp, p: axisPos(e) }];
        if (cancelPendingSnapRestore !== null) {
            // The previous gesture's own scroll-snap-type restore never
            // got to run -- harmless to abandon: this new drag's own
            // threshold-crossing below sets scroll-snap-type to 'none'
            // unconditionally regardless of its current value, and this
            // new drag's own release will schedule a fresh restore.
            // Leaving the old one pending instead would risk it firing
            // *during* this new drag and re-enabling snap mid-gesture --
            // exactly the 'fights every scrollBy' failure mode this
            // module's own doc describes.
            cancelPendingSnapRestore();
            cancelPendingSnapRestore = null;
        }
    };

    const onPointerMove = (e) => {
        if (e.pointerId !== pointerId) {
            return;
        }
        if (!dragging) {
            const dxFromOrigin = e.clientX - originX;
            const dyFromOrigin = e.clientY - originY;
            if (dxFromOrigin * dxFromOrigin + dyFromOrigin * dyFromOrigin < thresholdSq) {
                return;
            }
            dragging = true;
            el.setAttribute('data-dragging', 'true');
            dxTrace(el, 'drag-start', { pointerType: e.pointerType });
            try { el.setPointerCapture(pointerId); } catch (err) {}
            // `scroll-snap-type: ... mandatory` (set inline by this same
            // element's own Rust-rendered `style`) re-snaps after EVERY
            // individual scroll operation it sees as complete -- and each
            // `scrollBy` below is exactly that, since nothing tells the
            // browser these many small instant scrolls are one ongoing
            // gesture the way a real touch/wheel drag would. Left enabled,
            // a single ~80px drag snaps forward one slide per `scrollBy`
            // call and lands 4 slides away instead of 1. Suspending it for
            // the drag's own duration lets these calls move the track
            // freely; `endDrag` below settles it back on release -- see
            // that function's own doc, and this element's own 'Release
            // settle' doc, for why that settle is not simply 'restore this
            // property and let the browser re-snap'.
            el.style.scrollSnapType = 'none';
        }
        // Only reached once dragging -- a still-below-threshold move never
        // calls this, so a plain click's own tiny jitter never suppresses
        // text selection/native drag-ghost for nothing.
        e.preventDefault();
        const dx = e.clientX - lastX;
        const dy = e.clientY - lastY;
        {
            // The event's own timestamp, not the handler's run time: a busy
            // main thread delays handlers but not the input times.
            const now = e.timeStamp;
            samples.push({ t: now, p: axisPos(e) });
            while (samples.length > 2 && now - samples[0].t > 100) samples.shift();
        }
        lastX = e.clientX;
        lastY = e.clientY;
        // scrollBy's own delta is always physical-pixel, regardless of
        // `dir` -- unlike reading/writing `scrollLeft` itself, whose
        // zero-point and sign this module's own doc already warns
        // disagree across browsers, `scrollBy`'s relative
        // delta does not. So this formula never branches on direction --
        // and still gets the well-known RTL swap for free (the keyboard
        // handler, `carousel_key_intent`, does not swap): dragging is direct manipulation (the
        // content tracks the pointer 1:1), and content tracking the
        // pointer is itself a physical, dir-independent relationship --
        // it is *which physical direction reveals 'the next slide'* that
        // flips under RTL (DOM order never changes, but RTL lays later
        // slides physically further left, the mirror of LTR), exactly
        // like the familiar swipe-left-for-next vs. swipe-right-for-next
        // split between LTR and RTL photo galleries/story viewers.
        // On the `rtl` variant, dragging left from slide 1 hits the start
        // boundary and never moves (correct -- there is nothing before
        // slide 1); dragging right produces the identical `scrollLeft`
        // change (`0 -> -153`) the Next button's own `scrollBy`-by-delta
        // produces.
        //
        // Routed through `feedOverdrag` rather than a bare `scrollBy`
        // (mode B3 port, see this constant's own 'Edge rubber-band' doc
        // above): mid-range this is a no-op wrapper around the identical
        // `scrollBy` call, and at an edge it is what
        // turns the refused remainder into the bounce transform.
        feedOverdrag(orientation === 'horizontal' ? dx : dy);
    };

    // The drag came to rest WITHOUT a release-settle scroll (already exactly
    // on a slide -- always true for a drag that ends clamped against an
    // edge). Every `scrollend` during the drag was dropped by the tracking
    // bridge's `data-dragging` guard, so no scroll event will ever report
    // where it rested: say so explicitly. `CAROUSEL_SCROLL_TRACKING_JS`
    // treats this exactly like a `scrollend` (through its idle gate). Only
    // this script knows when its release is truly finished -- after any
    // spring-back, and only when no settle scroll follows.
    const announceRest = () => el.dispatchEvent(new Event('dx-carousel-rest'));

    const endDrag = (e) => {
        if (e.pointerId !== pointerId) {
            return;
        }
        if (dragging) {
            const releaseT = e.timeStamp;
            suppressNextClick = true;
            el.removeAttribute('data-dragging');
            try { el.releasePointerCapture(pointerId); } catch (err) {}

            // Mode B3: ease any overdrag back to identity BEFORE searching
            // for the nearest slide below -- that search reads live
            // `getBoundingClientRect` geometry, which the bounce transform
            // itself would otherwise skew. When there was no overdrag
            // (the overwhelmingly common case) `bounceHome` invokes this
            // callback immediately, so release is unaffected. This never restarts an in-flight ease (invariant
            // 4) -- see `bounceHome`'s own doc.
            bounceHome(() => {
            // `drag_free` (`data-drag-free`, the one switch the CSS and
            // Rust also read): no settle onto a slide -- coast on the
            // release velocity and rest wherever friction stops it.
            if (el.getAttribute('data-drag-free') === 'true') {
                const TAU = 325; // ms: velocity decay constant
                const MIN_V = 0.03; // px/ms: below this the coast is over
                const MAX_V = 2.5; // px/ms: fling velocity cap (about 800px of coast)
                const now = releaseT;
                const last = samples[samples.length - 1];
                const first = samples[0];
                let v = 0; // scroll velocity, px/ms (content follows the pointer, so scroll opposes it)
                if (last && first && last.t > first.t && now - last.t < 80) {
                    v = -(last.p - first.p) / (last.t - first.t);
                }
                v = Math.max(-MAX_V, Math.min(MAX_V, v));
                samples = [];
                const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
                if (reduced || Math.abs(v) < MIN_V * 4) {
                    dxTrace(el, 'drag-release', { coast: false, reduced, v });
                    announceRest();
                    return;
                }
                const myCoast = ++coastToken;
                let prevT = performance.now();
                let carry = 0;
                dxTrace(el, 'drag-release', { coast: true, v });
                (function step() {
                    if (myCoast !== coastToken) return;
                    const t = performance.now();
                    const dt = Math.min(48, t - prevT);
                    prevT = t;
                    // Whole pixels only: a sub-pixel scrollBy can round to
                    // nothing and would read as a refused (edge) write.
                    carry += v * dt;
                    const delta = Math.trunc(carry);
                    carry -= delta;
                    let moved = 0;
                    if (delta !== 0) {
                        const before = contentOffset();
                        if (orientation === 'horizontal') {
                            el.scrollBy({ left: delta, behavior: 'instant' });
                        } else {
                            el.scrollBy({ top: delta, behavior: 'instant' });
                        }
                        moved = Math.abs(contentOffset() - before);
                    }
                    v *= Math.exp(-dt / TAU);
                    if (Math.abs(v) < MIN_V || (delta !== 0 && moved < Math.abs(delta) / 2)) {
                        announceRest();
                        return;
                    }
                    window.requestAnimationFrame(step);
                })();
                return;
            }
            // This element's own 'Release settle' doc has the full
            // reasoning; short version: find the slide nearest the raw
            // drag position -- the same getBoundingClientRect-based
            // geometry `use_carousel_scroll_tracking`'s own settle()
            // already uses elsewhere on this identical element, since a
            // fresh `document::eval` string cannot import a shared JS
            // helper -- and explicitly `scrollBy` this same element (`el`,
            // the scroller itself -- never `nearest.scrollIntoView()`,
            // which would also walk and scroll every scrollable ancestor,
            // including the page) by the physical delta between `nearest`'s
            // own leading edge and this element's, with `behavior: 'smooth'`
            // (respecting reduced motion, the exact policy
            // CAROUSEL_SCROLL_TO_JS already applies to every other paging
            // path -- see that constant's own doc for why a scroller-only
            // `scrollBy` is used instead of `scrollIntoView` everywhere in this
            // module), rather than leaving the settle to however restoring
            // `scroll-snap-type` happens to behave.
            const children = Array.from(el.children);
            let nearest = null;
            let nearestDist = Infinity;
            if (children.length > 0) {
                const containerRect = el.getBoundingClientRect();
                const containerStart = orientation === 'horizontal' ? containerRect.left : containerRect.top;
                const containerEnd = orientation === 'horizontal' ? containerRect.right : containerRect.bottom;
                children.forEach((child) => {
                    const rect = child.getBoundingClientRect();
                    const childStart = orientation === 'horizontal' ? rect.left : rect.top;
                    const childEnd = orientation === 'horizontal' ? rect.right : rect.bottom;
                    const dist = Math.abs(
                        anchorOf(childStart, childEnd) - anchorOf(containerStart, containerEnd)
                    );
                    if (dist < nearestDist) {
                        nearestDist = dist;
                        nearest = child;
                    }
                });
            }

            const restoreSnap = () => {
                el.style.scrollSnapType = el.getAttribute('data-snap') || (orientation === 'horizontal' ? 'x mandatory' : 'y mandatory');
                cancelPendingSnapRestore = null;
            };
            if (nearest) {
                const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
                const settleBehavior = reduced ? 'auto' : 'smooth';
                const s = el.getBoundingClientRect();
                const t = nearest.getBoundingClientRect();
                const sStart = orientation === 'horizontal' ? s.left : s.top;
                const sEnd = orientation === 'horizontal' ? s.right : s.bottom;
                const tStart = orientation === 'horizontal' ? t.left : t.top;
                const tEnd = orientation === 'horizontal' ? t.right : t.bottom;
                const settleDelta = anchorOf(tStart, tEnd) - anchorOf(sStart, sEnd);
                // `nearest` is already exactly where it should be (the
                // drag released right on a boundary) -- `scrollBy({left:
                // 0})` never fires `scrollend` (CAROUSEL_SCROLL_TO_JS's
                // own doc has the same finding), and `scroll-snap-type` is
                // ALREADY 'none' from this drag's own start, so unlike
                // that constant's own zero-delta case (which can just
                // touch nothing) restoring must happen synchronously here
                // -- there is no scroll for a deferred listener/timeout to
                // ever key off, and skipping it entirely would leave
                // snapping suspended until the next gesture happens to
                // trigger it.
                if (Math.abs(settleDelta) < 1) {
                    dxTrace(el, 'drag-release', { settleDelta, settle: 'none (already aligned)', rest: true });
                    restoreSnap();
                    announceRest();
                } else {
                    dxTrace(el, 'scroll-write', { by: 'drag-release-settle', delta: settleDelta, behavior: settleBehavior, snapBefore: el.style.scrollSnapType });
                    // Deferred, not synchronous: setting scroll-snap-type
                    // back to mandatory on an already-stationary position is
                    // what makes release never animate in the first place
                    // (see 'Release settle'). Waiting for this
                    // specific scroll to actually finish (`scrollend`,
                    // falling back to a timeout where unsupported -- the same
                    // feature detection `use_carousel_scroll_tracking` already
                    // uses) means scroll-snap-type is still 'none' for the
                    // whole smooth-scroll animation this call starts, so
                    // there is nothing stationary for the browser to
                    // instantly correct until that animation has already
                    // finished on its own.
                    if ('onscrollend' in window) {
                        // listener-ok: a one-shot `{ once: true }` scrollend that restores snapping after this write's own scroll -- removed by firing or by the next write's cancel, and it must outlive this script (a teardown mid-settle would leave snapping off)
                        el.addEventListener('scrollend', restoreSnap, { once: true, passive: true });
                        cancelPendingSnapRestore = () => el.removeEventListener('scrollend', restoreSnap);
                    } else {
                        const timer = setTimeout(restoreSnap, snapRestoreFallbackMs);
                        cancelPendingSnapRestore = () => clearTimeout(timer);
                    }
                    if (orientation === 'horizontal') {
                        el.scrollBy({ left: settleDelta, behavior: settleBehavior });
                    } else {
                        el.scrollBy({ top: settleDelta, behavior: settleBehavior });
                    }
                }
            } else {
                // No children at all (shouldn't happen in practice) --
                // fall back to an unconditional restore
                // rather than leaving scroll-snap-type suspended forever.
                restoreSnap();
                announceRest();
            }
            });
        }
        pointerId = null;
        dragging = false;
    };

    // Capture phase, attached to this element itself (an ancestor of
    // every slide, so this still only ever sees clicks that landed
    // somewhere inside this carousel's own content): fires before the
    // click reaches whatever child it landed on (a link, a button) and
    // before Dioxus's own delegated bubble-phase handling, so both a
    // native default action (link navigation) and any `onclick` a caller
    // attached are suppressed together -- exactly once, only immediately
    // after a real drag, never for a plain click that never crossed the
    // threshold.
    const onClickCapture = (e) => {
        if (suppressNextClick) {
            suppressNextClick = false;
            e.preventDefault();
            e.stopPropagation();
        }
    };

    listen(el, 'pointerdown', onPointerDown);
    listen(el, 'pointermove', onPointerMove);
    listen(el, 'pointerup', endDrag);
    listen(el, 'pointercancel', endDrag);
    listen(el, 'lostpointercapture', endDrag);
    listen(el, 'click', onClickCapture, true);");

/// Attach [`CAROUSEL_DRAG_JS`] to the element with the given `id` for as
/// long as the calling component stays mounted and `enabled()` is `true`
/// -- mirrors [`use_carousel_scroll_tracking`]'s own shape exactly (an
/// initial `.arg(..)`, the derived `use_js_listeners` teardown), one layer simpler since
/// this bridge reports nothing back to Rust at all (see [`CAROUSEL_DRAG_JS`]'s
/// own doc for why: every `scrollBy` it issues -- including `endDrag`'s
/// own release settle -- is picked up by [`use_carousel_scroll_tracking`]'s
/// own listener on the same element, which is what recomputes both
/// `selected` and the a11y contract's "visible" set once a real,
/// `data-dragging`-free scroll/scrollend event follows the release).
///
/// `enabled` is read with tracked syntax so toggling
/// [`CarouselContentProps::draggable`] at runtime actually attaches/tears
/// down the listeners, not just the initial mount value -- the gate itself
/// lives in the script (`if (!el || !enabled) { ...; return; }`) rather
/// than in whether this hook installs an eval at all, so every path
/// through this function sends the exact same shape of config.
pub(super) fn use_carousel_drag(
    id: impl Readable<Target = String> + Copy + 'static,
    orientation: ReadSignal<CarouselOrientation>,
    align: ReadSignal<CarouselAlign>,
    enabled: ReadSignal<bool>,
) {
    crate::js_listener::use_js_listeners(move || {
        let id = id.cloned();
        let orientation_str = orientation().as_str().to_string();
        let align_str = align().as_str().to_string();
        // No `on_message` here, unlike `use_carousel_scroll_tracking` just
        // above -- this script never `dioxus.send`s anything back (see this
        // function's own doc).
        Some(
            crate::js_listener::JsListeners::install(CAROUSEL_DRAG_JS).arg((
                id,
                orientation_str,
                CAROUSEL_DRAG_THRESHOLD_PX,
                enabled(),
                CAROUSEL_SNAP_RESTORE_FALLBACK_MS,
                align_str,
            )),
        )
    });
}
