use super::*;

/// Fire-and-forget: scroll the carousel's own scroller element
/// (`scrollerId`) by the physical pixel delta needed to align
/// `targetId`'s slide with the scroller's own leading edge, along the
/// carousel's own axis only -- `scroller.scrollBy({ left: delta })` (or
/// `top` for [`CarouselOrientation::Vertical`]).
///
/// This is not `target.scrollIntoView()` (backlog row 91 holds the
/// reasoning behind that choice): `scrollIntoView` walks and scrolls *every*
/// scrollable ancestor between the target and the viewport, including the
/// document itself, in order to bring the target fully into view -- so
/// autoplay/Next would drag the whole page back to the carousel.
/// `Element.scrollBy()`, called directly
/// on the scroller, only ever writes that one element's own scroll
/// position, so there is no ancestor for it to reach in the first place --
/// the page cannot be dragged, by construction.
///
/// The delta is a `getBoundingClientRect()` difference -- a physical
/// pixel value -- so, per invariant 1 of
/// `dev-docs/archive/research/carousel-overscroll-2026-09-23.md` §6, it needs no
/// RTL sign-flip: `scrollBy`'s delta is always physical, the same
/// property this module's own drag gesture (`CAROUSEL_DRAG_JS`'s
/// `feedOverdrag`) already relies on for its own `scrollBy` calls. The
/// `endDrag` release settle below duplicates this exact computation
/// rather than importing it -- a fresh `document::eval` string is its own
/// standalone script with no module system to import across (see
/// [`CarouselContent`]'s own "Release settle" doc for the established
/// precedent of duplicating rather than sharing JS this way).
///
/// `instant` forces `behavior: 'auto'` on the very first call (mount), so
/// a non-zero `default_value` does not visibly animate in on page load;
/// every later call additionally respects `prefers-reduced-motion` the
/// same way. No response is read back -- [`CAROUSEL_SCROLL_TRACKING_JS`]
/// is what keeps `selected` in sync with wherever the scroll position
/// actually ends up.
///
/// `scroll-snap-type` is suspended for the duration of the scroll, then
/// restored once it actually finishes -- the exact same construction
/// [`CAROUSEL_DRAG_JS`]'s own release settle already uses, and for the
/// identical reason (that constant's own "Release settle" doc has the
/// full account): `scrollBy({ behavior:
/// 'smooth' })` on a `scroll-snap-type: ... mandatory` container, left
/// enabled, truncates a multi-slide jump at the FIRST intervening snap
/// point rather than reaching the requested delta -- a bare
/// `el.scrollBy({left: 672, behavior: 'smooth'})` on this element lands at
/// `336`, one slide short, with snapping left enabled, and reaches `672`
/// exactly with `scrollSnapType` set to `'none'` first. A single-slide step
/// (the overwhelmingly common case) happens to truncate at the only snap
/// point in its path anyway, which is also the correct destination, so it
/// reads as correct in isolation; a multi-slide jump (`loop`'s rewind
/// wraparound, a `CarouselIndicator` activation more than one tab away,
/// [`CarouselApi::scroll_to`] to a distant index) is what exposes it
/// (`carousel.spec.ts`'s `clicking a tab activates its slide and moves the
/// roving tab stop` jumps tab 1 -> tab 3). Since a fresh `document::eval` call has no
/// closure shared with any other call (unlike `CAROUSEL_DRAG_JS`'s single
/// long-lived script), the "is a restore already pending" hand-off
/// [`CAROUSEL_DRAG_JS`]'s `onPointerDown` keeps in a closure variable is
/// instead stashed directly on the scroller element itself
/// (`scroller.__dxCancelSnapRestore`) -- a plain DOM property survives
/// across independent eval calls the same way the element's own
/// `style.scrollSnapType` already does, so two paging calls in quick
/// succession (before the first one's own scroll has finished) still
/// cancel-and-replace rather than race a stale restore into firing mid-
/// second-scroll. Reuses [`CAROUSEL_SNAP_RESTORE_FALLBACK_MS`] for the
/// same non-`scrollend`-browser fallback [`CAROUSEL_DRAG_JS`] already
/// needs it for.
///
/// **Every call waits for the browser to be idle.** Its whole body --
/// including measuring the delta and suspending `scroll-snap-type` -- runs
/// through the scroller's `__dxWhenIdle` gate
/// ([`CAROUSEL_SCROLL_TRACKING_JS`], [`CAROUSEL_INPUT_QUIET_MS`]), so a write
/// requested while wheel/touch input is still arriving (the virtual window's
/// re-align, or a button/key/autoplay/indicator page) happens once that input
/// has gone quiet, measured from wherever the browser actually came to rest
/// -- never mid-gesture (THE RULE). With input already quiet it runs at once.
pub(super) const CAROUSEL_SCROLL_TO_JS: &str = concat!(carousel_trace_js!(), "\
    const [scrollerId, targetId, orientation, instant, snapRestoreFallbackMs, align] = await dioxus.recv();
    const behavior = (instant || window.matchMedia('(prefers-reduced-motion: reduce)').matches)
        ? 'auto' : 'smooth';
    const scroller = document.getElementById(scrollerId);
    const target = document.getElementById(targetId);
    // The point on `rect`, along this carousel's own axis, that
    // `align` (CarouselAlign::as_str()'s own 'start'/'center'/'end')
    // rests against -- applied identically to the target slide and the
    // scroller's own viewport rect below, so e.g. `align: 'center'`
    // aligns the slide's midpoint with the viewport's midpoint rather
    // than always the leading edge. Mirrors the native `scroll-snap-align`
    // keyword `CarouselItem` already sets to the same value -- this is
    // what keeps an explicit `scrollBy`-by-delta paging call consistent
    // with wherever the browser's own native snapping would have already
    // landed.
    const rtlAxis = orientation === 'horizontal' && !!scroller && getComputedStyle(scroller).direction === 'rtl';
    const anchorOf = (rect) => {
        const start = orientation === 'horizontal' ? rect.left : rect.top;
        const end = orientation === 'horizontal' ? rect.right : rect.bottom;
        if (align === 'center') return (start + end) / 2;
        if (align === 'end') return rtlAxis ? start : end;
        return rtlAxis ? end : start;
    };
    // A real drag (CAROUSEL_DRAG_JS) already owns the scroll position and
    // `scroll-snap-type` for as long as `data-dragging` is set:
    // `use_carousel_scroll_tracking` reports a
    // changing `selected` continuously WHILE a drag is still in progress
    // (every incremental `feedOverdrag` scrollBy fires its own `scrollend`
    // too), so this effect fires repeatedly mid-gesture. A
    // `scrollIntoView` would tolerate that safely --
    // already-near-enough is a harmless no-op (this effect's own Rust-side
    // doc comment documents this exact case). A `scrollBy`-by-delta
    // is not equally forgiving: it competes with the drag's own direct-
    // manipulation `scrollBy` calls for the same scroll position, and (see
    // `CAROUSEL_DRAG_JS`'s 'Pointer drag' doc) its own scroll-snap-type
    // restore firing mid-drag
    // re-enables mandatory snapping before the gesture ends, which then
    // fights every remaining `feedOverdrag` call and sends the drag flying
    // several slides past where the pointer actually stopped (about 2
    // slide-pitches in the pointer-drag tests). Skipping entirely while `data-dragging` is set
    // defers this bridge to the drag's own release settle, which already
    // performs the identical scroll-to-nearest-slide alignment once the
    // gesture actually ends.
    // Never write the scroll position while user scroll input is still
    // arriving (THE RULE): wait on the tracking bridge's browser-idle gate
    // (`CAROUSEL_SCROLL_TRACKING_JS`, `CAROUSEL_INPUT_QUIET_MS`), and measure
    // the delta only once it lets this run, from wherever the browser came
    // to rest. Runs at once when input is already quiet, or when no bridge
    // is attached yet (the very first mount settle).
    const run = () => {
    if (!(scroller && target && target.isConnected && !scroller.hasAttribute('data-dragging'))) {
        dxTrace(scroller, 'scroll-to-skip', { targetId, why: !scroller ? 'no scroller' : !target ? 'no target' : !target.isConnected ? 'target gone' : 'dragging' });
    }
    if (scroller && target && target.isConnected && !scroller.hasAttribute('data-dragging')) {
        const s = scroller.getBoundingClientRect();
        const t = target.getBoundingClientRect();
        const delta = anchorOf(t) - anchorOf(s);
        // A genuine no-op (already aligned -- the mount settle when
        // `default_value` needs no jump, or `use_carousel_scroll_tracking`
        // reporting the position a native scroll already reached) must
        // touch NOTHING, the same as `scrollIntoView` on an
        // already-fully-visible element: `scrollBy({ left: 0 })` never
        // fires `scrollend` (nothing scrolled), so suspending
        // `scroll-snap-type` for a scroll that is never going to happen
        // would leave it stuck at `none` forever (the mount settle's own
        // zero-delta call is the common case that hits this on every
        // single carousel).
        if (Math.abs(delta) < 1) {
            dxTrace(scroller, 'scroll-to-skip', { targetId, why: 'aligned', delta });
            delete scroller.__dxIntent;
            return;
        }
        // Cancel a still-pending restore from a previous paging call, the
        // same idiom CAROUSEL_DRAG_JS's own onPointerDown uses -- see this
        // constant's own doc for why this is stashed on the element rather
        // than a closure variable.
        if (scroller.__dxCancelSnapRestore) {
            scroller.__dxCancelSnapRestore();
            scroller.__dxCancelSnapRestore = null;
        }
        dxTrace(scroller, 'scroll-write', { by: 'scroll-to', targetId, delta, behavior, instant, snapBefore: scroller.style.scrollSnapType });
        scroller.style.scrollSnapType = 'none';
        // Declare where this write is meant to land, for the tracking
        // bridge's settle to verify (`CAROUSEL_SCROLL_TRACKING_JS`, 'Write
        // intent'): the delta is measured against the layout of THIS frame,
        // and the layout can still change before the scroll comes to rest.
        scroller.__dxIntent = { targetId, tries: 0 };
        if (orientation === 'horizontal') {
            scroller.scrollBy({ left: delta, behavior });
        } else {
            scroller.scrollBy({ top: delta, behavior });
        }
        const restoreSnap = () => {
            scroller.style.scrollSnapType = scroller.getAttribute('data-snap') || (orientation === 'horizontal' ? 'x mandatory' : 'y mandatory');
            scroller.__dxCancelSnapRestore = null;
        };
        if ('onscrollend' in window) {
            // listener-ok: a one-shot `{ once: true }` scrollend that restores snapping after this write's own scroll -- removed by firing or by the next write's cancel, and it must outlive this fire-and-forget script
            scroller.addEventListener('scrollend', restoreSnap, { once: true, passive: true });
            scroller.__dxCancelSnapRestore = () => scroller.removeEventListener('scrollend', restoreSnap);
        } else {
            const timer = setTimeout(restoreSnap, snapRestoreFallbackMs);
            scroller.__dxCancelSnapRestore = () => clearTimeout(timer);
        }
    }
    };
    if (scroller && scroller.__dxWhenIdle) {
        dxTrace(scroller, 'scroll-to-request', { targetId, instant, viaGate: true });
        scroller.__dxWhenIdle(run);
    } else {
        dxTrace(scroller, 'scroll-to-request', { targetId, instant, viaGate: false });
        run();
    }");

/// Long-lived (mount-to-unmount): translate a [`CarouselContent`]
/// element's *actual* scroll position into a slide index, so `selected`
/// stays correct after a native drag/wheel/trackpad scroll -- one this
/// crate never drove via [`CAROUSEL_SCROLL_TO_JS`] itself.
///
/// Deliberately not an `IntersectionObserver` (the shape
/// `dev-docs/archive/research/carousel-2026-09-19.md` §2.2 sketches as the
/// primary mechanism): that needs live per-item intersection-ratio
/// bookkeeping and threshold tuning to avoid mid-animation flicker
/// between two partially-visible neighbors. Settling once, on
/// `scrollend` (falling back to a debounced plain `scroll` listener where
/// `scrollend` is absent -- Safari only from v26.2 per the research's
/// own caniuse check, near 90% global support) and then comparing every
/// slide's `getBoundingClientRect()` against the container's own, is
/// simpler, fires exactly once per gesture (no flicker to begin with),
/// and is still zero-JS-dependency, feature-detected, degrade-not-break,
/// matching this repo's own platform-features-first posture
/// (`dev-docs/reference/platform-rationale.md`'s `<dialog>` finding).
/// **"Once" means once the browser is truly idle, not on the first
/// `scrollend`.** The settle waits on this bridge's browser-idle gate: the
/// latest scroll has ended AND no wheel/touch input for
/// [`CAROUSEL_INPUT_QUIET_MS`] -- see that constant's own doc for the class
/// of bug this closes and everything it subsumes. The same gate is exposed
/// on the scroller as `__dxWhenIdle` (a plain DOM property, the same
/// cross-eval hand-off [`CAROUSEL_SCROLL_TO_JS`]'s `__dxCancelSnapRestore`
/// already uses), which every [`CAROUSEL_SCROLL_TO_JS`] write waits on.
///
/// Comparing viewport (`getBoundingClientRect`) positions rather than
/// `scrollLeft`/`offsetLeft` sidesteps the well-known cross-browser
/// disagreement over what `scrollLeft` itself means under `dir="rtl"` --
/// a `getBoundingClientRect` is always a physical, direction-agnostic
/// viewport rectangle regardless of `dir`.
///
/// One read (`getBoundingClientRect`, of the container and every slide)
/// followed by nothing but a `dioxus.send` -- no DOM write at all in this
/// callback -- so there is no read-then-write-layout main-thread hazard
/// (`dev-docs/reference/platform-rationale.md` §11 Rule 1) to begin with.
/// Both listeners are `passive: true` (§11 Rule 2).
pub(super) const CAROUSEL_SCROLL_TRACKING_JS: &str = concat!(carousel_true_end_js!(), carousel_trace_js!(), "\
    const [id, orientation, align, inputQuietMs, virtualWindow] = await dioxus.recv();
    const container = document.getElementById(id);
    if (!container) return;
    // See `CAROUSEL_SCROLL_TO_JS`'s own identical helper's doc: which
    // slide counts as 'nearest' (and therefore `selected`) must anchor on
    // the same point `align` snaps to, so this agrees with wherever the
    // browser's own native scroll-snap settling already landed.
    // Logical start/end: under `dir=rtl` the leading edge is the physical
    // right (what `scroll-snap-align: start` means there), so a free
    // (unsnapped) position finds the same 'nearest' slide the snap would.
    const rtlAxis = orientation === 'horizontal' && getComputedStyle(container).direction === 'rtl';
    const anchorOf = (start, end) => {
        const lead = rtlAxis ? end : start;
        const trail = rtlAxis ? start : end;
        if (align === 'center') return (start + end) / 2;
        if (align === 'end') return trail;
        return lead;
    };
    let lastNearest = null;
    let lastNearestPos = null;
    let lastVisStart = null;
    let lastVisEnd = null;
    const settle = () => {
        // Freeze rule (dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md §7 / dev-docs/archive/research/carousel-overscroll-2026-09-23.md
        // §6): a live mouse/pen drag (`CAROUSEL_DRAG_JS`) fires its own
        // 'scroll'/'scrollend' on every single incremental `scrollBy` it
        // issues (`CAROUSEL_SCROLL_TO_JS`'s own doc,
        // 'A real drag already owns the scroll position' paragraph), so
        // without this guard `selected` (and the visible range this
        // drives `inert` from) would change continuously mid-drag rather
        // than only at rest. `data-dragging` is removed at the very start
        // of `endDrag`, before its own release-settle `scrollBy` runs, so
        // this never suppresses the genuine post-release settle -- only
        // the mid-gesture noise. `CAROUSEL_DRAG_JS`'s own release settle
        // reuses this exact geometry search for the identical reason (see
        // that constant's own 'Release settle' doc) and is what a drag
        // defers to instead.
        if (container.hasAttribute('data-dragging')) {
            return;
        }
        // ---- Write intent: a component write lands on its declared slide,
        // whatever the layout did meanwhile. `CAROUSEL_SCROLL_TO_JS` measures
        // its delta against the layout of the frame it writes in, with
        // `scroll-snap-type` suspended -- so nothing re-snaps if the slides
        // change size before that scroll rests. Example: the mount re-align of
        // a 12-item virtual loop writes 2080px against 416px slides; if the themed
        // stylesheet then shrinks them to 320px before `scrollend`, the settle
        // reads position 1 at a 160px offset, which would re-anchor the window and
        // select slide 2 of 12. So the first
        // settle after a write checks the declared target first: still
        // misaligned and no user input since, it re-aligns (instant, at most
        // twice per write) and lets that scroll's own end settle. Any user
        // input (`clearIntent`) drops the intent -- the user owns the
        // position from then on; so does the first settle that finds it met.
        const intent = container.__dxIntent;
        if (intent) {
            const target = document.getElementById(intent.targetId);
            if (target && target.isConnected && intent.tries < 2) {
                const t = target.getBoundingClientRect();
                const c = container.getBoundingClientRect();
                const d = orientation === 'horizontal'
                    ? anchorOf(t.left, t.right) - anchorOf(c.left, c.right)
                    : anchorOf(t.top, t.bottom) - anchorOf(c.top, c.bottom);
                if (Math.abs(d) >= 1) {
                    intent.tries++;
                    dxTrace(container, 'intent-correct', { targetId: intent.targetId, delta: d, tries: intent.tries });
                    if (orientation === 'horizontal') {
                        container.scrollBy({ left: d, behavior: 'instant' });
                    } else {
                        container.scrollBy({ top: d, behavior: 'instant' });
                    }
                    return;
                }
            }
            delete container.__dxIntent;
        }
        const children = Array.from(container.children);
        if (children.length === 0) {
            return;
        }
        const containerRect = container.getBoundingClientRect();
        const containerStart = orientation === 'horizontal' ? containerRect.left : containerRect.top;
        const containerEnd = orientation === 'horizontal' ? containerRect.right : containerRect.bottom;
        let nearest = 0;
        let nearestPos = 0;
        let nearestDist = Infinity;
        // Which item indices are actually inside the viewport right now --
        // an item counts as visible if its own midpoint falls within the
        // container's rect (\"at least half the slide is in view\", not a
        // stricter full-containment test, which would flicker a slide out
        // right at a scroll-snap boundary's own sub-pixel settle, nor a
        // looser any-overlap test, which would count a 1px sliver of a
        // fully-clipped neighbour as \"visible\" for a multi-per-view
        // layout with no peeking at all). Contiguous by construction (flex
        // row/column, no reordering), so only the first/last matching
        // index need to be kept.
        //
        // Every mapping from a DOM child to a slide reads `data-index`
        // (the stable data index -- may repeat across children in a
        // virtualised loop window smaller than the data set) and
        // `data-position` (the unique logical slot -- equals `data-index`
        // for the plain children API, so this one code path serves both;
        // `CarouselVirtualContent`'s own doc has the fuller account) --
        // never the child's own array position `i`. Why: with a single
        // leading non-slide DOM child, an index-position-based version of
        // this exact search runs away (`dev-docs/archive/research/carousel-loop-2026-09-25/loop-libraries.md`).
        let visStart = null;
        let visEnd = null;
        children.forEach((child) => {
            const dataIndex = parseInt(child.dataset.index, 10);
            const position = child.dataset.position !== undefined
                ? parseInt(child.dataset.position, 10)
                : dataIndex;
            if (Number.isNaN(dataIndex)) {
                return;
            }
            const rect = child.getBoundingClientRect();
            const childStart = orientation === 'horizontal' ? rect.left : rect.top;
            const childEnd = orientation === 'horizontal' ? rect.right : rect.bottom;
            const dist = Math.abs(
                anchorOf(childStart, childEnd) - anchorOf(containerStart, containerEnd)
            );
            if (dist < nearestDist) {
                nearestDist = dist;
                nearest = dataIndex;
                nearestPos = position;
            }
            const center = (childStart + childEnd) / 2;
            if (center >= containerStart && center <= containerEnd) {
                if (visStart === null) {
                    visStart = dataIndex;
                }
                visEnd = dataIndex;
            }
        });
        if (visStart === null) {
            visStart = nearest;
            visEnd = nearest;
        }
        // One wire shape for both message kinds (`[tag, a, b]`) -- `recv`
        // deserializes into a single fixed Rust tuple type regardless of
        // which fired. For 'selected', `b` is the nearest slide's own
        // logical POSITION (not its data index -- these disagree exactly
        // when a virtualised loop window is smaller than the data set),
        // consumed only by a caller that tracks an anchor position
        // (`CarouselVirtualContent`); the plain children API ignores it,
        // since position and data index are the same value there.
        if (nearest !== lastNearest || nearestPos !== lastNearestPos) {
            lastNearest = nearest;
            lastNearestPos = nearestPos;
            dioxus.send(['selected', nearest, nearestPos]);
            dxTrace(container, 'settle', { selected: nearest, position: nearestPos, offset: Math.round(nearestDist * 10) / 10 });
        }
        if (visStart !== lastVisStart || visEnd !== lastVisEnd) {
            lastVisStart = visStart;
            lastVisEnd = visEnd;
            dioxus.send(['visible', visStart, visEnd]);
        }
    };
    // ---- Browser-idle gate: the module's ONE 'the browser is truly idle'
    // signal (see `CAROUSEL_INPUT_QUIET_MS`'s own doc for the class this
    // closes). `scrollend` alone is not idle: a trackpad can end one scroll
    // (the pan) and keep delivering input (its momentum), and a scroll that
    // clamps at an edge ends while the fingers keep pushing. So a reaction to
    // the end of a scroll runs only once BOTH hold: the latest scroll has
    // ended (no scroll event since its `scrollend`), and no user scroll input
    // -- a wheel event, or a finger on the screen -- has arrived for
    // `inputQuietMs`. Every other scroll write this module makes waits on the
    // same signal through `container.__dxWhenIdle` (`CAROUSEL_SCROLL_TO_JS`).
    const supportsScrollEnd = 'onscrollend' in window;
    let debounceTimer = null;
    let lastInputAt = -Infinity;
    let touches = 0;
    let endPending = false;
    let gateTimer = null;
    let waiters = [];
    // ---- The ONE exception to the idle gate: pinned at a false end.
    // A virtual window only re-centres at idle, so a long flick runs into
    // its rendered slice's physical end and sits there, clamped, while the
    // rest of its input keeps arriving (seconds of momentum can still be
    // to come). In exactly that
    // state the browser is NOT moving the scroller -- it cannot -- so a
    // re-anchor plus its instant re-align does not fight a live scroll.
    // Conditions, all required: a virtual window (`virtualWindow`); trusted
    // wheel input still arriving (never during touch -- a finger is a pan
    // the browser owns); the scroller against a physical end whose edge slide
    // is NOT a true end of the data (the shared `dxIsTrueEnd`); and its
    // scroll position unchanged for PINNED_FRAMES consecutive frames. Then
    // the gate lets the pending settle (re-anchor) through, and grants its
    // re-align (which arrives a moment later, possibly after the re-render
    // has already moved the scroller off the edge) until that write runs,
    // or PIN_GRANT_MS at most. Once per pin episode: re-armed only after
    // the scroller has moved again, so a flick that runs into the NEW
    // slice end re-centres again, but never twice for one stall.
    const PINNED_FRAMES = 2;
    const PIN_GRANT_MS = 500;
    const horizontal = orientation === 'horizontal';
    let pinned = false;
    let pinFired = false;
    let pinRaf = 0;
    let pinLastPos = null;
    let pinStable = 0;
    let pinGrantUntil = 0;
    // Trusted wheel input only: the pinned state is about the browser's own
    // scroll, and an untrusted (synthetic) wheel event never scrolls
    // anything, so it can never be the momentum that pins a scroller. (The
    // gate itself still counts every wheel event as input.)
    let lastTrustedWheelAt = -Infinity;
    const pinTick = () => {
        pinRaf = 0;
        if (touches || performance.now() - lastTrustedWheelAt >= inputQuietMs) {
            pinned = false;
            pinStable = 0;
            pinLastPos = null;
            return;
        }
        const kids = container.children;
        if (kids.length) {
            const c = container.getBoundingClientRect();
            let lo = Infinity;
            let hi = -Infinity;
            for (const k of kids) {
                const r = k.getBoundingClientRect();
                lo = Math.min(lo, horizontal ? r.left : r.top);
                hi = Math.max(hi, horizontal ? r.right : r.bottom);
            }
            const side = lo - (horizontal ? c.left : c.top) >= -0.5 ? 'min'
                : hi - (horizontal ? c.right : c.bottom) <= 0.5 ? 'max' : null;
            const pos = horizontal ? container.scrollLeft : container.scrollTop;
            if (side && pos === pinLastPos) {
                pinStable++;
            } else {
                pinStable = 0;
                pinFired = false;
                pinned = false;
            }
            pinLastPos = pos;
            if (side && pinStable >= PINNED_FRAMES && !pinFired && !dxIsTrueEnd(container, horizontal, side)) {
                pinFired = true;
                pinned = true;
                pinGrantUntil = performance.now() + PIN_GRANT_MS;
                if (dxTraceOn) {
                    const ids = Array.from(kids).map((k) => k.dataset.index);
                    dxTrace(container, 'pinned-recentre', { side, frames: pinStable, sinceInput: Math.round(performance.now() - lastInputAt), edgeIndex: side === 'min' ? ids[0] : ids[ids.length - 1], window: ids.join(',') });
                }
                endPending = true;
                pump();
            }
        }
        pinRaf = requestAnimationFrame(pinTick);
    };
    const inputQuiet = () => touches === 0 && performance.now() - lastInputAt >= inputQuietMs;
    let holding = false;
    const pump = () => {
        clearTimeout(gateTimer);
        gateTimer = null;
        if (!endPending && waiters.length === 0) {
            return;
        }
        if (!inputQuiet() && !pinned && performance.now() >= pinGrantUntil) {
            if (!holding) {
                holding = true;
                dxTrace(container, 'gate-hold', { why: touches ? 'touch' : 'input', sinceInput: Math.round(performance.now() - lastInputAt), endPending, waiters: waiters.length });
            }
            // A finger still down re-pumps on its own touchend; otherwise
            // look again the moment the quiet window would close.
            if (touches === 0) {
                gateTimer = setTimeout(pump, Math.max(1, inputQuietMs - (performance.now() - lastInputAt)));
            }
            return;
        }
        if (dxTraceOn) {
            dxTrace(container, 'gate-release', { held: holding, settle: endPending, waiters: waiters.length });
        }
        holding = false;
        if (endPending) {
            endPending = false;
            settle();
        }
        const run = waiters;
        waiters = [];
        if (run.length && pinGrantUntil > performance.now() && !pinned) {
            // The re-centre's own re-align has gone through: end the grant.
            pinGrantUntil = 0;
        }
        run.forEach((f) => f());
    };
    const scrollEnded = (src) => {
        if (dxTraceOn) {
            dxTrace(container, 'scroll-ended', { why: typeof src === 'string' ? src : (src && src.type) || 'unknown' });
        }
        endPending = true;
        pump();
    };
    const onScroll = () => {
        // Scrolling again: whatever ended before has not come to rest after
        // all (a snap animation, momentum resuming) -- wait for the next end.
        endPending = false;
        if (supportsScrollEnd) {
            return;
        }
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => scrollEnded('scroll-debounce'), 120);
    };
    // Any user input supersedes a pending component write's intent (see
    // `settle`'s 'Write intent'): a pointer (a drag, the scrollbar), a key
    // (native keyboard scrolling), a wheel or a finger.
    const clearIntent = () => {
        delete container.__dxIntent;
    };
    const onInput = (e) => {
        lastInputAt = performance.now();
        clearIntent();
        if (dxTraceOn && e && e.type === 'wheel') {
            dxTrace(container, 'input', { dx: e.deltaX, dy: e.deltaY, mode: e.deltaMode, trusted: e.isTrusted });
        }
        if (virtualWindow && e && e.type === 'wheel' && e.isTrusted) {
            lastTrustedWheelAt = lastInputAt;
            if (!pinRaf) pinRaf = requestAnimationFrame(pinTick);
        }
        pump();
    };
    const onTouch = (e) => {
        touches = e.touches.length;
        onInput();
    };
    const whenIdle = (f) => {
        waiters.push(f);
        pump();
    };
    container.__dxWhenIdle = whenIdle;
    // The end of a pointer drag is the end of a scroll, even when its release
    // needs no settle scroll at all (it came to rest exactly on a slide --
    // always the case for a drag that ends clamped against an edge). Every
    // `scrollend` during the drag was dropped by `settle`'s own
    // `data-dragging` guard, so without this nothing would report where it
    // rested: a drag from slide 2 past slide 1 would leave `selected`
    // on slide 2 with slide 1 showing, and a virtual window dragged against
    // its slice edge would never re-centre. `CAROUSEL_DRAG_JS` announces it
    // (`dx-carousel-rest`) only once its release is truly finished -- after
    // any spring-back, and only when no settle scroll follows (that scroll's
    // own `scrollend` reports instead). Guessing it from here (waiting two
    // frames for a scroll that never comes) would fire mid-spring-back and
    // leave a stale sub-pixel transform, so the signal
    // comes from the one party that knows. Routed through the same idle gate.
    listen(container, 'dx-carousel-rest', scrollEnded);
    listen(container, 'pointerdown', clearIntent, { passive: true });
    listen(container, 'keydown', clearIntent, { passive: true });
    listen(container, 'scroll', onScroll, { passive: true });
    listen(container, 'wheel', onInput, { passive: true });
    listen(container, 'touchstart', onTouch, { passive: true });
    listen(container, 'touchend', onTouch, { passive: true });
    listen(container, 'touchcancel', onTouch, { passive: true });
    if (supportsScrollEnd) {
        listen(container, 'scrollend', scrollEnded, { passive: true });
    }
    // A resize of the scroller's own box re-settles too -- not just a
    // scroll -- closing a mount-time race:
    // `CarouselContent`'s own layout depends on the themed wrapper's
    // *external* stylesheet (`document::Link`, loaded asynchronously),
    // which can still be in flight the instant this bridge attaches, so
    // the very first geometry read can measure a temporarily wider box
    // (544px before `.dx-carousel`'s own `padding-inline`
    // reservation applies, 448px after) -- and for a `default_value == 0`
    // carousel that never scrolls again on its own, that first, too-early
    // read would otherwise never be corrected, permanently over-widening
    // the visible range one slide's worth. `ResizeObserver`'s own first
    // callback fires once for the box's *current* size the moment
    // `observe()` is called (so this alone establishes the initial visible
    // range even if the carousel never scrolls at all, the mount case),
    // and fires again on every subsequent
    // genuine size change (that stylesheet finishing, a responsive
    // breakpoint, a caller resizing the wrapper) -- a self-correcting
    // construction for the whole class of 'the box was momentarily the
    // wrong size when this measured it'.
    //
    // Routed through the same idle gate as a scroll end: a resize while a
    // gesture is still in flight settles once the gesture is over.
    const resizeObserver = new ResizeObserver(() => scrollEnded('resize'));
    // Debug only: every re-render of the rendered slides (a virtual window
    // re-anchoring), old -> new data-index / data-position ranges.
    const sliceOf = () => {
        const kids = Array.from(container.children);
        return { idx: kids.map((k) => k.dataset.index).join(','), pos: kids.map((k) => k.dataset.position).join(',') };
    };
    let lastSlice = dxTraceOn ? sliceOf() : null;
    const windowObserver = dxTraceOn ? new MutationObserver(() => {
        const next = sliceOf();
        if (next.idx !== lastSlice.idx || next.pos !== lastSlice.pos) {
            dxTrace(container, 'window', { fromIdx: lastSlice.idx, toIdx: next.idx, fromPos: lastSlice.pos, toPos: next.pos });
            lastSlice = next;
        }
    }) : null;
    if (windowObserver) windowObserver.observe(container, { childList: true });
    resizeObserver.observe(container);
    // The listeners above are `listen`'s (removed by the teardown); the rest is undone here.
    cleanup(() => {
        resizeObserver.disconnect();
        if (windowObserver) windowObserver.disconnect();
        clearTimeout(debounceTimer);
        clearTimeout(gateTimer);
        if (pinRaf) cancelAnimationFrame(pinRaf);
        // Only ever remove this bridge's OWN gate: a re-attach (new options)
        // can install its replacement before this teardown message arrives.
        // Still-waiting scroll writes run rather than being dropped.
        if (container.__dxWhenIdle === whenIdle) {
            delete container.__dxWhenIdle;
        }
        waiters.forEach((f) => f());
    });");

/// Attach [`CAROUSEL_SCROLL_TRACKING_JS`] to the element with the given
/// `id` for as long as the calling component stays mounted, forwarding
/// every reported `('selected', data_index, position)` message to
/// `set_selected` (and, if present, `on_position`) and every `('visible',
/// start, end)` message to `visible_range` -- the a11y contract's own
/// "visible" set (module doc, "Accessibility"; see [`CarouselItem`]'s own
/// "Visible slides / `inert`" doc). A [`use_js_listeners`] script: an
/// initial `.arg(..)` of the (id, orientation, ...) the script's own
/// `await dioxus.recv()` unpacks, an `on_message` handler for as long as
/// the script keeps sending, and the derived teardown (unmount or a changed
/// option) that removes its listeners and runs its `cleanup` before the
/// element is gone.
///
/// [`use_js_listeners`]: crate::js_listener::use_js_listeners
///
/// `on_position`, when `Some`, is called with the nearest slide's own
/// logical POSITION (`data-position`, may differ from its data index) --
/// used only by [`CarouselVirtualContent`] to keep its own anchor
/// bookkeeping in sync with a native drag/wheel/trackpad settle (or with
/// its own animated paging call finishing); `None` for the plain children
/// API ([`CarouselContent`]), where position and data index always agree.
pub(super) fn use_carousel_scroll_tracking(
    id: impl Readable<Target = String> + Copy + 'static,
    orientation: ReadSignal<CarouselOrientation>,
    align: ReadSignal<CarouselAlign>,
    set_selected: Callback<usize>,
    count: Memo<usize>,
    mut visible_range: Signal<(usize, usize)>,
    on_position: Option<Callback<isize>>,
) {
    crate::js_listener::use_js_listeners(move || {
        let id = id.cloned();
        let orientation_str = orientation().as_str().to_string();
        let align_str = align().as_str().to_string();
        Some(
            crate::js_listener::JsListeners::install(CAROUSEL_SCROLL_TRACKING_JS)
                .arg((
                    id,
                    orientation_str,
                    align_str,
                    CAROUSEL_INPUT_QUIET_MS,
                    on_position.is_some(),
                ))
                .on_message(move |(kind, a, b): (String, i64, i64)| {
                    if kind == "selected" {
                        set_selected.call(a.max(0) as usize);
                        if let Some(on_position) = on_position {
                            on_position.call(b as isize);
                        }
                    } else {
                        let count_now = *count.peek();
                        let a = a.max(0) as usize;
                        let b = b.max(0) as usize;
                        visible_range.set(clamp_visible_range((a, b), count_now));
                    }
                }),
        )
    });
}
