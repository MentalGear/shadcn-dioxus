#[cfg(doc)]
use super::*;

/// THE one home of "may an edge rubber-band engage here?" -- the rule
/// shared by both band bridges ([`CAROUSEL_WHEEL_BAND_JS`] and
/// [`CAROUSEL_DRAG_JS`]): only at a TRUE end of the data. Expands to one JS
/// function literal, `dxIsTrueEnd(el, horizontal, side)`, that `concat!`
/// splices into both scripts, so there is no second copy to drift (a
/// scroller-attached helper like `__dxWhenIdle` would instead depend on which
/// bridge's eval runs first).
///
/// Read from rendered markup, never a Rust signal: `data-loop` and
/// `data-slide-count` on the scroller, and the `data-index` of the slide at
/// that physical edge (`side` is `'min'` = left/top or `'max'` =
/// right/bottom, found by rect, so RTL and vertical need no branch). A
/// looping carousel (any [`LoopMode`], plain or virtual) has no true end; a
/// [`CarouselVirtualContent`] window's slice edge mid-list (e.g. items 40-44
/// of 100) is not one either -- the scroller simply clamps there and the
/// window re-centres once idle. Plain, non-looping content always has index
/// 0 / count - 1 at its two ends, so it is unaffected.
macro_rules! carousel_true_end_js {
    () => {
        "
    function dxIsTrueEnd(el, horizontal, side) {
        if (el.dataset.loop === 'true') return false;
        const count = parseInt(el.dataset.slideCount, 10);
        if (!(count > 0)) return false;
        let pick = null;
        let best = side === 'min' ? Infinity : -Infinity;
        for (const child of el.children) {
            if (child.dataset.index === undefined) continue;
            const r = child.getBoundingClientRect();
            const v = side === 'min' ? (horizontal ? r.left : r.top) : (horizontal ? r.right : r.bottom);
            if (side === 'min' ? v < best : v > best) {
                best = v;
                pick = child;
            }
        }
        const index = pick ? parseInt(pick.dataset.index, 10) : NaN;
        return index === 0 || index === count - 1;
    }
"
    };
}

/// Opt-in diagnostics shared by every carousel JS bridge: one JS literal,
/// spliced (`concat!`) into each script, defining `dxTraceOn` and
/// `dxTrace(el, kind, data)`. ONLY when `localStorage["dx-carousel-debug"]
/// === "1"` (the same flag the wheel band's `window.__dxCarouselWheel`
/// telemetry uses), or a page sets `window.__dxCarouselDebug = true` before
/// the carousel mounts (for hosts whose storage throws, e.g. sandboxed
/// frames), does `dxTrace` append `{ t: performance.now(), id, kind,
/// sl: scrollLeft, st: scrollTop, ...data }` to `window.__dxCarouselTrace`,
/// capped at the newest 5000 records. The flag is read once per page (cached
/// as `window.__dxCarouselTraceOn`), so with it off every call is a single
/// boolean check and returns -- nothing is allocated, observed or written,
/// and no rendered markup ever changes (hydration parity unaffected).
///
/// Kinds: `input` (a wheel event the idle gate saw), `gate-hold` /
/// `gate-release` (the browser-idle gate starting to defer, and letting a
/// settle or queued writes through), `scroll-ended` (a `scrollend`, the
/// no-`scrollend` debounce, a resize, or a drag's `dx-carousel-rest`),
/// `settle` (what the tracking bridge reported), `window` (a virtual
/// window re-render: old -> new rendered data-index/position ranges),
/// `scroll-write` (every scroll write a bridge makes: method, delta,
/// behavior, snap state), `scroll-to-skip`, and `drag-*`.
macro_rules! carousel_trace_js {
    () => {
        "
    const dxTraceOn = (() => {
        if (window.__dxCarouselTraceOn === undefined) {
            let on = false;
            try { on = window.localStorage.getItem('dx-carousel-debug') === '1'; } catch (err) {}
            if (window.__dxCarouselDebug === true) { on = true; }
            window.__dxCarouselTraceOn = on;
        }
        return window.__dxCarouselTraceOn;
    })();
    function dxTrace(el, kind, data) {
        if (!dxTraceOn) return;
        const log = (window.__dxCarouselTrace = window.__dxCarouselTrace || []);
        const rec = { t: Math.round(performance.now() * 10) / 10, id: el ? el.id : null, kind,
            sl: el ? Math.round(el.scrollLeft * 10) / 10 : null, st: el ? Math.round(el.scrollTop * 10) / 10 : null };
        if (data) Object.assign(rec, data);
        log.push(rec);
        if (log.length > 5000) log.splice(0, log.length - 5000);
    }
"
    };
}
