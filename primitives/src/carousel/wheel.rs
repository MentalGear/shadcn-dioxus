use super::*;

/// Long-lived (mount-to-unmount): the wheel/trackpad half of the edge
/// rubber-band -- **mode D, "platform-aware band"** -- attached to
/// [`CarouselContent`]'s own element alongside [`CAROUSEL_DRAG_JS`], as a
/// **separate** `document::eval`, not folded into that one, because the two
/// obey different rules and must not share a JS closure that could let one's
/// state leak into the other's: a mouse/pen drag has no native scrolling
/// behind it, so [`CAROUSEL_DRAG_JS`] may keep issuing `scrollBy` (THE RULE,
/// dev-docs/archive/research/carousel-overscroll-2026-09-23.md §3), while a
/// wheel/trackpad gesture on this same element has a compositor and a snap
/// engine also trying to write its scroll position, so this script may
/// **never** call `scrollBy`, `scrollLeft`/`scrollTop`, `preventDefault`, or
/// touch `scroll-snap-type`. The band is a `transform` on this element and
/// nothing else; `transform` is `''` (identity) whenever no band is drawn.
///
/// **What D is, and why it is not B3.** Ported from a standalone bench
/// (`bench-rev40.html`, `mode === "onewriter"`) and tuned on real hardware
/// (MacBook trackpad, Firefox 156); the notes are in
/// `dev-docs/archive/research/carousel-engineering-notes.md`.
/// B3 draws its band whenever the scroller was clamped at an edge, so a
/// flick from slide 3 that arrives at slide 1 with momentum gets B3's band
/// drawn *on top of* the platform's own overscroll -- a double bounce. D
/// decides **ownership once per gesture, from where the scroller RESTS when
/// the gesture starts**: only a gesture that begins at rest at an edge, and
/// pushes into that same edge, gets D's band. A gesture that arrives at the
/// edge from elsewhere is the platform's (native bounce, or none in a browser
/// that has none) and D draws nothing.
///
/// Release (a wheel has no `pointerup`), in the order the bench checks them:
/// 1. **smooth decay** -- fingers lifted, the stream turns into momentum,
///    which falls geometrically; a run of `RUNS` paired events each falling
///    to between `SMOOTH` and `RATIO_MAX` of the last means "lifted";
/// 2. **cursor moved** -- a two-finger scroll never moves the cursor, so a
///    real mouse-pointer displacement of `LIFT_PX` or more is a finger;
/// 3. **slow-push silence** -- a push whose peak step stayed under
///    `SLOW_PEAK_PX` can never end in momentum, so `SLOW_IDLE_MS` of
///    silence releases it;
/// 4. **notched silence** (component addition, not in the bench) -- a
///    notched mouse wheel has no momentum tail and its steps are above
///    `SLOW_PEAK_PX`, so neither test above could fire; `NOTCH_IDLE_MS` of
///    silence releases it;
/// 5. **idle backstop** -- `IDLE_MS` of silence, for anything else;
/// 6. **reversed** -- the stream turned away from the edge.
///
/// A new gesture begins after `GAP_MS` of silence, or at once on a *re-push*:
/// a delta that jumps above `REPRESS` x the last one after a release, or
/// above `COAST_RISE` x the last one once the stream has coasted (momentum
/// only ever falls, so a rise is new fingers -- bench rev 40, a
/// recorded `..., -57, -53, -49, -13, -28, -67, -112` at slide 1). A push
/// that resumes after a release, while the band is still springing home,
/// continues from wherever the spring-back had got to
/// (`webkitRubberInverse`), never from zero.
///
/// **Deliberate differences from the bench:**
/// - *Axis gating.* The bench fell back to `deltaY` when `deltaX` was 0; here
///   a horizontal carousel reacts only to horizontal intent
///   (`|deltaX| > |deltaY|`, or Shift+wheel reported as `deltaY` with
///   `shiftKey`), so ordinary vertical page scrolling over a horizontal
///   carousel resting at slide 1 never draws a band. Vertical orientation
///   reacts only to `|deltaY| > |deltaX|`.
/// - *Notched mouse wheels* release after `NOTCH_IDLE_MS` (item 4 above).
///   Notched = `deltaMode` 1/2, or `deltaMode` 0 with a step of at least
///   `NOTCH_MIN_PX` that repeats the previous step exactly (or a whole
///   multiple of a step already identified as a notch -- a browser may
///   coalesce two notches into one event). Classified per event, so a
///   trackpad stream that happens to repeat one value is back on the
///   trackpad paths at its very next, different, delta.
/// - *Vertical orientation and RTL* (the bench is horizontal LTR only): all
///   geometry is physical (`getBoundingClientRect` deltas, `translateX` or
///   `translateY`), so nothing here branches on direction.
/// - *A leftover band never outlives a gesture that is not drawing it.* In
///   the bench, a new gesture that did not own the edge could freeze a
///   spring-back mid-flight; here a band left over from an earlier gesture
///   springs home as soon as a new gesture turns out not to own it.
/// - *Telemetry* only behind `localStorage['dx-carousel-debug'] === '1'`
///   (or `window.__dxCarouselDebug === true` set by the host page),
///   read once at mount (see "Debug telemetry" below). The bench's knob
///   panel has no counterpart here; its defaults (the tuned values) are the named
///   constants below.
///
/// **Debug telemetry.** With the flag set, each gesture pushes one record
/// (live -- it is updated in place until the gesture ends) onto
/// `window.__dxCarouselWheel`, capped at `DEBUG_CAP` records: `{ id,
/// orientation, deltaMode, startedAtEdge, falseEnd, owned, notched, deltas,
/// releasedBy, peakBand, events, t0, lastEventAt, releasedAt, reengaged,
/// endedByRepush }` (times in ms from `t0`, deltas capped at
/// `DEBUG_DELTAS_CAP`). Without the flag nothing is allocated or pushed.
///
/// **Known open item (pending a real-device check).** Safari (and possibly
/// Firefox on some platforms) rubber-bands a scroll container even when a
/// gesture starts at rest at its edge, so D's band could stack on the
/// platform's own there (a double bounce). Not testable headlessly and
/// never solved by UA sniffing; the intended fix is to skip D's band where
/// the platform is observed to draw its own at a resting edge.
///
/// **Only at a true end of the data.** Whether
/// an edge may be owned is decided by ONE predicate in the script,
/// `isTrueEnd(side)`, at gesture start alongside the at-rest test, from
/// rendered markup: never for a looping carousel (`data-loop="true"` --
/// any `LoopMode`, plain or virtual), and otherwise only when the slide at
/// that physical edge (by rect, so RTL and vertical need nothing extra) has
/// `data-index` 0 or `data-slide-count - 1`. So a non-looping
/// [`CarouselVirtualContent`] bands at its real first/last item but not at
/// its rendered slice's edge mid-list, and plain non-looping content bands
/// at both ends. The telemetry's `falseEnd` names an edge the gesture started
/// at rest against that was refused this way. `prefers-reduced-motion:
/// reduce` draws no band (checked fresh per event, not cached at mount),
/// matching the pointer path's own reading of that preference.
pub(super) const CAROUSEL_WHEEL_BAND_JS: &str = concat!(carousel_true_end_js!(), "\
    const [id, orientation] = await dioxus.recv();
    const el = document.getElementById(id);
    if (!el) return;
    const horizontal = orientation === 'horizontal';

    // ---- Mode D's constants: the tuned bench defaults (rev 40
    // KNOB_DEFS), plus the component's own notched-wheel additions. ----
    // Wheel deltas are pointer-accelerated; the curve expects finger
    // travel (Firefox sent 150-590px per event on a recorded trackpad).
    const GAIN = 0.3;
    // WebKit's rubber-band slope at the start of the band. Lower is stiffer.
    const RUBBER_C = 0.55;
    // Where the curve flattens out, x the axis size. WebKit approaches
    // 1.0; 0.5 kept the hardest recorded flicks under ~28% of the width.
    const LIMIT = 0.5;
    // New gesture after this much silence. Measured rest after a real
    // burst is 64-78ms, so this sits just above it.
    const GAP_MS = 100;
    // Momentum = each (paired) event falls to at least this x the last;
    // below it is a hand jittering, not coasting (measured ~0.95/event).
    const SMOOTH = 0.8;
    // ...and to at most this x the last; above it is a hand still pushing
    // steadily (wheel-gestures' own 0.96).
    const RATIO_MAX = 0.96;
    // Re-push after a release = a delta above this x the last one.
    const REPRESS = 2;
    // Re-push once a stream has coasted = a delta above this x the last one
    // (a recorded re-push at slide 1 rose 1.6x, 1.97x, 1.39x per event).
    const COAST_RISE = 1.3;
    // Consecutive smoothly-decaying events that mean the fingers lifted
    // (also how many make a stream count as coasting).
    const RUNS = 5;
    // Safety release after silence: fingers resting still send nothing,
    // and a lift is normally caught by momentum or the cursor moving.
    const IDLE_MS = 1500;
    // A slow push has no momentum tail, so after the lift there is only
    // silence; release it after this much. Pushing again grabs it back.
    const SLOW_IDLE_MS = 250;
    // Slow push = peak step below this (recorded slow pushes peaked at
    // 5-16px per event, real flicks at 93-195px).
    const SLOW_PEAK_PX = 40;
    // Lift = the mouse cursor moves at least this far; smaller moves are
    // the hover updates browsers synthesise at the same coordinates.
    const LIFT_PX = 2;
    // Spring-back duration, ease-out cubic, like the pointer release.
    const HOME_MS = 340;
    // Hard cap on the drawn band, x the axis size. 1 = off.
    const CAP = 1;
    // Component addition: a notched mouse wheel has no momentum tail and
    // its 48-120px steps are above SLOW_PEAK_PX, so neither the decay test
    // nor the slow-push silence could release it and it would sit at the
    // band for the whole IDLE_MS backstop. Notches from a steadily turning
    // wheel arrive every ~16-100ms, so this is comfortably past the gap
    // between two of them and well short of the backstop.
    const NOTCH_IDLE_MS = 140;
    // A deltaMode-0 step only counts as a notch at or above this. Below
    // it the slow-push release already applies, and recorded Firefox
    // trackpad data has equal consecutive steps under it far more often.
    const NOTCH_MIN_PX = SLOW_PEAK_PX;
    // Re-engage after a release when the deltas rise again by this much:
    // the fingers never lifted (a slow push that eased off looked like
    // decay). The bench's own literal.
    const REENGAGE_RISE = 1.15;
    // Decay only counts once the peak is clearly above the device's own
    // smallest step (learned below), so a slow flat push is left to the
    // silence release. Fallback until the device has spoken.
    const PLATEAU_MULT = 3;
    const PLATEAU_FALLBACK_PX = 4;
    // deltaMode 1 (lines) -> px, the common browser line height.
    const LINE_PX = 16;
    const DEBUG_CAP = 50;
    const DEBUG_DELTAS_CAP = 80;

    let dbg = false;
    try {
        dbg = window.localStorage.getItem('dx-carousel-debug') === '1';
    } catch (err) {}
    if (window.__dxCarouselDebug === true) { dbg = true; }
    let rec = null;

    const ow = {
        lastAt: 0,
        startMin: false, // gesture began at rest at the physical left/top edge
        startMax: false, // ...at the physical right/bottom edge
        owned: null,
        raw: 0,
        peak: 0,
        lastAbs: Infinity,
        lowRun: 0,
        released: false,
        prevRaw: 0,
        lastPair: Infinity,
        rawLast: 0,
        homeToken: 0,
        homing: false,
        idleTimer: 0,
        coasting: false,
        coastRun: 0,
        coastPrev: 0,
        notched: false, // the latest event of this gesture was a notch
        falseEnd: null, // resting at an edge that is not a true end (`isTrueEnd`)
    };
    let visible = 0; // the band currently drawn, signed physical px
    let deviceMinDelta = Infinity; // learned: the smallest step this hardware sends
    let notchStep = 0; // the notch size, once identified
    let prevNotchAbs = 0;

    function axisSize() {
        // See `CAROUSEL_DRAG_JS`'s own identical helper's doc: the clipping
        // viewport wrapper's own size, not `el`'s (the gap model's
        // negative-margin compensation widens `el` by the gap).
        const track = el.parentElement;
        const size = horizontal
            ? (track ? track.clientWidth : el.clientWidth)
            : (track ? track.clientHeight : el.clientHeight);
        return size || 320;
    }
    // WebKit's rubber-band curve, b(x) = x*c*d / (d + c*x): slope c at 0,
    // asymptote d = LIMIT x the axis size.
    function webkitRubber(x) {
        const w = axisSize();
        const d = w * LIMIT;
        const ax = Math.abs(x);
        let v = (ax * RUBBER_C * d) / (d + RUBBER_C * ax);
        v = Math.min(v, w * CAP);
        return x < 0 ? -v : v;
    }
    // Inverse (ignoring the cap), so a push that resumes while the band
    // springs home continues from wherever it had got to, never jumping.
    function webkitRubberInverse(v) {
        const d = axisSize() * LIMIT;
        const av = Math.min(Math.abs(v), d * 0.999);
        const x = (av * d) / (RUBBER_C * (d - av));
        return v < 0 ? -x : x;
    }
    // Physical, direction-agnostic: min = left/top, max = right/bottom.
    // `min >= 0` means no content left beyond the min edge; `max <= 0`, none
    // beyond the max edge. Measured against `el`'s own rect, which carries
    // the same transform as its children, so the band cancels out.
    function edgeGaps() {
        const children = el.children;
        if (!children.length) {
            return { min: 0, max: 0 };
        }
        const c = el.getBoundingClientRect();
        let lo = Infinity;
        let hi = -Infinity;
        for (let i = 0; i < children.length; i++) {
            const r = children[i].getBoundingClientRect();
            const s = horizontal ? r.left : r.top;
            const e = horizontal ? r.right : r.bottom;
            if (s < lo) lo = s;
            if (e > hi) hi = e;
        }
        return {
            min: lo - (horizontal ? c.left : c.top),
            max: hi - (horizontal ? c.right : c.bottom),
        };
    }
    // Only at a true end of the data: the shared `dxIsTrueEnd`
    // (`carousel_true_end_js!`), decided once per gesture below.
    const isTrueEnd = (side) => dxIsTrueEnd(el, horizontal, side);
    function plateauCeiling() {
        return isFinite(deviceMinDelta) ? deviceMinDelta * PLATEAU_MULT : PLATEAU_FALLBACK_PX;
    }
    // The delta along this carousel's own axis, or 0 when the event is not
    // this carousel's to react to (vertical intent over a horizontal one).
    function axisDelta(e) {
        const dx = e.deltaX;
        const dy = e.deltaY;
        if (horizontal) {
            if (Math.abs(dx) > Math.abs(dy)) return dx;
            if (e.shiftKey && dx === 0) return dy; // Shift+wheel, reported as deltaY
            return 0;
        }
        if (e.shiftKey && dx === 0) return 0; // Shift+wheel means horizontal intent
        return Math.abs(dy) > Math.abs(dx) ? dy : 0;
    }
    function isNotch(e, ad) {
        if (e.deltaMode !== 0) {
            return true;
        }
        let notch = false;
        if (ad < NOTCH_MIN_PX) {
            notchStep = 0;
        } else if (notchStep && Math.round(ad / notchStep) >= 1
            && Math.abs(ad - Math.round(ad / notchStep) * notchStep) < 0.5) {
            notch = true;
        } else if (Math.abs(ad - prevNotchAbs) < 0.5) {
            notchStep = ad;
            notch = true;
        } else {
            notchStep = 0;
        }
        prevNotchAbs = ad;
        return notch;
    }

    function recBegin(e) {
        if (!dbg) return;
        rec = {
            id,
            orientation,
            deltaMode: e.deltaMode,
            startedAtEdge: ow.startMin ? (horizontal ? 'left' : 'top')
                : ow.startMax ? (horizontal ? 'right' : 'bottom') : 'none',
            falseEnd: ow.falseEnd,
            owned: null,
            notched: false,
            deltas: [],
            releasedBy: null,
            peakBand: 0,
            events: 0,
            t0: performance.now(),
            lastEventAt: 0,
            releasedAt: null,
            reengaged: 0,
            endedByRepush: false,
        };
        const log = (window.__dxCarouselWheel = window.__dxCarouselWheel || []);
        log.push(rec);
        if (log.length > DEBUG_CAP) log.shift();
    }
    function recSample(d) {
        if (!rec) return;
        rec.events++;
        rec.lastEventAt = Math.round(performance.now() - rec.t0);
        if (rec.deltas.length < DEBUG_DELTAS_CAP) rec.deltas.push(Math.round(d * 10) / 10);
        if (ow.notched) rec.notched = true;
        rec.owned = ow.owned;
        const b = Math.round(Math.abs(visible) * 10) / 10;
        if (b > rec.peakBand) rec.peakBand = b;
    }
    function recRelease(how) {
        if (!rec || rec.releasedBy) return;
        rec.releasedBy = how;
        rec.releasedAt = Math.round(performance.now() - rec.t0);
    }

    function apply(v) {
        visible = v;
        if (Math.abs(v) > 0.05) {
            const px = v.toFixed(2);
            el.style.transform = horizontal ? `translateX(${px}px)` : `translateY(${px}px)`;
        } else {
            el.style.transform = '';
        }
    }
    function clearIdle() {
        if (ow.idleTimer) {
            window.clearTimeout(ow.idleTimer);
            ow.idleTimer = 0;
        }
    }
    function cancelHome() {
        if (!ow.homing) return;
        ow.homeToken++;
        ow.homing = false;
    }
    // Ease-out cubic spring-back from wherever the band is. Never restarted
    // while one is in flight; superseded (token) by a push that resumes.
    function home() {
        clearIdle();
        ow.raw = 0;
        if (ow.homing) return;
        const from = visible;
        if (Math.abs(from) < 0.5 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            apply(0);
            return;
        }
        ow.homing = true;
        const token = ++ow.homeToken;
        const t0 = performance.now();
        (function step() {
            if (token !== ow.homeToken) return;
            const t = Math.min(1, (performance.now() - t0) / HOME_MS);
            apply(from * Math.pow(1 - t, 3));
            if (t < 1) {
                window.requestAnimationFrame(step);
            } else {
                apply(0);
                ow.homing = false;
            }
        })();
    }
    function releaseOwned(how) {
        recRelease(how);
        ow.released = true;
        home();
    }

    // THE RULE, restated for this listener: no `preventDefault`, no
    // `scrollBy`, no scroll-position read or write, no `scroll-snap-type`
    // write, ever. `passive: true`: this only observes geometry and writes
    // its own `transform`.
    const onWheel = (e) => {
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            if (visible || ow.homing) {
                cancelHome();
                clearIdle();
                ow.raw = 0;
                apply(0);
            }
            return;
        }
        const raw = axisDelta(e);
        if (!raw) return;
        const d = e.deltaMode === 1 ? raw * LINE_PX : e.deltaMode === 2 ? raw * axisSize() : raw;
        const cd = -d; // desired physical content displacement, + = toward right/bottom
        const ad = Math.abs(d);
        const now = performance.now();
        if (ad > 0 && ad < deviceMinDelta) deviceMinDelta = ad;
        const notch = isNotch(e, ad);

        // Coasting is tracked for EVERY stream, owned or not (bench rev 40):
        // a native scroll that carried the user into the edge must not
        // swallow the next push into its own momentum tail.
        const fallOk = ow.coastPrev > 0 && ad <= ow.coastPrev * RATIO_MAX && ad >= ow.coastPrev * SMOOTH;
        ow.coastRun = fallOk ? ow.coastRun + 1 : 0;
        if (ow.coastRun >= RUNS) ow.coasting = true;
        ow.coastPrev = ad;
        const rise = ow.coasting ? COAST_RISE : REPRESS;
        const repush = ow.lastAt && (ow.released || ow.coasting) && ow.rawLast > 0
            && ad > Math.max(2, ow.rawLast * rise);
        ow.rawLast = ad;
        // `!ow.lastAt`: the very first event always starts one, however
        // soon after the page's time origin it arrives.
        if (!ow.lastAt || now - ow.lastAt > GAP_MS || repush) {
            if (rec && repush && now - ow.lastAt <= GAP_MS) rec.endedByRepush = true;
            // A new gesture. Where the scroller RESTS decides ownership, read
            // BEFORE this event scrolls it (a wheel event is dispatched ahead
            // of its own default scroll).
            const g0 = edgeGaps();
            const restMin = g0.min >= -0.5;
            const restMax = g0.max <= 0.5;
            ow.startMin = restMin && isTrueEnd('min');
            ow.startMax = restMax && isTrueEnd('max');
            ow.falseEnd = (restMin && !ow.startMin) ? (horizontal ? 'left' : 'top')
                : (restMax && !ow.startMax) ? (horizontal ? 'right' : 'bottom') : null;
            ow.owned = null;
            ow.peak = 0;
            ow.lastAbs = Infinity;
            ow.lowRun = 0;
            ow.released = false;
            ow.prevRaw = 0;
            ow.lastPair = Infinity;
            ow.coasting = false;
            ow.coastRun = 0;
            ow.coastPrev = ad;
            // A previous gesture's pending silence release belongs to that
            // gesture; this one re-arms its own on its first drawn event.
            clearIdle();
            recBegin(e);
        }
        ow.lastAt = now;
        ow.notched = notch;

        const g = edgeGaps();
        const atLimit = cd > 0 ? g.min >= -0.5 : g.max <= 0.5;
        if (atLimit && ow.owned === null) {
            ow.owned = cd > 0 ? ow.startMin : ow.startMax;
        }

        if (ow.owned && ow.released && atLimit && ow.lastAbs !== Infinity && ad > ow.lastAbs * REENGAGE_RISE) {
            // Rising again after a release: the fingers never lifted. Grab
            // the band back from wherever the spring-back has got to.
            cancelHome();
            ow.raw = webkitRubberInverse(visible);
            ow.released = false;
            ow.lowRun = 0;
            ow.peak = ad;
            ow.lastPair = Infinity;
            ow.prevRaw = 0;
            if (rec) {
                rec.reengaged++;
                rec.releasedBy = null;
                rec.releasedAt = null;
            }
        }
        if (!ow.owned || ow.released) {
            // The platform's bounce, or a spent tail. A band left over from
            // an earlier gesture never outlives one that is not drawing it.
            if (!ow.owned && visible && !ow.homing) {
                recRelease('superseded');
                home();
            }
            if (ow.released) ow.lastAbs = ad;
            recSample(d);
            return;
        }
        if (!atLimit) {
            releaseOwned('reversed');
            recSample(d);
            return;
        }

        // Fingers lifted -> SMOOTH geometric decay. Paired events damp a
        // hand's jitter; each pair must fall, but by a bounded ratio.
        const pair = ow.prevRaw ? (ad + ow.prevRaw) / 2 : ad;
        ow.prevRaw = ad;
        const smoothFall = ow.lastPair !== Infinity && pair <= ow.lastPair * RATIO_MAX
            && pair >= ow.lastPair * SMOOTH;
        if (ad > ow.peak) ow.peak = ad;
        if (ow.peak > plateauCeiling() && smoothFall) ow.lowRun++;
        else ow.lowRun = 0;
        ow.lastPair = pair;
        ow.lastAbs = ad;
        if (ow.lowRun >= RUNS) {
            releaseOwned('smooth-decay');
            recSample(d);
            return;
        }

        if (ow.homing) {
            cancelHome();
            ow.raw = webkitRubberInverse(visible);
        }
        ow.raw += cd * GAIN;
        apply(webkitRubber(ow.raw));
        recSample(d);

        // Silence is the only lift signal a slow push or a notched wheel
        // will ever give; everything else waits for decay, the cursor, or
        // the backstop. Pushing again after a release re-engages.
        const how = ow.notched ? 'notch-silence' : ow.peak < SLOW_PEAK_PX ? 'slow-silence' : 'idle';
        const ms = ow.notched ? NOTCH_IDLE_MS : ow.peak < SLOW_PEAK_PX ? SLOW_IDLE_MS : IDLE_MS;
        clearIdle();
        ow.idleTimer = window.setTimeout(() => {
            ow.idleTimer = 0;
            if (!ow.released) releaseOwned(how);
        }, ms);
    };

    // The web exposes neither the trackpad's touch state nor the scroll
    // phase, but a two-finger scroll never moves the cursor: after a lift
    // the next thing that moves it is a finger. Require a real
    // displacement -- browsers synthesise same-coordinate moves on scroll.
    let liftX = null;
    let liftY = null;
    const onPointerMove = (e) => {
        if (e.pointerType && e.pointerType !== 'mouse') return;
        if (liftX === null) {
            liftX = e.clientX;
            liftY = e.clientY;
            return;
        }
        const moved = Math.abs(e.clientX - liftX) + Math.abs(e.clientY - liftY);
        liftX = e.clientX;
        liftY = e.clientY;
        if (!ow.owned || ow.released || !visible) return;
        if (moved >= LIFT_PX) releaseOwned('cursor-moved');
    };
    // A mouse/pen press hands the element's transform to `CAROUSEL_DRAG_JS`:
    // drop any wheel band first, so the two never both own it.
    const onPointerDown = (e) => {
        if (e.pointerType === 'touch' || e.button !== 0) return;
        if (!visible && !ow.homing) return;
        recRelease('pointerdown');
        cancelHome();
        clearIdle();
        ow.raw = 0;
        ow.released = true;
        apply(0);
    };

    listen(el, 'wheel', onWheel, { passive: true });
    listen(el, 'pointerdown', onPointerDown, { passive: true });
    listen(window, 'pointermove', onPointerMove, { passive: true });
    cleanup(() => {
        cancelHome();
        clearIdle();
        if (visible) {
            visible = 0;
            el.style.transform = '';
        }
    });");

/// Attach [`CAROUSEL_WHEEL_BAND_JS`] to the element with the given `id` for
/// as long as the calling component stays mounted -- mirrors
/// [`use_carousel_drag`]'s own shape. Always attached: where the band may
/// engage is decided per gesture inside the script (`isTrueEnd`), from the
/// `data-loop`/`data-slide-count`/`data-index` markup, so the rule has one
/// home instead of a per-component `enabled` gate. A wheel/trackpad user has
/// no other opt-out today; the band is purely cosmetic feedback on top of
/// scrolling that happens regardless.
pub(super) fn use_carousel_wheel_band(
    id: impl Readable<Target = String> + Copy + 'static,
    orientation: ReadSignal<CarouselOrientation>,
) {
    crate::js_listener::use_js_listeners(move || {
        let id = id.cloned();
        let orientation_str = orientation().as_str().to_string();
        Some(
            crate::js_listener::JsListeners::install(CAROUSEL_WHEEL_BAND_JS)
                .arg((id, orientation_str)),
        )
    });
}
